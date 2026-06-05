//! T15.3 — semantic-witness mechanism tests (the public behaviour-evidence surface).
//!
//! The oracle-available path auto-skips when reference `zic`/`zdump` are not on PATH (oracle-availability
//! discipline); the **oracle-unavailable** path is deterministic and always asserted — that is the one
//! that proves absence is *visible* (`oracle_mode: unavailable` + `skipped_with_reason`), never silent.

use std::path::PathBuf;

use tzcompile::compare::{reference_zic, zdump};
use tzcompile::load_database;
use tzcompile::manifest::ArtifactCategory;
use tzcompile::semantic_witness::{build_semantic_witness_report, SemanticWitnessVerdict};

fn write_src(dir: &std::path::Path, body: &str) -> Vec<PathBuf> {
    let p = dir.join("in.zi");
    std::fs::write(&p, body).unwrap();
    vec![p]
}

const UTC: &str = "Zone Etc/UTC 0:00 - UTC\n";

#[test]
fn oracle_unavailable_renders_skipped_with_reason() {
    // Bogus oracle program names → unavailable. The report must say so, on every witness, with a reason.
    let dir = tempfile::tempdir().unwrap();
    let inputs = write_src(dir.path(), UTC);
    let db = load_database(&inputs).unwrap();
    let report = build_semantic_witness_report(
        &db,
        &["Etc/UTC".to_string()],
        "zic-rs-no-such-zic",
        "zic-rs-no-such-zdump",
        &inputs,
        dir.path(),
    )
    .unwrap();

    // oracle_mode = unavailable with a non-null reason; never silent.
    let j = report.to_json();
    assert!(j.contains("\"oracle_mode\": { \"mode\": \"unavailable\", \"skipped_with_reason\": "));
    assert!(!j.contains("\"skipped_with_reason\": null"));
    assert!(report
        .witnesses
        .iter()
        .all(|w| w.verdict == SemanticWitnessVerdict::SkippedOracleUnavailable));

    // T15.5-remainder — richer oracle identity is present even on the unavailable path (the *shape* is
    // always emitted; binary hashes/versions are null when the bogus tool can't be located). The
    // zoneinfo-resolution proof is constant: the oracle is pointed at the compiled file path, never a
    // zone name against system zoneinfo (the "did zdump read the intended tree?" hazard).
    for field in [
        "\"zic_binary_sha256\"",
        "\"zdump_binary_sha256\"",
        "\"zdump_command_line\"",
        "\"zoneinfo_resolution\": \"explicit_tzif_path_argument\"",
        "\"env_tz\"",
        "\"env_lc_all\"",
    ] {
        assert!(j.contains(field), "oracle_identity missing {field}:\n{j}");
    }
    // The bogus program names cannot be resolved on PATH → honest null binary hashes (never invented).
    assert!(j.contains("\"zic_binary_sha256\": null"));
}

#[test]
fn report_is_semantic_not_structural_and_every_witness_is_categorised() {
    // Even on the unavailable path the report carries its evidence category and the explicit non-claim
    // that a semantic witness is NOT RFC 9636 structural validity (the layer separation we insist
    // on). artifact_category appears on the report and on every witness row.
    let dir = tempfile::tempdir().unwrap();
    let inputs = write_src(dir.path(), UTC);
    let db = load_database(&inputs).unwrap();
    let report = build_semantic_witness_report(
        &db,
        &["Etc/UTC".to_string()],
        "zic-rs-no-such-zic",
        "zic-rs-no-such-zdump",
        &inputs,
        dir.path(),
    )
    .unwrap();
    let j = report.to_json();
    assert!(j.contains("\"schema\": \"zic-rs-semantic-report-v1\""));
    assert!(j.contains(&format!(
        "\"artifact_category\": \"{}\"",
        ArtifactCategory::SemanticWitnessArtifact.as_str()
    )));
    // Explicitly NOT a structural-validity claim.
    assert!(j.contains("NOT a claim of RFC 9636 structural validity"));
    assert!(!j.contains("structural_validation_artifact"));
    // Every witness row is categorised.
    let rows = j.matches("\"zone\":").count();
    let cats = j.matches("\"artifact_category\":").count();
    assert_eq!(
        cats,
        rows + 1,
        "report + every witness row carry a category"
    );
}

#[test]
fn oracle_available_produces_match_witnesses() {
    // Auto-skip when the oracle tools are absent (oracle-availability discipline).
    if !reference_zic::is_available("zic") || !zdump::is_available("zdump") {
        eprintln!(
            "note: reference zic/zdump not on PATH — skipping oracle-backed witness assertion"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let inputs = write_src(dir.path(), UTC);
    let db = load_database(&inputs).unwrap();
    let report = build_semantic_witness_report(
        &db,
        &["Etc/UTC".to_string()],
        "zic",
        "zdump",
        &inputs,
        dir.path(),
    )
    .unwrap();

    assert!(
        !report.witnesses.is_empty(),
        "expected witnesses with the oracle present"
    );
    // zic-rs's Etc/UTC is behaviour-identical to reference → all rows match.
    assert!(report
        .witnesses
        .iter()
        .all(|w| w.verdict == SemanticWitnessVerdict::Match));
    // And the oracle mode is the footer-aware zdump oracle, visibly.
    assert!(report
        .to_json()
        .contains("\"oracle_mode\": { \"mode\": \"reference_zdump\""));
    // The observation is the real (offset, is_dst, abbr).
    let w = &report.witnesses[0];
    let o = w.zic_rs.as_ref().unwrap();
    assert_eq!(o.offset_seconds, 0);
    assert!(!o.is_dst);
    assert_eq!(o.abbreviation, "UTC");
}

#[test]
fn artifact_category_strings_are_unique() {
    use ArtifactCategory::*;
    let all = [
        CompileInput,
        PolicyInput,
        ReferenceInput,
        GeneratedArtifact,
        OutputArtifact,
        DiagnosticArtifact,
        SemanticWitnessArtifact,
        StructuralValidationArtifact,
        PolicyProse,
        ReleaseNoteEvidence,
    ];
    let mut seen = std::collections::BTreeSet::new();
    for c in all {
        assert!(
            seen.insert(c.as_str()),
            "duplicate ArtifactCategory string {}",
            c.as_str()
        );
    }
    assert_eq!(
        seen.len(),
        10,
        "the T12 evidence-category spine + T15 witness/structural categories"
    );
}
