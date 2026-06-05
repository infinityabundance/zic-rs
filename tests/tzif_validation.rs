//! T15.4 — RFC 9636 TZif structural validator (integration).
//!
//! The validator's headline guard: it validates **reference `zic`'s** output too, not only zic-rs's —
//! so it must respect the real producer profile, not just our writer's assumptions. The reference path
//! auto-skips when `zic` is unavailable; the zic-rs path and the malformed-rejection path are always
//! asserted. Structural validity is a *separate axis* from semantic witnesses (T15.3).

use std::path::PathBuf;

use tzcompile::compare::reference_zic;
use tzcompile::load_database;
use tzcompile::tzif::rfc9636::{build_validation_report, validate, TzifStructuralVerdict};

fn write_src(dir: &std::path::Path, body: &str) -> Vec<PathBuf> {
    let p = dir.join("in.zi");
    std::fs::write(&p, body).unwrap();
    vec![p]
}

// A DST zone (transitions + footer) + a fixed zone — non-trivial structural content.
const SRC: &str = "Rule D 2020 max - Mar Sun>=8 2:00 1:00 D\n\
                   Rule D 2020 max - Nov Sun>=1 2:00 0 S\n\
                   Zone T/DST -5:00 D E%sT\n\
                   Zone Etc/UTC 0:00 - UTC\n";

#[test]
fn zic_rs_output_is_structurally_conformant() {
    let dir = tempfile::tempdir().unwrap();
    let inputs = write_src(dir.path(), SRC);
    let db = load_database(&inputs).unwrap();
    let report = build_validation_report(
        &db,
        &["T/DST".to_string(), "Etc/UTC".to_string()],
        "zic-rs-no-such-zic", // force reference unavailable so this test is deterministic
        &inputs,
        dir.path(),
    )
    .unwrap();
    assert!(
        !report.reference_validated,
        "reference forced unavailable here"
    );
    let zr: Vec<_> = report
        .rows
        .iter()
        .filter(|r| r.producer == "zic_rs")
        .collect();
    assert_eq!(zr.len(), 2);
    assert!(zr
        .iter()
        .all(|r| r.validation.structural == TzifStructuralVerdict::Conformant));
}

#[test]
fn reference_zic_output_also_passes_the_validator() {
    // The key guard: a validator that only accepts zic-rs output could be validating its own
    // assumptions. Reference `zic` output must pass too. Auto-skip when `zic` is unavailable.
    if !reference_zic::is_available("zic") {
        eprintln!("note: reference `zic` not on PATH — skipping reference-output validation");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let inputs = write_src(dir.path(), SRC);
    let db = load_database(&inputs).unwrap();
    let report =
        build_validation_report(&db, &["T/DST".to_string()], "zic", &inputs, dir.path()).unwrap();
    assert!(report.reference_validated);
    let refs: Vec<_> = report
        .rows
        .iter()
        .filter(|r| r.producer == "reference_zic")
        .collect();
    assert!(
        !refs.is_empty(),
        "reference output should have been validated"
    );
    assert!(
        refs.iter()
            .all(|r| r.validation.structural == TzifStructuralVerdict::Conformant),
        "reference `zic` output must satisfy the validator (else it is too strict/wrong)"
    );
    assert!(report
        .to_json()
        .contains("\"schema\": \"zic-rs-tzif-validation-v1\""));
}

#[test]
fn malformed_input_is_a_violation_never_a_panic() {
    // Integration-level safety: arbitrary hostile bytes yield a Violation verdict, not a crash.
    for bad in [
        &b""[..],
        &b"TZif"[..],
        &b"TZif2\x00\x00"[..],
        &b"not a tzif file at all, just random bytes................"[..],
    ] {
        assert_eq!(validate(bad).structural, TzifStructuralVerdict::Violation);
    }
}

#[test]
fn structural_validation_is_not_a_semantic_report() {
    // Layer separation: the structural report is its own schema + artifact category, distinct from the
    // semantic-witness report. (Same zone, different surface, different claim.)
    let dir = tempfile::tempdir().unwrap();
    let inputs = write_src(dir.path(), "Zone Etc/UTC 0:00 - UTC\n");
    let db = load_database(&inputs).unwrap();
    let j = build_validation_report(
        &db,
        &["Etc/UTC".to_string()],
        "zic-rs-none",
        &inputs,
        dir.path(),
    )
    .unwrap()
    .to_json();
    assert!(j.contains("\"schema\": \"zic-rs-tzif-validation-v1\""));
    assert!(j.contains("structural_validation_artifact"));
    assert!(j.contains("NOT semantic behaviour"));
    // It must NOT masquerade as the semantic-witness surface.
    assert!(!j.contains("zic-rs-semantic-report-v1"));
}
