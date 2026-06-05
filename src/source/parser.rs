//! The parser: lexed [`Line`]s → a typed [`Database`] of `Rule`/`Zone`/`Link` records.
//!
//! The parser is deliberately *complete* with respect to the source grammar — it parses
//! rules, multi-era zones with `UNTIL` continuations, and links — independently of what the
//! compiler can currently turn into TZif. Keeping the parser general means:
//!
//! * we validate the whole file the way `zic` does (so we don't silently ignore typos in
//!   rules a selected zone happens not to use); and
//! * the compiler inherits a stable, complete front-end as its supported subset grows.
//!
//! The *unsupported-construct* decision lives in `compile`, not here: the parser's job is to
//! represent the source faithfully; the compiler decides what it can turn into TZif correctly
//! (and fails closed otherwise). Record keywords are matched as `zic`-style unambiguous
//! prefixes (see the `record_keyword` helper), so the parser reads both the canonical
//! `Rule`/`Zone`/`Link` spelling **and** the zishrink-abbreviated `R`/`Z`/`L` form used by the
//! installed single-file `tzdata.zi` — i.e. zic-rs can parse the real installed source directly.
//!
//! ## Zone continuation handling (the one piece of cross-line state)
//!
//! A `Zone` line whose era ends with an `UNTIL` is continued by the following line(s),
//! which omit the `Zone` keyword and the name. A line is a continuation iff the previous
//! era carried an `UNTIL`. We track exactly that with `expect_continuation`.

use std::path::Path;

use super::LegacySource;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::error::{Error, Result};
use crate::model::calendar::OnDay;
use crate::model::time::{parse_offset, parse_save, parse_time_of_day};
use crate::model::{
    Database, LinkRecord, Origin, RuleRecord, Until, YearBound, YearType, ZoneEra, ZoneRecord,
    ZoneRules,
};

use super::lexer::tokenize_with;
use super::names;
use super::records::Line;

/// Parse a whole source file's bytes into `db`, appending its records. UTF-8 (modern contract).
pub fn parse_into(bytes: &[u8], file: &Path, db: &mut Database) -> Result<()> {
    parse_into_with(bytes, file, db, LegacySource::default())
}

/// Like [`parse_into`], but `legacy` opts into bounded historical-source replay modes
/// ([`LegacySource`]): Latin-1 comments (LEGACY-SOURCE.1) and/or the `yearistype` Rule `TYPE`
/// ecology (YEARISTYPE.1). With the default (all-off) bundle this is byte-identical to [`parse_into`].
pub fn parse_into_with(
    bytes: &[u8],
    file: &Path,
    db: &mut Database,
    legacy: LegacySource,
) -> Result<()> {
    let lines = tokenize_with(bytes, file, legacy.latin1)?;

    // Duplicate-zone detection (T13.2): a name → first-definition-line map. Seeded from any zones
    // already in `db` (prior files) with line 0 ("earlier, line unknown") so cross-file duplicates are
    // caught the same as within-file ones — matching reference `zic`'s single-namespace "duplicate zone
    // name N" error. Built once (O(n)); checked/updated per `Zone`.
    let mut seen_zones: std::collections::HashMap<String, usize> =
        db.zones.iter().map(|z| (z.name.clone(), 0usize)).collect();

    let mut expect_continuation = false;
    for line in &lines {
        if expect_continuation {
            // The previous era ended with UNTIL; this line continues that zone.
            let era = parse_era(&line.fields, 0, file, line)?;
            expect_continuation = era.until.is_some();
            let zone = db
                .zones
                .last_mut()
                .expect("continuation without an open zone is impossible by construction");
            zone.eras.push(era);
            continue;
        }

        let keyword = line.keyword().unwrap_or("");
        match record_keyword(keyword) {
            Some(RecordKind::Rule) => {
                let r = parse_rule(line, file, legacy.yearistype)?;
                db.rules.entry(r.name.clone()).or_default().push(r);
            }
            Some(RecordKind::Zone) => {
                let (zone, more) = parse_zone(line, file)?;
                // Reject a second definition of the same zone name (T13.2 — structural). zic-rs
                // records the original line when it was in this file (`> 0`); a `0` means an earlier
                // file, where the precise line is not tracked.
                if let Some(&orig) = seen_zones.get(&zone.name) {
                    let where_orig = if orig > 0 {
                        format!(" (originally defined at line {orig})")
                    } else {
                        " (originally defined in an earlier input)".to_string()
                    };
                    return Err(diag(
                        DiagnosticCode::DuplicateZone,
                        format!("duplicate zone name {:?}{where_orig}", zone.name),
                        file,
                        line,
                    ));
                }
                seen_zones.insert(zone.name.clone(), line.number);
                expect_continuation = more;
                db.zones.push(zone);
            }
            Some(RecordKind::Link) => {
                db.links.push(parse_link(line, file)?);
            }
            None => {
                // `record_keyword == None` — the first token is not `Rule`/`Zone`/`Link`. Split by
                // column (T13.2): an **indented** line (col > 0) is continuation-shaped but no zone is
                // open to continue it (`ContinuationWithoutZone`); a line in **command position**
                // (col 0) is an unrecognised keyword (`UnknownLineType`). Reference `zic` folds both
                // into "input line of unknown type"; the continuation refinement is an intentional,
                // documented divergence (see `docs/zic-warning-parity.md`).
                let indented = line.fields.first().is_some_and(|f| f.col > 0);
                let (code, msg) = if indented {
                    (
                        DiagnosticCode::ContinuationWithoutZone,
                        format!("continuation line {keyword:?} has no open zone to continue"),
                    )
                } else {
                    (
                        DiagnosticCode::UnknownLineType,
                        format!("input line of unknown type beginning with {keyword:?}"),
                    )
                };
                return Err(diag(code, msg, file, line));
            }
        }
    }

    if expect_continuation {
        // A zone whose final era had UNTIL but no continuation is malformed.
        let last = db.zones.last();
        let name = last.map(|z| z.name.as_str()).unwrap_or("<zone>");
        return Err(Error::message(format!(
            "zone {name:?} ends with an UNTIL but has no continuation line"
        )));
    }
    Ok(())
}

/// The three record kinds zic's main-file keyword table recognises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordKind {
    Rule,
    Zone,
    Link,
}

/// Classify a line's leading keyword, accepting any **unambiguous case-insensitive prefix** of
/// `Rule` / `Zone` / `Link` — exactly as reference `zic` does.
///
/// `zic`'s `byword` matches a keyword against its table by exact match or unambiguous prefix.
/// The main zone-file table is `{Rule, Zone, Link}` only: `Leap`/`Expires` are recognised
/// solely in a *leapseconds* file, so in a zone file `Leap` is "input line of unknown type"
/// (verified against reference `zic` 2026b) and a bare `L` is therefore the **unambiguous**
/// abbreviation of `Link`. Because the three words share no common prefix, every non-empty
/// prefix maps to at most one — so this never returns an ambiguous match for them.
///
/// This is what lets zic-rs read the **zishrink-abbreviated** single-file `tzdata.zi` directly
/// (it uses `R`/`Z`/`L` for the ~2700 records). The month/weekday/year tokens are already
/// accepted as `zic`-style prefixes by [`crate::source::names`], so no other de-abbreviation is
/// needed. (`Leap`/`Expires` remain unrecognised here, matching `zic`; leap seconds are out of
/// scope — see `docs/unsupported-syntax.md`.)
fn record_keyword(word: &str) -> Option<RecordKind> {
    if word.is_empty() {
        return None;
    }
    let needle = word.to_ascii_lowercase();
    const TABLE: [(&str, RecordKind); 3] = [
        ("rule", RecordKind::Rule),
        ("zone", RecordKind::Zone),
        ("link", RecordKind::Link),
    ];
    let mut found = None;
    for (full, kind) in TABLE {
        if full.starts_with(&needle) {
            if found.is_some() {
                return None; // ambiguous prefix (unreachable for Rule/Zone/Link)
            }
            found = Some(kind);
        }
    }
    found
}

/// Build a located error diagnostic for `line`.
fn diag(code: DiagnosticCode, msg: impl Into<String>, file: &Path, line: &Line) -> Error {
    Error::from(Diagnostic::error(code, msg, file, line.number))
}

/// Turn a `(code, message)` field-level failure into a located error.
fn field_err(e: (DiagnosticCode, String), file: &Path, line: &Line, col: usize) -> Error {
    Error::from(Diagnostic::error(e.0, e.1, file, line.number).with_span(col, col))
}

// ---- Rule -----------------------------------------------------------------------------

/// Parse a `Rule NAME FROM TO TYPE IN ON AT SAVE LETTER` line (10 fields). `legacy_yearistype`
/// admits the historical `TYPE` predicates (YEARISTYPE.1); with it off, any non-`-` TYPE is a hard
/// error (the modern default — `zic` itself removed the `-y`/`yearistype` ecology in tzcode 2020a).
fn parse_rule(line: &Line, file: &Path, legacy_yearistype: bool) -> Result<RuleRecord> {
    let f = &line.fields;
    if f.len() != 10 {
        return Err(diag(
            DiagnosticCode::InvalidFieldCount,
            format!("Rule line needs 10 fields, found {}", f.len()),
            file,
            line,
        ));
    }
    let name = f[1].text.clone();
    let from = parse_year(&f[2].text).map_err(|e| field_err(e, file, line, f[2].col))?;
    let to = parse_to_year(&f[3].text, from).map_err(|e| field_err(e, file, line, f[3].col))?;
    let year_type = parse_year_type(&f[4].text, from, to, legacy_yearistype, file, line)?;
    let in_month = names::month(&f[5].text).map_err(|e| field_err(e, file, line, f[5].col))?;
    let on = parse_on_day(&f[6].text).map_err(|e| field_err(e, file, line, f[6].col))?;
    let at = parse_time_of_day(&f[7].text).map_err(|e| field_err(e, file, line, f[7].col))?;
    let save = parse_save(&f[8].text).map_err(|e| field_err(e, file, line, f[8].col))?;
    // LETTER: '-' denotes the empty variable part.
    let letter = if f[9].text == "-" {
        String::new()
    } else {
        f[9].text.clone()
    };

    Ok(RuleRecord {
        name,
        from,
        to,
        in_month,
        on,
        at,
        save,
        letter,
        year_type,
        origin: Origin::new(file, line.number),
    })
}

/// Resolve the Rule `TYPE` field (the 5th field). `-` is always [`YearType::All`].
///
/// **Default (modern) mode** fails closed on any non-`-` TYPE: the `-y`/`yearistype` ecology was
/// removed from reference `zic` in tzcode 2020a, and silently *ignoring* a year-parity predicate would
/// mis-compile the rule (firing it every year). This is a bucket-3 safer divergence from the parser's
/// historical lenience — it never affects modern tzdata (all `-`).
///
/// **`--legacy-yearistype`** admits the four historical predicates (`even`/`odd`/`uspres`/`nonpres`,
/// anchored to `yearistype.sh` v7.4). An unknown ("wild") type is still a hard error in either mode —
/// the historical script itself exits 1 on a wild type. A *typed single year* with a predicate
/// mirrors reference `zic`'s "typed single year" error (a predicate is meaningless on one year).
fn parse_year_type(
    token: &str,
    from: i32,
    to: YearBound,
    legacy_yearistype: bool,
    file: &Path,
    line: &Line,
) -> Result<YearType> {
    if token == "-" {
        return Ok(YearType::All);
    }
    if !legacy_yearistype {
        return Err(diag(
            DiagnosticCode::UnsupportedRuleType,
            format!(
                "Rule TYPE {token:?} is the obsolete yearistype ecology (removed from reference zic \
                 in tzcode 2020a); pass --legacy-yearistype to replay historical pre-2000f source"
            ),
            file,
            line,
        ));
    }
    let year_type = YearType::from_field(token).ok_or_else(|| {
        diag(
            DiagnosticCode::UnsupportedRuleType,
            format!(
                "unknown Rule TYPE {token:?} (historical yearistype admits only \
                 even/odd/uspres/nonpres)"
            ),
            file,
            line,
        )
    })?;
    // Mirror reference `zic`: a year-type on a single-year rule is an error ("typed single year") —
    // a parity predicate is only meaningful across a range.
    if matches!(to, YearBound::Year(t) if t == from) {
        return Err(diag(
            DiagnosticCode::UnsupportedRuleType,
            format!(
                "typed single year: Rule TYPE {token:?} on a one-year rule ({from}) is invalid"
            ),
            file,
            line,
        ));
    }
    Ok(year_type)
}

/// Parse a `FROM` year: an integer, or the keywords `minimum`/`maximum`.
///
/// `minimum`/`maximum` are matched as `zic`-style **unambiguous case-insensitive prefixes**, so
/// a bare `m` is ambiguous (errors), `mi` is `minimum`, `ma` is `maximum`.
///
/// **`FROM = minimum` is obsolete.** Reference `zic` (2026b) emits *"FROM year 'minimum' is
/// obsolete; treated as 1900"* and coerces it to the year **1900** — it does **not** expand into
/// an unbounded past. zic-rs follows the same finite-1900 semantics. (tzdata 2026b uses it zero
/// times; it exists only for legacy-source compatibility. Surfacing the obsolescence as a
/// compile-time *warning* is a tracked follow-up — the compile path has no warning sink yet; see
/// `docs/reference-zic-semantics.md`.)
fn parse_year(s: &str) -> std::result::Result<i32, (DiagnosticCode, String)> {
    if s.is_empty() {
        return Err((
            DiagnosticCode::UnsupportedYearType,
            "empty year".to_string(),
        ));
    }
    let low = s.to_ascii_lowercase();
    let is_min = "minimum".starts_with(&low);
    let is_max = "maximum".starts_with(&low);
    match (is_min, is_max) {
        (true, true) => Err((
            DiagnosticCode::UnsupportedYearType,
            format!("ambiguous year {s:?} — write `min`/`minimum` or `max`/`maximum`"),
        )),
        (true, false) => Ok(1900), // obsolete; coerced to 1900, exactly as reference `zic`
        (false, true) => Ok(i32::MAX),
        (false, false) => s.parse::<i32>().map_err(|_| {
            (
                DiagnosticCode::UnsupportedYearType,
                format!("invalid year {s:?}"),
            )
        }),
    }
}

/// Parse a `TO` year: an integer, `only` (== `from`), or the keywords `minimum`/`maximum`,
/// each accepted as a `zic`-style unambiguous case-insensitive prefix (`o`/`on`→`only`,
/// `mi`→`minimum`, `ma`→`maximum`; a bare `m` is ambiguous and rejected). `minimum` is the
/// obsolete spelling coerced to 1900 (see [`parse_year`]); `maximum` is the open-ended tail.
fn parse_to_year(s: &str, from: i32) -> std::result::Result<YearBound, (DiagnosticCode, String)> {
    if s.is_empty() {
        return Err((
            DiagnosticCode::UnsupportedYearType,
            "empty year".to_string(),
        ));
    }
    let low = s.to_ascii_lowercase();
    if "only".starts_with(&low) {
        // `only`/`minimum`/`maximum` share no first letter, so `only`'s prefixes are exclusive.
        return Ok(YearBound::Year(from));
    }
    let is_min = "minimum".starts_with(&low);
    let is_max = "maximum".starts_with(&low);
    match (is_min, is_max) {
        (true, true) => Err((
            DiagnosticCode::UnsupportedYearType,
            format!("ambiguous year {s:?} — write `min`/`minimum` or `max`/`maximum`"),
        )),
        (true, false) => Ok(YearBound::Year(1900)), // obsolete; coerced to 1900, as reference `zic`
        (false, true) => Ok(YearBound::Max),
        (false, false) => s.parse::<i32>().map(YearBound::Year).map_err(|_| {
            (
                DiagnosticCode::UnsupportedYearType,
                format!("invalid year {s:?}"),
            )
        }),
    }
}

/// Parse an `ON` day spec: `5`, `lastSun`, `Sun>=8`, `Sun<=25`.
fn parse_on_day(s: &str) -> std::result::Result<OnDay, (DiagnosticCode, String)> {
    // Numeric day-of-month.
    if let Ok(d) = s.parse::<u8>() {
        if (1..=31).contains(&d) {
            return Ok(OnDay::Day(d));
        }
        return Err((
            DiagnosticCode::InvalidDayRule,
            format!("day {d} out of range"),
        ));
    }
    // `lastWeekday`.
    if let Some(rest) = strip_prefix_ci(s, "last") {
        let wd = names::weekday(rest)?;
        return Ok(OnDay::Last(wd));
    }
    // `Weekday>=N` / `Weekday<=N`.
    if let Some((wd_str, n)) = s.split_once(">=") {
        let wd = names::weekday(wd_str)?;
        let n = parse_dom(n)?;
        return Ok(OnDay::OnAfter(wd, n));
    }
    if let Some((wd_str, n)) = s.split_once("<=") {
        let wd = names::weekday(wd_str)?;
        let n = parse_dom(n)?;
        return Ok(OnDay::OnBefore(wd, n));
    }
    Err((
        DiagnosticCode::InvalidDayRule,
        format!("unrecognised ON day spec {s:?}"),
    ))
}

fn parse_dom(s: &str) -> std::result::Result<u8, (DiagnosticCode, String)> {
    s.parse::<u8>()
        .ok()
        .filter(|d| (1..=31).contains(d))
        .ok_or_else(|| {
            (
                DiagnosticCode::InvalidDayRule,
                format!("invalid day-of-month {s:?}"),
            )
        })
}

/// Case-insensitive prefix strip (returns the suffix if `s` begins with `prefix`).
fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    // Compare on bytes, never `s[..prefix.len()]`: byte-length guards alone do not guarantee a
    // char boundary, so a `prefix.len()` falling mid-UTF-8 (e.g. inside `\u{064c}`) would panic.
    // When the first `prefix.len()` bytes match `prefix` ignoring ASCII case they are all ASCII
    // (they equal ASCII bytes), so `prefix.len()` *is* a char boundary and `&s[prefix.len()..]`
    // is panic-free; a mismatch (including any non-ASCII byte) returns `None`.
    let (sb, pb) = (s.as_bytes(), prefix.as_bytes());
    if sb.len() >= pb.len() && sb[..pb.len()].eq_ignore_ascii_case(pb) {
        Some(&s[pb.len()..])
    } else {
        None
    }
}

// ---- Zone -----------------------------------------------------------------------------

/// Parse a `Zone NAME ...era...` line. Returns the zone (with its first era) and whether a
/// continuation line is expected (i.e. the first era ended with `UNTIL`).
fn parse_zone(line: &Line, file: &Path) -> Result<(ZoneRecord, bool)> {
    let f = &line.fields;
    // `Zone` + NAME + at least STDOFF RULES FORMAT.
    if f.len() < 5 {
        return Err(diag(
            DiagnosticCode::InvalidFieldCount,
            format!("Zone line needs at least 5 fields, found {}", f.len()),
            file,
            line,
        ));
    }
    let name = f[1].text.clone();
    let era = parse_era(f, 2, file, line)?;
    let more = era.until.is_some();
    let zone = ZoneRecord {
        name,
        eras: vec![era],
        origin: Origin::new(file, line.number),
    };
    Ok((zone, more))
}

/// Parse one era from `fields[start..]`: `STDOFF RULES FORMAT [UNTIL...]`.
fn parse_era(
    fields: &[super::records::Field],
    start: usize,
    file: &Path,
    line: &Line,
) -> Result<ZoneEra> {
    let era = &fields[start..];
    if era.len() < 3 {
        return Err(diag(
            DiagnosticCode::InvalidFieldCount,
            "zone era needs STDOFF, RULES and FORMAT",
            file,
            line,
        ));
    }
    let stdoff = parse_offset(&era[0].text).map_err(|e| field_err(e, file, line, era[0].col))?;
    let rules =
        parse_rules_field(&era[1].text).map_err(|e| field_err(e, file, line, era[1].col))?;
    let format = era[2].text.clone();
    let until = if era.len() > 3 {
        Some(parse_until(&era[3..], file, line)?)
    } else {
        None
    };
    Ok(ZoneEra {
        stdoff,
        rules,
        format,
        until,
        origin: Origin::new(file, line.number),
    })
}

/// Parse the `RULES` column: `-`, an inline saving like `1:00`, or a named ruleset.
fn parse_rules_field(s: &str) -> std::result::Result<ZoneRules, (DiagnosticCode, String)> {
    if s == "-" {
        return Ok(ZoneRules::None);
    }
    // An inline amount looks like a time: starts with a digit or sign and has a digit.
    let looks_like_time = s
        .chars()
        .next()
        .map(|c| c.is_ascii_digit() || c == '-' || c == '+')
        .unwrap_or(false)
        && s.chars().any(|c| c.is_ascii_digit());
    if looks_like_time {
        return parse_save(s).map(ZoneRules::Save);
    }
    Ok(ZoneRules::Named(s.to_string()))
}

/// Parse an `UNTIL` field: `YEAR [MONTH [DAY [TIME]]]`. Omitted parts default to the
/// earliest possible value (`zic`: month → January, day → 1, time → 00:00 wall).
fn parse_until(fields: &[super::records::Field], file: &Path, line: &Line) -> Result<Until> {
    let year = fields[0].text.parse::<i32>().map_err(|_| {
        diag(
            DiagnosticCode::InvalidValue,
            "invalid UNTIL year",
            file,
            line,
        )
    })?;
    let month = match fields.get(1) {
        Some(m) => names::month(&m.text).map_err(|e| field_err(e, file, line, m.col))?,
        None => 1,
    };
    let day = match fields.get(2) {
        Some(d) => parse_on_day(&d.text).map_err(|e| field_err(e, file, line, d.col))?,
        None => OnDay::Day(1),
    };
    let time = match fields.get(3) {
        Some(t) => parse_time_of_day(&t.text).map_err(|e| field_err(e, file, line, t.col))?,
        None => crate::model::TimeOfDay::zero(),
    };
    Ok(Until {
        year,
        month,
        day,
        time,
    })
}

// ---- Link -----------------------------------------------------------------------------

/// Parse a `Link TARGET LINK-NAME` line.
fn parse_link(line: &Line, file: &Path) -> Result<LinkRecord> {
    let f = &line.fields;
    if f.len() != 3 {
        return Err(diag(
            DiagnosticCode::InvalidFieldCount,
            format!("Link line needs 3 fields, found {}", f.len()),
            file,
            line,
        ));
    }
    Ok(LinkRecord {
        target: f[1].text.clone(),
        link_name: f[2].text.clone(),
        origin: Origin::new(file, line.number),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::calendar::Weekday;
    use std::path::PathBuf;

    fn parse(s: &str) -> Result<Database> {
        let mut db = Database::default();
        parse_into(s.as_bytes(), &PathBuf::from("t.zi"), &mut db)?;
        Ok(db)
    }

    #[test]
    fn strip_prefix_ci_does_not_slice_through_multibyte_char() {
        // T23.cargo-fuzz.2 / F3 regression: `s[..prefix.len()]` guarded only by byte-length sliced
        // through a multibyte UTF-8 code point (the seed embedded `\u{064c}` = bytes d9 8c) and
        // panicked. Now it compares on bytes → a non-ASCII byte is a clean non-match, never a panic.
        assert_eq!(strip_prefix_ci("ma\u{064c}x", "max"), None);
        assert_eq!(strip_prefix_ci("\u{064c}max", "max"), None);
        // ASCII case-insensitive matching still works and returns the correct suffix.
        assert_eq!(strip_prefix_ci("MaXimum", "max"), Some("imum"));
        assert_eq!(strip_prefix_ci("min", "max"), None);
        assert_eq!(strip_prefix_ci("ma", "max"), None); // shorter than prefix
    }

    #[test]
    fn fixed_zone_and_link() {
        let db = parse("Zone Etc/UTC 0 - UTC\nLink Etc/UTC UTC\n").unwrap();
        assert_eq!(db.zones.len(), 1);
        assert_eq!(db.zones[0].name, "Etc/UTC");
        assert_eq!(db.zones[0].eras.len(), 1);
        assert_eq!(db.zones[0].eras[0].rules, ZoneRules::None);
        assert_eq!(db.links.len(), 1);
        assert_eq!(db.links[0].target, "Etc/UTC");
        assert_eq!(db.links[0].link_name, "UTC");
    }

    #[test]
    fn rule_round_trip() {
        let db = parse("Rule X 2020 only - Mar Sun>=8 2:00 1:00 D\n").unwrap();
        let r = &db.rules["X"][0];
        assert_eq!(r.from, 2020);
        assert_eq!(r.to, YearBound::Year(2020));
        assert_eq!(r.in_month, 3);
        assert_eq!(r.on, OnDay::OnAfter(Weekday::Sun, 8));
        assert_eq!(r.save.seconds, 3600);
        assert!(r.save.is_dst);
        assert_eq!(r.letter, "D");
    }

    #[test]
    fn multi_era_zone_with_until() {
        // Two eras: the first ends at an UNTIL, the second is open.
        let src = "Zone T/Z -5:00 - EST 1970\n\t\t-6:00 - CST\n";
        let db = parse(src).unwrap();
        assert_eq!(db.zones.len(), 1);
        assert_eq!(db.zones[0].eras.len(), 2);
        assert!(db.zones[0].eras[0].until.is_some());
        assert!(db.zones[0].eras[1].until.is_none());
    }

    #[test]
    fn wrong_field_count_is_diagnostic() {
        let e = parse("Link only-one-field\n").unwrap_err();
        assert_eq!(
            e.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::InvalidFieldCount)
        );
    }

    #[test]
    fn dangling_until_errors() {
        assert!(parse("Zone T/Z -5:00 - EST 1970\n").is_err());
    }

    /// Parse with the YEARISTYPE.1 legacy mode on (admits historical Rule `TYPE` predicates).
    fn parse_yit(s: &str) -> Result<Database> {
        let mut db = Database::default();
        parse_into_with(
            s.as_bytes(),
            &PathBuf::from("t.zi"),
            &mut db,
            LegacySource {
                yearistype: true,
                ..Default::default()
            },
        )?;
        Ok(db)
    }

    #[test]
    fn default_mode_rejects_year_typed_rule() {
        // The 5th (TYPE) field carries `even` → ZIC027 in the modern default (no silent ignore).
        let e = parse("Rule R 1990 1994 even Mar Sun>=18 2:00 0 -\n").unwrap_err();
        assert_eq!(
            e.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::UnsupportedRuleType)
        );
    }

    #[test]
    fn legacy_mode_admits_even_and_odd_year_types() {
        let db =
            parse_yit("Rule R 1990 1994 even Mar Sun>=18 2:00 0 -\n").expect("legacy admits even");
        assert_eq!(db.rules["R"][0].year_type, crate::model::YearType::Even);
        let db =
            parse_yit("Rule R 1990 1994 odd Mar Sun>=1 2:00 0 -\n").expect("legacy admits odd");
        assert_eq!(db.rules["R"][0].year_type, crate::model::YearType::Odd);
    }

    #[test]
    fn legacy_mode_rejects_wild_year_type() {
        // An unknown ("wild") TYPE is a hard error even in legacy mode — the historical
        // yearistype.sh itself exits 1 on a wild type.
        let e = parse_yit("Rule R 1990 1994 leapday Mar Sun>=18 2:00 0 -\n").unwrap_err();
        assert_eq!(
            e.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::UnsupportedRuleType)
        );
    }

    #[test]
    fn legacy_mode_rejects_typed_single_year() {
        // A year-type on a single-year rule is meaningless (mirrors zic's "typed single year").
        let e = parse_yit("Rule R 1990 only even Mar Sun>=18 2:00 0 -\n").unwrap_err();
        assert_eq!(
            e.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::UnsupportedRuleType)
        );
    }

    #[test]
    fn record_keyword_matches_zic_style_unambiguous_prefixes() {
        // Full words and any unambiguous prefix, case-insensitively.
        for w in ["Rule", "rule", "RULE", "Rul", "Ru", "R", "r"] {
            assert_eq!(record_keyword(w), Some(RecordKind::Rule), "{w:?}");
        }
        for w in ["Zone", "zone", "Zon", "Zo", "Z", "z"] {
            assert_eq!(record_keyword(w), Some(RecordKind::Zone), "{w:?}");
        }
        for w in ["Link", "link", "Lin", "Li", "L", "l"] {
            assert_eq!(record_keyword(w), Some(RecordKind::Link), "{w:?}");
        }
        // `Leap`/`Expires` are NOT in the zone-file table (verified against reference `zic`):
        // a zone file's `Leap` is "input line of unknown type". So they map to None here.
        for w in [
            "", "Leap", "Le", "Expires", "Ex", "X", "Rules", "Zonely", "q",
        ] {
            assert_eq!(record_keyword(w), None, "{w:?}");
        }
    }

    #[test]
    fn parses_zishrink_abbreviated_record_keys() {
        // The form used by the installed single-file `tzdata.zi`: `R`/`Z`/`L`.
        let db = parse("R X 2020 o - Mar Sun>=8 2:00 1:00 D\nZ Etc/UTC 0 - UTC\nL Etc/UTC UTC\n")
            .unwrap();
        assert_eq!(db.rules["X"][0].from, 2020);
        assert_eq!(db.zones.len(), 1);
        assert_eq!(db.zones[0].name, "Etc/UTC");
        assert_eq!(db.links[0].link_name, "UTC");
    }

    #[test]
    fn year_keyword_prefixes_match_zic_style() {
        // FROM keywords: any unambiguous prefix of minimum/maximum.
        for w in ["minimum", "minimu", "min", "mi"] {
            assert_eq!(parse_year(w), Ok(1900), "{w:?}"); // obsolete -> coerced to 1900
        }
        for w in ["maximum", "max", "ma"] {
            assert_eq!(parse_year(w), Ok(i32::MAX), "{w:?}");
        }
        assert_eq!(parse_year("1916"), Ok(1916));
        // TO keywords: only/minimum/maximum prefixes.
        for w in ["only", "onl", "on", "o"] {
            assert_eq!(parse_to_year(w, 1916), Ok(YearBound::Year(1916)), "{w:?}");
        }
        assert_eq!(parse_to_year("ma", 1916), Ok(YearBound::Max));
        assert_eq!(parse_to_year("mi", 1916), Ok(YearBound::Year(1900)));
    }

    #[test]
    fn bare_m_year_keyword_is_ambiguous() {
        // `m` could be minimum or maximum — reject, don't silently pick one.
        assert!(parse_year("m").is_err());
        assert!(parse_to_year("m", 2000).is_err());
    }

    #[test]
    fn from_minimum_is_lowered_to_1900() {
        // Obsolete `minimum` is coerced to a finite 1900, never an infinite-past sentinel.
        let db = parse("Rule X minimum max - Apr lastSun 2:00 1:00 D\n").unwrap();
        assert_eq!(db.rules["X"][0].from, 1900);
        assert_eq!(db.rules["X"][0].to, YearBound::Max);
        assert_ne!(db.rules["X"][0].from, i32::MIN);
    }

    #[test]
    fn leap_line_rejected_in_main_source_context() {
        // `Leap`/`Expires` are NOT part of the zone-file keyword table (reference `zic` reports
        // "input line of unknown type"). zic-rs matches that class precisely (T13.2): a `Leap` line in
        // command position is `UnknownLineType`, not the (recognised-but-unsupported) `UnsupportedDirective`.
        let e = parse("Leap 2016 Dec 31 23:59:60 + S\n").unwrap_err();
        assert_eq!(
            e.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::UnknownLineType)
        );
    }
}
