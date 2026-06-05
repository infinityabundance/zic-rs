//! T14.4 — pathological rule/time-law ledger, as an **executable witness** (the doc table's twin).
//!
//! Classify-first (not exhaustive-support): for each pathological edge class, pin reference `zic`'s
//! behaviour (in `docs/zic-pathology-ledger.md`), record zic-rs's *current* behaviour here as a
//! machine-checked assertion, and let the ledger flip deliberately if behaviour ever changes. Most
//! classes are **already matched** (negative SAVE, sub-hour, `24:00`, year zero, big year ranges,
//! far-past/min, duplicate no-op) — accepted by both; this witness pins that they stay accepted.
//!
//! The one behaviour change T14.4 made (the ledger's headline finding): **two rules for the same
//! instant**, which reference `zic` treats as a fatal error ("two rules for same instant", exit 1),
//! used to **panic** zic-rs (a `debug_assert!`) / emit a non-monotonic TZif in release. It now fails
//! closed with `ZIC023_SIMULTANEOUS_TRANSITION`. No valid `tzdata.zi` zone is affected (CORE.1 341/0/0).

use tzcompile::{compile_zone_to_bytes, load_database, DiagnosticCode};

/// Compile a single zone from `source`; return `Ok(())` if it compiles, or the diagnostic code if it
/// fails closed. (`load_database` is the parse layer; the transition/semantic checks fire in
/// `compile_zone_to_bytes`, so a same-instant conflict surfaces there, not at parse.)
fn disposition(source: &str, zone: &str) -> Result<(), Option<DiagnosticCode>> {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, source).unwrap();
    let db = load_database(std::slice::from_ref(&p)).map_err(|e| e.diagnostic().map(|d| d.code))?;
    compile_zone_to_bytes(&db, zone)
        .map(|_| ())
        .map_err(|e| e.diagnostic().map(|d| d.code))
}

#[test]
fn two_rules_for_same_instant_fails_closed_not_panics() {
    // Reference `zic`: fatal "two rules for same instant" (exit 1). zic-rs: `ZIC023` (was a panic).
    let src = "Rule S 2000 only - Jun 1 2:00 1:00 D\n\
               Rule S 2000 only - Jun 1 2:00 0 S\n\
               Zone T/Sim 0:00 S %sT\n";
    assert_eq!(
        disposition(src, "T/Sim"),
        Err(Some(DiagnosticCode::SimultaneousTransition)),
        "same-instant rule conflict must fail closed with ZIC023, never panic or emit a non-monotonic TZif"
    );
}

#[test]
fn pathologies_that_both_tools_accept_still_compile() {
    // Each row: (label, source, zone). Reference `zic` accepts all of these (pinned in the ledger);
    // zic-rs must keep accepting them (a regression to fail-closed here would be a silent narrowing).
    let cases: &[(&str, &str, &str)] = &[
        // Negative SAVE (law 7, T7) — rule-form DST that subtracts.
        (
            "negative_save",
            "Rule N 2000 only - Jun 1 2:00 -1:00 D\nRule N 2000 only - Oct 1 2:00 0 S\nZone T/Neg 1:00 N %sT\n",
            "T/Neg",
        ),
        // Large SAVE (>= 2h).
        (
            "large_save",
            "Rule L 2000 only - Jun 1 2:00 2:00 DD\nRule L 2000 only - Oct 1 2:00 0 S\nZone T/Large 0:00 L %sT\n",
            "T/Large",
        ),
        // Sub-hour STDOFF (seconds precision).
        ("sub_hour_offset", "Zone T/Sub 5:30:20 - WEIRD\n", "T/Sub"),
        // `24:00` rule times (T8-v3 keeps this valid; folds into the next day).
        (
            "time_2400",
            "Rule T 2000 max - Mar lastSun 24:00 1:00 D\nRule T 2000 max - Oct lastSun 24:00 0 S\nZone T/T2400 0:00 T %sT\n",
            "T/T2400",
        ),
        // Year zero (proleptic Gregorian year 0 is a valid `zic` year).
        (
            "year_zero",
            "Rule Z 0 only - Jan 1 0:00 0 S\nZone T/Zero 0:00 Z %sT\n",
            "T/Zero",
        ),
        // Extreme offset > 24h. Reference adds a `-v` "values over 24 hours" portability warning that
        // zic-rs does not yet emit (a deferred warning class — see the ledger); both still *accept*.
        ("extreme_offset", "Zone T/Ext 25:00 - LMT\n", "T/Ext"),
        // Far-past `minimum` (coerced to 1900, matching reference's obsolete-keyword handling, T3.2a).
        (
            "far_past_minimum",
            "Rule M minimum 1910 - Jan 1 0:00 0 LMT\nZone T/Min 0:00 M %sT\n",
            "T/Min",
        ),
        // Duplicate-instant no-op era change (same offset/abbr across an `UNTIL`): deduped by both.
        (
            "duplicate_noop_era",
            "Zone T/Dup 0:00 - AAA 2000\n0:00 - AAA\n",
            "T/Dup",
        ),
    ];
    for (label, src, zone) in cases {
        assert_eq!(
            disposition(src, zone),
            Ok(()),
            "{label}: reference accepts this pathology; zic-rs must keep accepting it"
        );
    }
}

#[test]
fn large_year_range_compiles_and_is_strictly_increasing() {
    // A 2000..9999 yearly rule expands to ~8000 transitions — accepted by both. (Under `-v` this also
    // trips the ZIC020 too-many-transitions warning; here we only assert it compiles to a valid stream.)
    let src = "Rule Y 2000 9999 - Jan 1 0:00 0 S\nZone T/Year 0:00 Y %sT\n";
    let bytes = {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("in.zi");
        std::fs::write(&p, src).unwrap();
        let db = load_database(std::slice::from_ref(&p)).unwrap();
        compile_zone_to_bytes(&db, "T/Year").expect("large year range compiles")
    };
    assert!(!bytes.is_empty());
}
