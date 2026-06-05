//! T14.5 — `ZoneNamePathPolicy`: zone/link **name → output path** admissibility, as an executable
//! witness (twin of `docs/zic-zone-name-path-policy.md`).
//!
//! A zone/link name becomes an output path under `--out`, so it is untrusted input on two axes that
//! must not be conflated:
//!
//! * **Structural path safety (fatal, `ZIC008`)** — absolute · `..`/`.` component · empty · `//` ·
//!   trailing `/` · leading-`-` component · NUL. zic-rs rejects all of these (the first five match
//!   reference `zic`'s fatal `namecheck`/`componentcheck`; leading-`-` and NUL are bucket-3 *safer
//!   divergences* — reference only `-v`-warns on leading-`-`). Enforced by `safe_relative_path`, so a
//!   bad name aborts the compile before any write (no partial install).
//! * **Name portability (verbose-only warning)** — a byte outside the benign set (`ZIC024`) or an
//!   overlength component (`ZIC025`), mirroring reference `zic -v`'s `namecheck`/`componentcheck`
//!   warnings. The file still compiles (these never change bytes or exit status).
//!
//! No behaviour change to valid canonical output (CORE.1 341/0/0): real names like `America/New_York`
//! are all benign; only digit/`+`/punctuation names (e.g. `Etc/GMT+5`) trip the verbose-only warnings,
//! exactly as reference `zic -v` does.

use tzcompile::compile::plan;
use tzcompile::{
    load_database, CompileConfig, CompileReport, DiagnosticCode, EmitStyle, LinkMode, Severity,
    UnsupportedPolicy, ZoneSelection, DEFAULT_TRANSITION_LIMIT,
};

/// Compile `source` (all supported zones) to a throwaway dir; return the report or the failing error.
fn compile_all(source: &str) -> tzcompile::Result<CompileReport> {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, source).unwrap();
    let inputs = vec![p];
    let db = load_database(&inputs)?;
    let config = CompileConfig {
        input_paths: inputs.clone(),
        output_dir: dir.path().join("out"),
        zones: ZoneSelection::AllSupported,
        link_mode: LinkMode::Copy,
        overwrite: true,
        unsupported_policy: UnsupportedPolicy::Error,
        transition_limit: DEFAULT_TRANSITION_LIMIT,
        emit_style: EmitStyle::Default,
        no_create_dirs: false,
        localtime: None,
        localtime_name: None,
        file_mode: None,
        redundant_until: None,
        range: None,
        leaps: None,
        allow_empty_footer_on_legacy_nonposix_recurrence: false,
    };
    plan::run(&db, &config)
}

/// Codes present in a successful report's collected diagnostics.
fn codes(report: &CompileReport) -> Vec<DiagnosticCode> {
    report.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn structural_path_violations_are_fatal_zic008() {
    // Each is a link *name* (an output path) that must be rejected before any write. The target zone is
    // defined so the link is actually selected/materialised (where the name is validated).
    let bad_names = [
        "/abs/evil",   // absolute
        "a/../escape", // parent traversal
        "a//b",        // empty interior component
        "trailing/",   // trailing slash
        "-startsdash", // leading '-' component (zic-rs safer divergence; reference only -v warns)
    ];
    for bad in bad_names {
        let src = format!("Zone Real/Z 0:00 - X\nLink Real/Z {bad}\n");
        let err = compile_all(&src).expect_err(&format!("{bad:?} must be rejected"));
        assert_eq!(
            err.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::OutputPathTraversal),
            "{bad:?} should be ZIC008",
        );
    }
}

#[test]
fn benign_names_compile_with_no_name_warnings() {
    // Real-shaped names (letters, '_', '/') are fully portable — no ZIC024/ZIC025.
    let report =
        compile_all("Zone America/New_York 0:00 - LMT\nLink America/New_York US/Eastern\n")
            .expect("benign names compile");
    let cs = codes(&report);
    assert!(
        !cs.contains(&DiagnosticCode::ZoneNameNonPortableByte)
            && !cs.contains(&DiagnosticCode::ZoneNameOverlengthComponent),
        "benign names must not warn, got {cs:?}",
    );
}

#[test]
fn nonportable_byte_in_name_warns_zic024_verbose_only() {
    // `Etc/GMT+5`: `+` (and `5`) are outside the benign set → ZIC024, mirroring reference `zic -v`
    // "file name '…' contains byte '+'". The file still compiles (warning, not error).
    let report =
        compile_all("Zone Etc/GMT+5 -5:00 - X\n").expect("non-portable name still compiles");
    let zic024 = report
        .diagnostics
        .iter()
        .find(|d| d.code == DiagnosticCode::ZoneNameNonPortableByte)
        .expect("ZIC024 expected for `Etc/GMT+5`");
    assert_eq!(zic024.severity, Severity::Warning);
    // Verbose-only: it is *collected* in the report but gated out of quiet CLI output.
    assert_eq!(
        zic024.verbosity,
        tzcompile::DiagnosticVerbosity::VerboseOnly,
        "name portability warnings are -v-gated, matching reference `zic`'s `noise`",
    );
}

#[test]
fn overlength_component_warns_zic025() {
    // A component longer than reference's 14-byte portable cap → ZIC025 (verbose-only warning).
    let report = compile_all("Zone Area/Supercalifragilistic 0:00 - X\n")
        .expect("overlength-component name still compiles");
    assert!(
        codes(&report).contains(&DiagnosticCode::ZoneNameOverlengthComponent),
        "ZIC025 expected for an over-14-byte component",
    );
}
