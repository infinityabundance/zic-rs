//! Transition-compiler tests for finite DST rule sets (milestone T2).
//!
//! Two layers: (1) library-level assertions on exact transition instants/types/footer that
//! always run; (2) oracle comparisons against reference `zic` that auto-skip when no `zic`
//! is on `PATH`.

use std::path::PathBuf;

use tzcompile::compare::{compare_zone, reference_zic, CompareMode};
use tzcompile::{compile_zone, load_database};

const REFERENCE: &str = "zic";

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn test_simple_exact_transitions() {
    // The canonical T2 fixture: one spring-forward and one fall-back in 2020.
    let db = load_database(&[fixture("fixtures/minimal/dst.zi")]).unwrap();
    let d = compile_zone(&db, "Test/Simple").unwrap();

    assert_eq!(d.transitions.len(), 2, "one spring + one fall");
    // Mar 8 2020 07:00 UTC (02:00 EST → 03:00 EDT) and Nov 1 2020 06:00 UTC.
    assert_eq!(d.transitions[0].at, 1_583_650_800);
    assert_eq!(d.transitions[1].at, 1_604_210_400);

    let spring = &d.types[d.transitions[0].type_index as usize];
    let fall = &d.types[d.transitions[1].type_index as usize];
    assert_eq!((spring.utoff, spring.is_dst), (-14400, true));
    assert_eq!(spring.abbr, "EDT");
    assert_eq!((fall.utoff, fall.is_dst), (-18000, false));
    assert_eq!(fall.abbr, "EST");

    // Finite rules → permanent standard time afterwards → fixed footer.
    assert_eq!(d.footer, "EST5");
}

#[test]
fn ut_referenced_at_and_lastsun() {
    // EU-style: `lastSun` with a UT-referenced AT (`1:00u`). Verifies both the day form and
    // the universal-time conversion path. Last Sunday of March 2021 is the 28th; 01:00 UT.
    let db = load_database(&[fixture("fixtures/minimal/euro.zi")]).unwrap();
    let d = compile_zone(&db, "Test/Euro").unwrap();
    assert_eq!(d.transitions.len(), 2);
    // 2021-03-28 01:00:00 UTC = 1616893200 (UT-referenced, so no offset adjustment).
    assert_eq!(d.transitions[0].at, 1_616_893_200);
    let summer = &d.types[d.transitions[0].type_index as usize];
    assert_eq!((summer.utoff, summer.is_dst), (7200, true)); // CEST = UT+2
    assert_eq!(summer.abbr, "CEST");
}

#[test]
fn eastern_recurring_footer_string() {
    // T3: the recurring footer must be exactly zic's, including the omitted default DST
    // offset and the omitted /02:00 transition times.
    let db = load_database(&[fixture("fixtures/minimal/eastern.zi")]).unwrap();
    let d = compile_zone(&db, "Test/Eastern").unwrap();
    assert_eq!(d.footer, "EST5EDT,M3.2.0,M11.1.0");
}

/// Oracle: a fixture must match reference `zic` under the real **`zdump` behaviour** oracle
/// over the declared `[lo, hi]` horizon. Auto-skips when `zic`/`zdump` is absent.
fn oracle(source: &str, zone: &str, lo: i32, hi: i32) {
    if !reference_zic::is_available(REFERENCE) || !reference_zic::is_available("zdump") {
        eprintln!("skipping oracle: reference `zic`/`zdump` not on PATH");
        return;
    }
    let inputs = vec![fixture(source)];
    let db = load_database(&inputs).unwrap();
    let work = tempfile::tempdir().unwrap();
    let mode = CompareMode::Zdump {
        program: "zdump".to_string(),
        lo,
        hi,
    };
    let cmp = compare_zone(&db, &inputs, zone, REFERENCE, work.path(), &mode).unwrap();
    assert!(cmp.is_match(), "{}", cmp.summary());
}

// Finite fixtures: declared horizon 2019..2022.
#[test]
fn oracle_test_simple() {
    oracle("fixtures/minimal/dst.zi", "Test/Simple", 2019, 2022);
}

#[test]
fn oracle_euro_lastsun_ut() {
    oracle("fixtures/minimal/euro.zi", "Test/Euro", 2019, 2022);
}

#[test]
fn oracle_sle_sun_le_25_standard_at() {
    oracle("fixtures/minimal/sle.zi", "Test/Sle", 2019, 2022);
}

// Recurring fixture: wide horizon (2019..2099) so the POSIX footer governs the tail well
// beyond the explicit-transition window — proving the footer, not just the transitions.
#[test]
fn oracle_eastern_recurring() {
    oracle("fixtures/minimal/eastern.zi", "Test/Eastern", 2019, 2099);
}
