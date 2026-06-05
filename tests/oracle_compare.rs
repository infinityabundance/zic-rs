//! Oracle comparison against reference `zic`.
//!
//! This is the project's headline guarantee: our output means the same thing as the
//! canonical compiler's. The test **auto-skips** when no `zic` is on `PATH`, so the default
//! `cargo test` never *requires* a system `zic` (CI runs it in a job that installs tzcode).

use std::path::PathBuf;

use tzcompile::compare::{compare_zone, reference_zic, CompareMode};
use tzcompile::load_database;

const REFERENCE: &str = "zic";

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Fixed-offset fixtures are checked with the decoded-TZif (structural) comparison — exact
/// and appropriate here (these are also byte-identical). The `zdump` *behaviour* oracle is
/// exercised on the DST fixtures in `transition_generation.rs`.
fn compare(source: &str, zone: &str) {
    if !reference_zic::is_available(REFERENCE) {
        eprintln!("skipping oracle test: no reference `zic` on PATH");
        return;
    }
    let inputs = vec![fixture(source)];
    let db = load_database(&inputs).expect("load");
    let work = tempfile::tempdir().unwrap();
    let cmp = compare_zone(
        &db,
        &inputs,
        zone,
        REFERENCE,
        work.path(),
        &CompareMode::Structural,
    )
    .expect("compare");
    assert!(
        cmp.is_match(),
        "{} disagrees with reference zic:\n{}",
        zone,
        cmp.summary()
    );
}

#[test]
fn utc_matches_reference() {
    compare("fixtures/minimal/utc.zi", "Etc/UTC");
}

#[test]
fn fixed_offset_matches_reference() {
    compare("fixtures/minimal/fixed.zi", "Test/Fixed");
}
