//! The semantic compiler: a parsed [`Database`] zone → in-memory [`TzifData`].
//!
//! This is where source records become the offsets/transitions/abbreviations a TZif file
//! encodes. It never touches the filesystem and never shells out — it is pure data → data.
//!
//! ## Supported shapes and the fail-closed contract
//!
//! A **single-era** zone is dispatched on its `RULES` field:
//!
//! * `RULES = -` → a **fixed-offset** zone with a literal `FORMAT` (any constant standard
//!   offset, not only zero);
//! * `RULES = <name>` → a rule-driven zone, compiled by [`transitions`] with real DST
//!   transitions and `%s`/slash/`%z` abbreviations — both **finite** rule sets (fixed footer)
//!   and **recurring** `TO = maximum` rule sets (recurring POSIX footer).
//!
//! A **multi-era** zone (`Zone` line with `UNTIL` continuations) is compiled by
//! [`transitions::compile_multi_era`], which carries state across era boundaries (the `UNTIL`
//! is interpreted in the ending era's context with the prevailing save; the footer comes from
//! the final era). A final era is classified by its **effective in-era** rule activations, so
//! a rule set that is "mixed" only because it contains finite rows ending before the era starts
//! is treated as recurring-only — this is what admits real zones such as `Europe/London`.
//!
//! An **inline-saving** era (`RULES` = a clock value like `1:00`) compiles to a single fixed
//! type at `STDOFF + SAVE` with `is_dst` set, abbreviation from a literal or `%z` `FORMAT`.
//!
//! A **mixed finite+recurring** final era is handled in both shapes — effectively
//! recurring-only (finite rows predate the era → anchor + footer) and genuinely mixed-in-era
//! (finite history expanded explicitly + recurring footer tail).
//!
//! A **no-rules** era (`RULES = -`) may use a literal `FORMAT` *or* `%z` (which renders the
//! era's numeric standard offset, e.g. `0:30 - %z` → `+0030`, as reference `zic` does — this is
//! how ~half of all zones describe a fixed offset).
//!
//! Everything else is *refused with an explicit diagnostic*, never approximated: recurring
//! rules whose day form is not POSIX-expressible, inline-save with a `%s`/slash `FORMAT`, and
//! `%s`/slash `FORMAT`s on a no-rules era (there is no LETTER / DST context to resolve them). A
//! *negative* inline `SAVE` is **supported** (signed effective offset; law 7 — `Europe/Prague`).
//! This is the project's core safety property — a construct we cannot compile
//! *correctly* produces an error, not a plausible-looking but wrong TZif file. The authoritative,
//! always-current lists live in `docs/supported-syntax.md` / `docs/unsupported-syntax.md`.

pub mod abbreviations;
pub mod leap;
pub mod plan;
pub mod posix_footer;
pub mod transitions;

pub use leap::apply_leaps;

use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::error::{Error, Result};
use crate::model::{Database, Save, ZoneEra, ZoneRecord, ZoneRules};
use crate::tzif::{LocalTimeType, TzifData};

/// Compile a single zone (by exact name) into TZif data, default emission style.
pub fn compile_zone(db: &Database, name: &str) -> Result<TzifData> {
    compile_zone_styled(db, name, crate::EmitOptions::default())
}

/// Compile a single zone with explicit [`crate::EmitOptions`] (style + `-R` redundant-tail bound).
/// The options reach only the two rule-driven paths
/// ([`transitions::compile_rule_zone`]/[`transitions::compile_multi_era`]) that can have a
/// footer-governed recurring tail; fixed-offset and inline-save eras emit a single constant type with
/// no tail, so they are irrelevant. With [`crate::EmitStyle::Default`] the output is byte-for-byte
/// what it was before T8-slim (CORE.1-gated).
pub fn compile_zone_styled(
    db: &Database,
    name: &str,
    opts: crate::EmitOptions,
) -> Result<TzifData> {
    let zone = db
        .zone(name)
        .ok_or_else(|| Error::message(format!("no zone named {name:?} in the input")))?;

    // Build the full semantic transition stream first (with any `-b`/`-R` emission shaping). Range
    // truncation (`-r`) is then a distinct **post-emission** pass (T10.4d) — never interleaved with
    // the rule/era compilers.
    //
    // Dispatch on shape:
    //   multi-era      → cross-era stitching (the era walker)
    //   `-`            → fixed offset                              (T1)
    //   named rule set → transition compiler (finite + recurring)  (T2/T3)
    //   inline saving  → one fixed type at STDOFF + SAVE           (T3.2b)
    // The rule-driven paths (`compile_rule_zone` / `compile_multi_era`) apply `-r` **internally**,
    // before their slim pass (correct ordering). The footer-less fixed/inline paths have no slim
    // interaction, so `-r` is applied to their finished `TzifData` here.
    if zone.eras.len() != 1 {
        return transitions::compile_multi_era(zone, db, opts);
    }
    let era = &zone.eras[0];
    let data = match &era.rules {
        ZoneRules::None => {
            let mut d = compile_fixed_offset(zone, era)?;
            if let Some(range) = opts.range {
                transitions::apply_range(&mut d, range);
            }
            d
        }
        ZoneRules::Named(rule_name) => {
            let rules = db.rules.get(rule_name).ok_or_else(|| {
                unsupported(
                    zone,
                    format!("zone references unknown rule set {rule_name:?}"),
                )
            })?;
            transitions::compile_rule_zone(zone, era, rules, opts)?
        }
        ZoneRules::Save(save) => {
            let mut d = compile_inline_save(zone, era, *save)?;
            if let Some(range) = opts.range {
                transitions::apply_range(&mut d, range);
            }
            d
        }
    };
    Ok(data)
}

/// Compile a single-era zone whose `RULES` is an inline constant saving (e.g. `1:00`). The era is
/// one fixed local-time type at `STDOFF + SAVE`, with `is_dst = save.is_dst` (the parsed `Save`
/// already encodes the `s`/`d` suffix or the no-suffix `seconds != 0` heuristic), and an
/// abbreviation rendered from `FORMAT` — literal text or `%z` over the **total** offset. `SAVE` is
/// **signed**: a *negative* inline save is valid (the effective offset is `STDOFF + SAVE`, which can
/// be ≤ the standard offset — e.g. Europe/Prague's `1 -1 GMT` → `GMT`, `isdst=1`, gmtoff 0; pinned
/// against reference `zic`). Only `%s` (no LETTER) and the `STD/DST` slash form fail closed. See
/// `docs/reference-zic-semantics.md` §9 and `docs/zic-deep-semantics.md` law 7.
fn compile_inline_save(zone: &ZoneRecord, era: &ZoneEra, save: Save) -> Result<TzifData> {
    let fmt = &era.format;
    if fmt.contains("%s") || fmt.contains('/') {
        return Err(unsupported(
            zone,
            format!("inline-save FORMAT {fmt:?}: only literal and %z are supported yet (no %s / STD/DST slash)"),
        ));
    }
    let utoff = era.stdoff.0 + save.seconds;
    let abbr = abbreviations::render(fmt, "", save.is_dst, utoff);
    let footer = posix_footer::fixed_offset(&abbr, utoff);
    Ok(TzifData {
        types: vec![LocalTimeType {
            utoff,
            is_dst: save.is_dst,
            abbr,
        }],
        transitions: Vec::new(),
        footer,
        version: b'2',
        leaps: Vec::new(),
    })
}

/// Compile a fixed-offset, no-rules era (`RULES` = `-`). Dispatched here only for that case.
fn compile_fixed_offset(zone: &ZoneRecord, era: &ZoneEra) -> Result<TzifData> {
    // A no-rules era has no LETTER and no DST context, so `%s` (LETTER substitution) and the
    // `STD/DST` slash form are meaningless here and are refused. `%z` IS allowed: it renders the
    // era's numeric standard offset (e.g. `0:30 - %z` → `+0030`), exactly as reference `zic` does
    // — this is the common shape across the database (≈ half of all zones use a `%z` no-rules era
    // somewhere), so refusing it would be a large, needless gap.
    let fmt = &era.format;
    if fmt.contains("%s") || fmt.contains('/') {
        return Err(unsupported(
            zone,
            format!(
                "FORMAT {fmt:?}: %s / STD-DST slash need rule context, but this era has no rules"
            ),
        ));
    }

    let utoff = era.stdoff.0;
    // `render` substitutes `%z` with the numeric offset and passes a literal through unchanged
    // (there is no LETTER/DST here, hence the empty letter and `false`).
    let abbr = abbreviations::render(fmt, "", false, utoff);
    let footer = posix_footer::fixed_offset(&abbr, utoff);
    Ok(TzifData::fixed(utoff, abbr, footer))
}

/// Build an `UnsupportedDirective` error located at the zone's source line.
fn unsupported(zone: &ZoneRecord, msg: impl Into<String>) -> Error {
    Error::from(Diagnostic::error(
        DiagnosticCode::UnsupportedDirective,
        msg,
        &zone.origin.file,
        zone.origin.line,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn db(src: &str) -> Database {
        let mut db = Database::default();
        crate::source::parse_into(src.as_bytes(), &PathBuf::from("t.zi"), &mut db).unwrap();
        db
    }

    #[test]
    fn compiles_utc() {
        let d = compile_zone(&db("Zone Etc/UTC 0 - UTC\n"), "Etc/UTC").unwrap();
        assert_eq!(d.types.len(), 1);
        assert_eq!(d.types[0].utoff, 0);
        assert_eq!(d.types[0].abbr, "UTC");
        assert_eq!(d.footer, "UTC0");
        assert!(d.transitions.is_empty());
    }

    #[test]
    fn compiles_fixed_offset() {
        let d = compile_zone(&db("Zone Test/Fixed -5:00 - EST\n"), "Test/Fixed").unwrap();
        assert_eq!(d.types[0].utoff, -18000);
        assert_eq!(d.footer, "EST5");
    }

    #[test]
    fn finite_rule_zone_compiles() {
        // T2: a finite ("only") DST rule set compiles to real transitions.
        let src = "Rule X 2020 only - Mar Sun>=8 2:00 1:00 D\n\
                   Rule X 2020 only - Nov Sun>=1 2:00 0 S\n\
                   Zone Test/Simple -5:00 X E%sT\n";
        let d = compile_zone(&db(src), "Test/Simple").unwrap();
        assert_eq!(d.transitions.len(), 2);
        assert_eq!(d.footer, "EST5");
        // Spring forward to EDT, fall back to EST.
        let spring = &d.types[d.transitions[0].type_index as usize];
        let fall = &d.types[d.transitions[1].type_index as usize];
        assert_eq!(
            (spring.utoff, spring.is_dst, spring.abbr.as_str()),
            (-14400, true, "EDT")
        );
        assert_eq!(
            (fall.utoff, fall.is_dst, fall.abbr.as_str()),
            (-18000, false, "EST")
        );
        // Exact UT instants (Mar 8 2020 07:00Z, Nov 1 2020 06:00Z) — matches reference zic.
        assert_eq!(d.transitions[0].at, 1583650800);
        assert_eq!(d.transitions[1].at, 1604210400);
    }

    #[test]
    fn recurring_rule_zone_compiles_with_posix_footer() {
        // T3: a `max` (recurring) US-Eastern-style rule set compiles, with the recurring
        // POSIX footer describing the infinite tail.
        let src = "Rule US 2007 max - Mar Sun>=8 2:00 1:00 D\n\
                   Rule US 2007 max - Nov Sun>=1 2:00 0 S\n\
                   Zone Test/Eastern -5:00 US E%sT\n";
        let d = compile_zone(&db(src), "Test/Eastern").unwrap();
        assert_eq!(d.footer, "EST5EDT,M3.2.0,M11.1.0");
        assert!(
            d.transitions.len() > 2,
            "explicit transitions across the window"
        );
    }

    #[test]
    fn recurring_sun_leq_25_uses_extended_v3_footer() {
        // `Sun<=25` re-anchors onto the 3rd Wednesday + 4 days (Wed+4 = the Sunday ≤ 25), at
        // 02:00 + 96h = 98h → `M10.3.3/98`, needing the v3 footer extension — the same `zic`
        // mechanism that expresses Asia/Gaza's `Sat<=30` as `M3.4.4/50`. (law 10.)
        let src = "Rule W 2000 max - Mar Sun>=8 2:00 1:00 D\n\
                   Rule W 2000 max - Oct Sun<=25 2:00 0 S\n\
                   Zone Z -5:00 W E%sT\n";
        let d = compile_zone(&db(src), "Z").unwrap();
        assert_eq!(d.footer, "EST5EDT,M3.2.0,M10.3.3/98");
        assert_eq!(d.version, b'3');
    }

    #[test]
    fn percent_format_without_rules_fails_closed() {
        let e = compile_zone(&db("Zone Z -5:00 - E%sT\n"), "Z").unwrap_err();
        assert!(e.diagnostic().is_some());
    }
}
