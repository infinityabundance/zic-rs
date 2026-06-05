//! Output-tree safety: path traversal is refused, output is never silently clobbered, and
//! the CLI requires an explicit `--out`.

use std::path::PathBuf;
use std::process::Command;

use tzcompile::diagnostics::DiagnosticCode;
use tzcompile::fs::output_tree::{safe_relative_path, write_zone_file};

#[test]
fn traversal_names_are_rejected_with_zic008() {
    for bad in ["../etc/passwd", "/abs", "a/../b", "..", ".", "-x/y", ""] {
        let err = safe_relative_path(bad).unwrap_err();
        assert_eq!(
            err.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::OutputPathTraversal),
            "name {bad:?} must be rejected as traversal"
        );
    }
}

#[test]
fn write_refuses_traversal_target() {
    let dir = tempfile::tempdir().unwrap();
    let err = write_zone_file(dir.path(), "../escape", b"x", false, true).unwrap_err();
    assert_eq!(
        err.diagnostic().map(|d| d.code),
        Some(DiagnosticCode::OutputPathTraversal)
    );
}

#[test]
fn no_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    write_zone_file(dir.path(), "Zone/A", b"first", false, true).unwrap();
    // Second write without overwrite must fail.
    assert!(write_zone_file(dir.path(), "Zone/A", b"second", false, true).is_err());
    // With overwrite it succeeds and replaces the content.
    write_zone_file(dir.path(), "Zone/A", b"second", true, true).unwrap();
    let got = std::fs::read(dir.path().join("Zone/A")).unwrap();
    assert_eq!(got, b"second");
}

#[test]
fn cli_requires_out() {
    // Missing --out must be a clean configuration error (non-zero exit), not a panic and
    // certainly not a write into a default system directory.
    let out = Command::new(env!("CARGO_BIN_EXE_zic-rs"))
        .args([
            "compile",
            "--input",
            &fixture("fixtures/minimal/utc.zi"),
            "--zone",
            "Etc/UTC",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "missing --out must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--out"),
        "error should mention --out: {stderr}"
    );
}

#[test]
fn cli_fails_closed_on_unsupported_rule_zone() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("rule.zi");
    // An inline-save era with a `%s` FORMAT is unsupported (inline save has no LETTER to
    // substitute) and must fail closed. (Fixed-offset, finite/recurring rule, multi-era UNTIL,
    // and inline-save with literal/`%z` FORMAT now compile — see tests/transition_generation,
    // tests/multi_era, and the unit tests.)
    std::fs::write(&src, "Zone Z -5:00 1:00 E%sT\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_zic-rs"))
        .args([
            "compile",
            "--input",
            src.to_str().unwrap(),
            "--out",
            dir.path().join("out").to_str().unwrap(),
            "--zone",
            "Z",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "rule-driven zone must fail closed");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("ZIC001_UNSUPPORTED_DIRECTIVE"),
        "expected unsupported diagnostic, got: {stderr}"
    );
}

fn fixture(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_str()
        .unwrap()
        .to_string()
}
