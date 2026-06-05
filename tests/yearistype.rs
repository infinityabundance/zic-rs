//! YEARISTYPE.1 — historical Rule `TYPE` / `-y` (yearistype) replay support.
//!
//! The `even`/`odd`/`uspres`/`nonpres` predicates are anchored to `yearistype.sh` v7.4 (the script
//! reference `zic` shelled out to before tzcode 2020a removed `-y`) and verified byte-identical to an
//! old `zic` oracle on `Australia/Adelaide` 1990–1994 (`reports/yearistype/`). These tests pin the
//! behaviour at the unit + compile level: the predicate arithmetic, the parser's mode gate, and the
//! per-year firing that the compile path produces under `--legacy-yearistype`.

use std::path::PathBuf;

use tzcompile::model::calendar::civil_from_days;
use tzcompile::model::YearType;
use tzcompile::source::LegacySource;
use tzcompile::{compile_zone, load_database, load_database_with};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Local-wall `(year, month, day)` of a transition: the UT instant shifted by the offset of the type
/// that takes effect at it (so a fall-back to 02:00 standard reads as that calendar day).
fn local_ymd(at: i64, utoff: i32) -> (i32, u8, u8) {
    civil_from_days((at + utoff as i64).div_euclid(86_400))
}

// ----- unit: the predicate, anchored to yearistype.sh v7.4 -----

#[test]
fn yeartype_predicate_matches_yearistype_sh_v74() {
    // even: decimal year ends in {0,2,4,6,8}; odd: {1,3,5,7,9}.
    assert!(YearType::Even.includes(1990) && YearType::Even.includes(1992));
    assert!(!YearType::Even.includes(1991));
    assert!(YearType::Odd.includes(1991) && YearType::Odd.includes(1993));
    assert!(!YearType::Odd.includes(1990));
    // uspres: divisible by 4 (US presidential election years); nonpres: the complement.
    assert!(YearType::Uspres.includes(1992) && YearType::Uspres.includes(2000));
    assert!(!YearType::Uspres.includes(1990));
    assert!(YearType::Nonpres.includes(1990) && !YearType::Nonpres.includes(1992));
    // All applies to every year (the modern `-`).
    assert!(YearType::All.includes(1990) && YearType::All.includes(1991));
    // Round-trip the source tokens.
    for (tok, yt) in [
        ("-", YearType::All),
        ("even", YearType::Even),
        ("odd", YearType::Odd),
        ("uspres", YearType::Uspres),
        ("nonpres", YearType::Nonpres),
    ] {
        assert_eq!(YearType::from_field(tok), Some(yt));
        assert_eq!(yt.as_str(), tok);
    }
    assert_eq!(YearType::from_field("wild"), None);
}

// ----- the mode gate: default rejects, legacy accepts -----

#[test]
fn default_mode_rejects_year_typed_rule_with_zic027() {
    let err = load_database(&[fixture("fixtures/minimal/yearistype.zi")])
        .expect_err("modern default must refuse a non-`-` Rule TYPE");
    let msg = err.to_string();
    assert!(
        msg.contains("ZIC027") && msg.contains("even"),
        "expected ZIC027 on the `even` TYPE, got: {msg}"
    );
}

#[test]
fn legacy_yearistype_mode_admits_the_year_typed_rule() {
    let db = load_database_with(
        &[fixture("fixtures/minimal/yearistype.zi")],
        LegacySource {
            yearistype: true,
            ..Default::default()
        },
    )
    .expect("legacy mode admits even/odd");
    compile_zone(&db, "Test/YearIsType").expect("compiles under legacy mode");
}

// ----- the payoff: even/odd fire in the right years -----

#[test]
fn even_and_odd_rules_fire_by_year_parity() {
    let db = load_database_with(
        &[fixture("fixtures/minimal/yearistype.zi")],
        LegacySource {
            yearistype: true,
            ..Default::default()
        },
    )
    .expect("load legacy");
    let tz = compile_zone(&db, "Test/YearIsType").expect("compile");

    // Collect the March DST-end transitions (those landing on a standard, non-DST type) by year.
    let mut march_end: std::collections::BTreeMap<i32, u8> = std::collections::BTreeMap::new();
    for t in &tz.transitions {
        let ty = &tz.types[t.type_index as usize];
        if ty.is_dst {
            continue;
        }
        let (y, m, d) = local_ymd(t.at, ty.utoff);
        if m == 3 && (1990..=1994).contains(&y) {
            march_end.insert(y, d);
        }
    }

    // ODD years use `Sun>=1` (first Sunday of March); EVEN years use `Sun>=18`.
    //   1990 even → Mar 18 · 1991 odd → Mar 3 · 1992 even → Mar 22 · 1993 odd → Mar 7 · 1994 even → Mar 20
    // (matches the old-`zic` oracle on Australia/Adelaide; see reports/yearistype/).
    assert_eq!(march_end.get(&1990), Some(&18), "even 1990 → Sun>=18");
    assert_eq!(march_end.get(&1991), Some(&3), "odd 1991 → Sun>=1");
    assert_eq!(march_end.get(&1992), Some(&22), "even 1992 → Sun>=18");
    assert_eq!(march_end.get(&1993), Some(&7), "odd 1993 → Sun>=1");
    assert_eq!(march_end.get(&1994), Some(&20), "even 1994 → Sun>=18");
}
