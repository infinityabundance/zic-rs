//! Tests for `structural-report` — the TZif structural-parity inventory (campaign T9).
//!
//! These run the real reference `zic` (auto-skipping when it is absent), because a structural
//! comparison is meaningless without the other side. The fixed-offset fixtures are the only
//! zones zic-rs emits **byte-for-byte** identically to reference `zic`, so they pin the
//! `ByteIdentical` classification deterministically; a separate auto-skipping smoke test runs the
//! inventory over the installed `tzdata.zi` and asserts the honest invariants (no errors, every
//! zone classified, version+footer parity is the overwhelming majority).

use std::path::PathBuf;

use tzcompile::compare::reference_zic;
use tzcompile::load_database;
use tzcompile::structural::{build_structural_report, ParityClass};

const REFERENCE: &str = "zic";

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn fixed_offset_zone_is_byte_identical() {
    if !reference_zic::is_available(REFERENCE) {
        eprintln!("skipping structural-report test: reference `zic` not found");
        return;
    }
    let input = fixture("fixtures/minimal/utc.zi");
    let db = load_database(std::slice::from_ref(&input)).unwrap();
    let work = tempfile::tempdir().unwrap();
    let report = build_structural_report(
        &db,
        std::slice::from_ref(&input),
        REFERENCE,
        work.path(),
        Some("Etc/UTC"),
        None,
        tzcompile::EmitStyle::Default,
    )
    .unwrap();

    assert_eq!(report.zones_compared(), 1);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    // Etc/UTC is one of the pinned byte-identical fixtures (see fixtures/expected/).
    assert_eq!(report.zones[0].class, ParityClass::ByteIdentical);
    assert!(report.zones[0].diffs.is_empty());
    assert_eq!(report.version_footer_match(), 1);
    // The slim/fat delta is zero for a fixed-offset zone (no explicit transitions either side).
    assert_eq!(report.timecnt_delta_total, 0);
}

#[test]
fn report_text_and_json_render() {
    if !reference_zic::is_available(REFERENCE) {
        eprintln!("skipping structural-report render test: reference `zic` not found");
        return;
    }
    let input = fixture("fixtures/minimal/fixed.zi");
    let db = load_database(std::slice::from_ref(&input)).unwrap();
    let work = tempfile::tempdir().unwrap();
    let report = build_structural_report(
        &db,
        std::slice::from_ref(&input),
        REFERENCE,
        work.path(),
        Some("Test/Fixed"),
        Some("test-1".to_string()),
        tzcompile::EmitStyle::Default,
    )
    .unwrap();

    let text = report.to_text();
    assert!(text.contains("structural-parity inventory"));
    assert!(text.contains("SEPARATE from behaviour parity"));
    let json = report.to_json();
    assert!(json.contains("\"schema\": \"zic-rs-structural-report-v3\""));
    assert!(json.contains("\"class_counts\""));
    // T15.2 — structural-report's oracle is reference `zic` (visible, typed).
    assert!(json.contains("\"oracle_mode\": { \"mode\": \"reference_zic\""));
    assert!(json.contains("\"version_footer_match\""));
    // T12.6 — the static provenance/capability block is surfaced here too (gate lifted in T12.5a.2).
    assert!(json.contains("\"provenance\""));
    assert!(json.contains("\"source_variant_reference_pin_gate\": \"lifted_for_2026b\""));
    assert!(json.contains("\"source_variant_behavior_implemented\": false"));
}

/// Smoke test over the real installed single-file source. Auto-skips if either the file or the
/// reference `zic` is absent. Asserts the structural axis behaves honestly database-wide: every
/// canonical zone is compared with no errors, and the version+footer parity covers the vast
/// majority (the only known deviations are America/Santiago and Pacific/Easter — a subtle
/// `stringrule` v3 bump — documented in docs/structural-parity.md).
#[test]
fn tzdata_zi_structural_smoke() {
    let tzdata = PathBuf::from("/usr/share/zoneinfo/tzdata.zi");
    if !tzdata.exists() {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    }
    if !reference_zic::is_available(REFERENCE) {
        eprintln!("skipping: reference `zic` not found");
        return;
    }
    let db = load_database(std::slice::from_ref(&tzdata)).unwrap();
    let work = tempfile::tempdir().unwrap();
    let report = build_structural_report(
        &db,
        std::slice::from_ref(&tzdata),
        REFERENCE,
        work.path(),
        None,
        None,
        tzcompile::EmitStyle::Default,
    )
    .unwrap();

    assert!(report.zones_compared() > 300);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    // Every compared zone lands in exactly one class.
    let classified: usize = report.class_counts().values().sum();
    assert_eq!(classified, report.zones_compared());
    // version+footer parity should cover all but a tiny handful (documented outliers only).
    assert!(report.version_footer_match() >= report.zones_compared() - 5);
}
