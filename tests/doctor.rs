//! T16.6b — `doctor` integration tests.
//!
//! `doctor` is a read-only environment probe that **always exits 0** and degrades every probe to an
//! explicit `Absent`/`not_found` (never a silent gap). These tests pin the no-host case (every tool
//! absent → all-`Absent`, valid JSON, no panic) and `ToolStatus` totality.

use std::process::Command;
use tzcompile::doctor::{run_doctor, DoctorOptions, ToolStatus, ToolVersionStatus, TzdataStatus};

/// Probe with deliberately bogus tool names → everything resolves to `Absent`, and the run still
/// succeeds (a diagnosis, not a gate).
fn absent_report() -> tzcompile::doctor::DoctorReport {
    run_doctor(&DoctorOptions {
        reference_zic: "zic-rs-definitely-not-a-real-binary".into(),
        reference_zdump: "zic-rs-definitely-not-a-real-binary".into(),
        tzdata: None,
    })
    .expect("doctor never fails")
}

#[test]
fn bogus_tools_are_all_absent_and_run_succeeds() {
    let report = absent_report();
    assert_eq!(report.reference_zic, ToolStatus::Absent);
    assert_eq!(report.reference_zdump, ToolStatus::Absent);
    assert_eq!(report.tzdata, TzdataStatus::NotProbed);
}

#[test]
fn json_shape_is_well_formed_when_everything_absent() {
    let json = absent_report().to_json();
    assert!(json.contains("\"schema\": \"zic-rs-doctor-v2\""));
    assert!(json.contains("\"reference_zic\""));
    assert!(json.contains("\"reference_zdump\""));
    assert!(json.contains("\"tzdata\""));
    assert!(json.contains("\"compiler_identity\""));
    assert!(json.contains("\"status\": \"absent\""));
    assert!(json.contains("\"status\": \"not_probed\""));
    assert!(json.contains("\"non_claim\""));
}

#[test]
fn text_render_is_well_formed_when_everything_absent() {
    let text = absent_report().to_text();
    assert!(text.contains("reference zic"));
    assert!(text.contains("reference zdump"));
    assert!(text.contains("absent"));
    assert!(text.contains("zic-rs"));
}

/// T17.3: a present tool with a working `--version` reports a typed `Supported` version and a `Read`
/// hash — never a fake-version string or a non-hash `sha256`. Uses `cargo` (always present under
/// `cargo test`), resolved via PATH; auto-skips in the unlikely event it isn't on PATH.
#[test]
fn present_tool_reports_typed_supported_version_and_read_hash() {
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("note: cargo not spawnable — skipping present-tool probe");
        return;
    }
    let report = run_doctor(&DoctorOptions {
        reference_zic: "cargo".into(), // any present, `--version`-supporting tool
        reference_zdump: "zic-rs-not-real".into(),
        tzdata: None,
    })
    .expect("doctor never fails");
    match &report.reference_zic {
        ToolStatus::Present {
            version_status,
            hash_status,
            ..
        } => {
            assert!(
                matches!(version_status, ToolVersionStatus::Supported { .. }),
                "cargo --version should classify as Supported, got {version_status:?}"
            );
            assert!(
                matches!(hash_status, tzcompile::doctor::HashReadStatus::Read { .. }),
                "the resolved binary should hash as Read, got {hash_status:?}"
            );
        }
        ToolStatus::Absent => panic!("cargo should resolve on PATH"),
    }
    // JSON carries the typed sub-objects, not a bare version/sha256.
    let json = report.to_json();
    assert!(json.contains("\"version_status\""));
    assert!(json.contains("\"hash_status\""));
    assert!(json.contains("\"status\": \"supported\""));
    assert!(json.contains("\"status\": \"read\""));
}

#[test]
fn missing_tzdata_path_is_not_found_not_an_error() {
    let report = run_doctor(&DoctorOptions {
        reference_zic: "zic-rs-not-real".into(),
        reference_zdump: "zic-rs-not-real".into(),
        tzdata: Some("/nonexistent/zic-rs/tzdata.zi".into()),
    })
    .expect("doctor never fails");
    assert!(matches!(report.tzdata, TzdataStatus::NotFound(_)));
    let json = report.to_json();
    assert!(json.contains("\"status\": \"not_found\""));
}
