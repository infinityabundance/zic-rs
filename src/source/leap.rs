//! Leap-source parsing (T11.2 — the **grammar wall**).
//!
//! A *leap-source* file (reference `zic`'s `-L` argument) uses a **different keyword table** from a
//! zone-source file: only `Leap` and `Expires` are recognised (`zic`'s `leap_line_codes`; a
//! `Rule`/`Zone`/`Link` here is "input line of unknown type"). Symmetrically, the ordinary
//! zone-source path ([`super::parse_into`]) never recognises `Leap`/`Expires`. This module is that
//! wall: it parses a leap-source into a [`LeapTable`] and nothing else.
//!
//! **Scope (T11.2): grammar only.** It does *not* compile leap data into TZif, do `right/` emission,
//! change the TZif version, interact with `-r`, or perform the `Rolling` local-wall conversion —
//! those are T11.3–T11.6. Leap entries are stored verbatim (UT-as-written) with the `rolling` flag.

use std::path::Path;

use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::error::{Error, Result};
use crate::model::calendar::{days_from_civil, days_in_month};
use crate::model::{LeapSecond, LeapTable};
use crate::source::lexer::tokenize;
use crate::source::names;
use crate::source::records::Line;

/// Parse an explicit **leap-source** file into a [`LeapTable`]. Recognises only `Leap`/`Expires`;
/// any other line (incl. `Rule`/`Zone`/`Link`) is rejected as an unknown line type — matching
/// reference `zic`'s leap-file keyword table.
pub fn parse_leap_source(bytes: &[u8], file: &Path) -> Result<LeapTable> {
    let lines = tokenize(bytes, file)?;
    let mut table = LeapTable::default();
    for line in &lines {
        match leap_keyword(line.keyword().unwrap_or("")) {
            Some(LeapKind::Leap) => {
                let entry = parse_leap_line(line, file)?;
                // Insert sorted by `trans`, as `zic`'s `leapadd` does.
                let pos = table
                    .entries
                    .iter()
                    .position(|e| entry.trans <= e.trans)
                    .unwrap_or(table.entries.len());
                table.entries.insert(pos, entry);
            }
            Some(LeapKind::Expires) => {
                if table.expires.is_some() {
                    return Err(err(
                        DiagnosticCode::InvalidValue,
                        "multiple Expires lines",
                        file,
                        line,
                    ));
                }
                table.expires = Some(parse_expires_line(line, file)?);
            }
            None => {
                return Err(err(
                    DiagnosticCode::UnsupportedDirective,
                    format!(
                        "input line of unknown type {:?} (a leap-source file accepts only \
                         Leap/Expires)",
                        line.keyword().unwrap_or("")
                    ),
                    file,
                    line,
                ));
            }
        }
    }
    // T17.1b: bound the leap table (a bucket-3 safer divergence; real tables hold ~27 entries, the
    // default cap is far above that). Also caps the O(n²) sorted-insert above on adversarial input.
    crate::limits::ResourceLimits::default().check_leap_count(table.entries.len())?;
    Ok(table)
}

#[derive(Clone, Copy)]
enum LeapKind {
    Leap,
    Expires,
}

/// The leap-file keyword table: `Leap` / `Expires`, matched by exact spelling or unambiguous prefix
/// (mirroring `zic`'s `byword`). The two words share no common prefix, so any non-empty prefix is
/// unambiguous.
fn leap_keyword(word: &str) -> Option<LeapKind> {
    if word.is_empty() {
        return None;
    }
    let lower = word.to_ascii_lowercase();
    let leap = "leap".starts_with(&lower);
    let expires = "expires".starts_with(&lower);
    match (leap, expires) {
        (true, false) => Some(LeapKind::Leap),
        (false, true) => Some(LeapKind::Expires),
        _ => None,
    }
}

/// `Leap  YEAR  MON  DAY  HH:MM:SS  CORR  ROLL` — 7 fields.
fn parse_leap_line(line: &Line, file: &Path) -> Result<LeapSecond> {
    let f = &line.fields;
    if f.len() != 7 {
        return Err(err(
            DiagnosticCode::InvalidFieldCount,
            format!("Leap line needs 7 fields, found {}", f.len()),
            file,
            line,
        ));
    }
    let trans = leap_datetime(&f[1].text, &f[2].text, &f[3].text, &f[4].text, file, line)?;
    let correction = match f[5].text.as_str() {
        "+" => 1,
        // Reference `zic` treats a bare `-` correction as `-1` (its `infile` blanks a lone `-`, so an
        // empty field also means `-1`).
        "-" | "" => -1,
        other => {
            return Err(err(
                DiagnosticCode::InvalidValue,
                format!("invalid CORRECTION field {other:?} on Leap line (want + or -)"),
                file,
                line,
            ));
        }
    };
    let rolling = match roll_kind(&f[6].text) {
        Some(r) => r,
        None => {
            return Err(err(
                DiagnosticCode::InvalidValue,
                format!(
                    "invalid Rolling/Stationary field {:?} on Leap line",
                    f[6].text
                ),
                file,
                line,
            ));
        }
    };
    Ok(LeapSecond {
        trans,
        correction,
        rolling,
    })
}

/// `Expires  YEAR  MON  DAY  HH:MM:SS` — 5 fields (no CORR/ROLL).
fn parse_expires_line(line: &Line, file: &Path) -> Result<i64> {
    let f = &line.fields;
    if f.len() != 5 {
        return Err(err(
            DiagnosticCode::InvalidFieldCount,
            format!("Expires line needs 5 fields, found {}", f.len()),
            file,
            line,
        ));
    }
    leap_datetime(&f[1].text, &f[2].text, &f[3].text, &f[4].text, file, line)
}

/// `Rolling`/`Stationary`, matched by unambiguous prefix (`zic`'s `leap_types` byword). No common
/// prefix, so any non-empty prefix is unambiguous.
fn roll_kind(word: &str) -> Option<bool> {
    let lower = word.to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }
    let rolling = "rolling".starts_with(&lower);
    let stationary = "stationary".starts_with(&lower);
    match (rolling, stationary) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}

/// Resolve a leap `YEAR MON DAY HH:MM:SS` to seconds since the 1970 epoch. Rejects a malformed field
/// and — like reference `zic` — an instant that **precedes the Epoch** (`t < 0`).
fn leap_datetime(
    year_s: &str,
    mon_s: &str,
    day_s: &str,
    time_s: &str,
    file: &Path,
    line: &Line,
) -> Result<i64> {
    let bad = |msg: String| err(DiagnosticCode::InvalidValue, msg, file, line);
    let year: i32 = year_s
        .parse()
        .map_err(|_| bad(format!("invalid leap year {year_s:?}")))?;
    let month = names::month(mon_s).map_err(|(_, m)| bad(m))?;
    let day: u8 = day_s
        .parse()
        .ok()
        .filter(|&d| d >= 1 && d <= days_in_month(year, month))
        .ok_or_else(|| bad(format!("invalid day of month {day_s:?}")))?;
    let tod = leap_seconds_of_day(time_s).map_err(bad)?;
    let t = days_from_civil(year, month, day) * 86_400 + tod;
    if t < 0 {
        return Err(bad("leap second precedes Epoch".to_string()));
    }
    Ok(t)
}

/// Parse a leap-line time-of-day to seconds. **Unlike ordinary `AT` times, a `60` seconds field is
/// allowed** — that is the leap second itself (`23:59:60` ≡ 86 400 s ≡ next-day midnight). No w/s/u
/// suffix. (Deliberately a *separate* parser from `model::time::parse_time_of_day`, which rejects
/// `:60` — leap times are a different grammar.)
fn leap_seconds_of_day(s: &str) -> std::result::Result<i64, String> {
    let mut parts = s.split(':');
    let parse = |p: Option<&str>| -> std::result::Result<i64, String> {
        match p {
            None => Ok(0),
            Some(x) if !x.is_empty() && x.bytes().all(|b| b.is_ascii_digit()) => x
                .parse::<i64>()
                .map_err(|_| format!("number {x:?} out of range")),
            Some(x) => Err(format!("invalid time component {x:?} in {s:?}")),
        }
    };
    let h = parse(parts.next())?;
    let m = parse(parts.next())?;
    let sec = parse(parts.next())?;
    if parts.next().is_some() {
        return Err(format!("too many ':' groups in time {s:?}"));
    }
    if m >= 60 || sec > 60 {
        return Err(format!("minutes/seconds out of range in {s:?}"));
    }
    Ok(h * 3600 + m * 60 + sec)
}

fn err(code: DiagnosticCode, msg: impl Into<String>, file: &Path, line: &Line) -> Error {
    Error::from(Diagnostic::error(code, msg, file, line.number))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::parse_into;
    use std::path::Path;

    fn leap(src: &str) -> Result<LeapTable> {
        parse_leap_source(src.as_bytes(), Path::new("<leap>"))
    }

    #[test]
    fn parses_stationary_and_rolling_leaps() {
        let t = leap("Leap 2016 Dec 31 23:59:60 + S\nLeap 2015 Jun 30 23:59:60 + R\n").unwrap();
        assert_eq!(t.entries.len(), 2);
        // Sorted by trans: 2015 before 2016.
        assert!(t.entries[0].trans < t.entries[1].trans);
        assert!(!t.entries[1].rolling, "S = Stationary");
        assert!(t.entries[0].rolling, "R = Rolling");
        assert_eq!(t.entries[0].correction, 1);
        // 23:59:60 normalises to next-day midnight.
        assert_eq!(t.entries[1].trans, days_from_civil(2017, 1, 1) * 86_400);
    }

    #[test]
    fn parses_expires() {
        let t = leap("Leap 2016 Dec 31 23:59:60 + S\nExpires 2025 Jan 1 0:00:00\n").unwrap();
        assert_eq!(t.expires, Some(days_from_civil(2025, 1, 1) * 86_400));
    }

    #[test]
    fn rejects_rule_zone_link_in_leap_source() {
        for kw in [
            "Rule US 2007 max - Mar Sun>=8 2:00 1:00 D",
            "Zone X 0 - X",
            "Link A B",
        ] {
            assert!(
                leap(&format!("{kw}\n")).is_err(),
                "{kw} must be rejected in a leap source"
            );
        }
    }

    #[test]
    fn rejects_multiple_expires() {
        let e = leap("Expires 2025 Jan 1 0:00:00\nExpires 2026 Jan 1 0:00:00\n");
        assert!(e.is_err());
    }

    #[test]
    fn rejects_malformed_correction_and_roll() {
        assert!(leap("Leap 2016 Dec 31 23:59:60 ? S\n").is_err(), "bad CORR");
        assert!(
            leap("Leap 2016 Dec 31 23:59:60 + Wobbly\n").is_err(),
            "bad ROLL"
        );
    }

    #[test]
    fn rejects_pre_epoch_leap() {
        // `1969 Jun 30` is genuinely before the 1970 epoch (t < 0). (Note `1969 Dec 31 23:59:60`
        // normalises to t = 0 = 1970-01-01 00:00:00, which reference `zic` *accepts* — the check is
        // `t < 0`, not `year < 1970`.)
        assert!(leap("Leap 1969 Jun 30 23:59:60 + S\n").is_err());
    }

    /// The wall holds the other way too: the ordinary zone-source parser still rejects `Leap`.
    #[test]
    fn zone_source_still_rejects_leap_and_expires() {
        let mut db = crate::model::Database::default();
        assert!(parse_into(
            b"Leap 2016 Dec 31 23:59:60 + S\n",
            Path::new("<z>"),
            &mut db
        )
        .is_err());
        assert!(parse_into(b"Expires 2025 Jan 1 0:00:00\n", Path::new("<z>"), &mut db).is_err());
    }
}
