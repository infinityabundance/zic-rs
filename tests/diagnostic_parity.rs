//! T13.2 — diagnostic-parity comparison harness (class/location-first).
//!
//! For each crafted malformed input this harness compares zic-rs's coded diagnostic to reference
//! `zic -v` **by canonical class and source line, before wording** (the T13.1 method). The reference
//! half **auto-skips** when `zic` is not on `PATH` (oracle-availability), but zic-rs's own
//! class/location is always asserted (deterministic, via the public `load_database` API + the rendered
//! diagnostic string `…:LINE: severity[ZICnnn_…]: message`).
//!
//! Diagnostics are `diagnostic_artifact`s (T12 doctrine): this proves the tools *noticed the same
//! thing in the same place*, not anything about compiled output. Wording is compared **last** and only
//! recorded — see the wording ledger in `docs/zic-warning-parity.md`. Class names are the **rule
//! violated**, never the printed text.

use std::process::Command;

use tzcompile::compile::plan;
use tzcompile::{
    compile_zone_to_bytes, load_database, CompileConfig, EmitStyle, LinkMode, Severity,
    UnsupportedPolicy, ZoneSelection, DEFAULT_TRANSITION_LIMIT,
};

/// A canonical, wording-independent class name (the "rule violated"). Both zic-rs codes and reference
/// `zic` wordings map onto these so the comparison never depends on exact English.
fn zic_rs_canonical_class(rendered_error: &str) -> Option<&'static str> {
    // The rendered error contains `[ZICnnn_NAME]`; map the code to a canonical class.
    const MAP: &[(&str, &str)] = &[
        ("ZIC013_UNKNOWN_LINE_TYPE", "UnknownLineType"),
        (
            "ZIC014_CONTINUATION_WITHOUT_ZONE",
            "ContinuationWithoutZone",
        ),
        ("ZIC015_DUPLICATE_ZONE", "DuplicateZone"),
        ("ZIC016_NUL_INPUT_BYTE", "NulByteInInput"),
        ("ZIC002_INVALID_FIELD_COUNT", "InvalidFieldCount"),
        ("ZIC021_UNTERMINATED_INPUT_LINE", "UnterminatedInputLine"),
        ("ZIC022_UNTERMINATED_QUOTE", "UnterminatedQuote"),
        ("ZIC026_VALUE_OVER_24_HOURS", "ValueOver24Hours"),
    ];
    MAP.iter()
        .find(|(code, _)| rendered_error.contains(code))
        .map(|(_, class)| *class)
}

/// Map a line of reference `zic` stderr to the same canonical class (pinned to tzcode 2026b wording;
/// compared by class, never asserted verbatim — wording drifts across releases/vendors).
fn reference_canonical_class(stderr: &str) -> Option<&'static str> {
    const MAP: &[(&str, &str)] = &[
        ("input line of unknown type", "UnknownLineType"),
        ("duplicate zone name", "DuplicateZone"),
        ("NUL input byte", "NulByteInInput"),
        ("wrong number of fields", "InvalidFieldCount"),
        ("unterminated line", "UnterminatedInputLine"),
        ("Odd number of quotation marks", "UnterminatedQuote"),
    ];
    MAP.iter()
        .find(|(needle, _)| stderr.contains(needle))
        .map(|(_, class)| *class)
}

/// The explicit verdict vocabulary.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    ClassLocationMatch,
    ClassMatchLocationDiff,
    IntentionalDivergence,
    LayerMatchClassGap,
    ReferenceOnly,
    ZicRsOnly,
}

/// zic-rs may classify a continuation-shaped stray line more precisely than reference `zic` (which
/// folds it into "input line of unknown type"). That specific pair is a *documented* divergence.
fn is_intentional_divergence(zic_rs: &str, reference: &str) -> bool {
    zic_rs == "ContinuationWithoutZone" && reference == "UnknownLineType"
}

fn verdict(zic_rs: Option<(&str, usize)>, reference: Option<(&str, usize)>) -> Verdict {
    match (zic_rs, reference) {
        (Some((cz, lz)), Some((cr, lr))) => {
            if cz == cr && lz == lr {
                Verdict::ClassLocationMatch
            } else if cz == cr {
                Verdict::ClassMatchLocationDiff
            } else if is_intentional_divergence(cz, cr) {
                Verdict::IntentionalDivergence
            } else {
                Verdict::LayerMatchClassGap
            }
        }
        (Some(_), None) => Verdict::ZicRsOnly,
        (None, Some(_)) => Verdict::ReferenceOnly,
        (None, None) => Verdict::ZicRsOnly, // both silent shouldn't happen for these fixtures
    }
}

struct Fixture {
    name: &'static str,
    source: &'static str,
    /// Expected zic-rs canonical class + line.
    zic_rs_class: &'static str,
    zic_rs_line: usize,
    /// Acceptable verdicts when the reference is available.
    accept: &'static [Verdict],
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "unknown_keyword",
        source: "Frobnicate A 0 - X\n",
        zic_rs_class: "UnknownLineType",
        zic_rs_line: 1,
        accept: &[Verdict::ClassLocationMatch],
    },
    Fixture {
        name: "continuation_without_zone",
        source: "\t\t\t1:00 - BST 1990\n",
        zic_rs_class: "ContinuationWithoutZone",
        zic_rs_line: 1,
        // Reference folds this into "input line of unknown type" → documented divergence.
        accept: &[Verdict::IntentionalDivergence],
    },
    Fixture {
        name: "duplicate_zone",
        source: "Zone A 0 - X\nZone A 1 - Y\n",
        zic_rs_class: "DuplicateZone",
        zic_rs_line: 2,
        accept: &[Verdict::ClassLocationMatch],
    },
    Fixture {
        name: "nul_byte",
        source: "Zone A 0 - X\0Y\n",
        zic_rs_class: "NulByteInInput",
        zic_rs_line: 1,
        accept: &[Verdict::ClassLocationMatch],
    },
    Fixture {
        name: "wrong_field_count",
        source: "Zone\n",
        zic_rs_class: "InvalidFieldCount",
        zic_rs_line: 1,
        accept: &[Verdict::ClassLocationMatch],
    },
    Fixture {
        // T14.2: a non-empty final line with no terminating newline. Reference `inputline` → fatal
        // "unterminated line" on line 1; zic-rs → `ZIC021` on line 1. (Source intentionally lacks `\n`.)
        name: "missing_final_newline",
        source: "Zone A 0 - X",
        zic_rs_class: "UnterminatedInputLine",
        zic_rs_line: 1,
        accept: &[Verdict::ClassLocationMatch],
    },
    Fixture {
        // T14.2: an odd number of quotation marks. Reference `getfields` → fatal "Odd number of
        // quotation marks" on line 1; zic-rs → `ZIC022` on line 1. Newline-terminated to isolate the
        // quote rule from the missing-newline rule.
        name: "unterminated_quote",
        source: "Zone A 0 - \"X\n",
        zic_rs_class: "UnterminatedQuote",
        zic_rs_line: 1,
        accept: &[Verdict::ClassLocationMatch],
    },
];

/// Compile `source` with zic-rs (parse layer) and return `(canonical_class, line)` from the error.
fn zic_rs_diagnostic(dir: &std::path::Path, source: &str) -> Option<(&'static str, usize)> {
    let p = dir.join("in.zi");
    std::fs::write(&p, source).unwrap();
    let err = tzcompile::load_database(&[p]).err()?;
    let rendered = err.to_string();
    let class = zic_rs_canonical_class(&rendered)?;
    // Rendered shape: `<path>:LINE: severity[CODE]: message` — pull the first `:N:` after `.zi`.
    let line = rendered
        .split(".zi:")
        .nth(1)
        .and_then(|rest| rest.split(':').next())
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    Some((class, line))
}

/// Run reference `zic -v` on `source`; return `(canonical_class, line)` from stderr, or `None` if
/// `zic` is unavailable or produced no mappable diagnostic.
fn reference_diagnostic(dir: &std::path::Path, source: &str) -> Option<(&'static str, usize)> {
    let p = dir.join("ref.zi");
    let out = dir.join("refout");
    std::fs::write(&p, source).unwrap();
    let output = Command::new("zic")
        .args(["-v", "-d"])
        .arg(&out)
        .arg(&p)
        .output()
        .ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let class = reference_canonical_class(&stderr)?;
    // Reference shape: `… line N: …`.
    let line = stderr
        .split("line ")
        .nth(1)
        .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    Some((class, line))
}

fn zic_available() -> bool {
    Command::new("zic")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn diagnostics_compare_by_class_and_location() {
    let have_zic = zic_available();
    if !have_zic {
        eprintln!(
            "note: reference `zic` not on PATH — asserting zic-rs classes only (oracle skip)"
        );
    }
    for f in FIXTURES {
        let dir = tempfile::tempdir().unwrap();

        // zic-rs side is always asserted (deterministic).
        let zr = zic_rs_diagnostic(dir.path(), f.source);
        assert_eq!(
            zr,
            Some((f.zic_rs_class, f.zic_rs_line)),
            "{}: zic-rs class/line",
            f.name
        );

        if !have_zic {
            continue;
        }
        // Reference comparison: class/location-first, verdict in the accepted set.
        let rf = reference_diagnostic(dir.path(), f.source);
        let v = verdict(zr, rf);
        eprintln!(
            "  [{}] zic-rs={zr:?} reference={rf:?} verdict={v:?}",
            f.name
        );
        assert!(
            f.accept.contains(&v),
            "{}: verdict {v:?} not in accepted {:?} (zic-rs={zr:?}, reference={rf:?})",
            f.name,
            f.accept
        );
    }
}

/// The five new coded classes (T13.2) are stable, distinct strings (contract guard).
#[test]
fn new_diagnostic_codes_are_distinct_and_stable() {
    let dir = tempfile::tempdir().unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for f in FIXTURES {
        if let Some((class, _)) = zic_rs_diagnostic(dir.path(), f.source) {
            seen.insert(class);
        }
    }
    // All five canonical classes the harness exercises were observed.
    for expected in [
        "UnknownLineType",
        "ContinuationWithoutZone",
        "DuplicateZone",
        "NulByteInInput",
        "InvalidFieldCount",
    ] {
        assert!(seen.contains(expected), "missing class: {expected}");
    }
}

// ---------------------------------------------------------------------------------------------
// T13.3 — warning-severity diagnostics (the first non-fatal `-v` class). Unlike the parse-time
// errors above (surfaced by `load_database`), a warning is collected during *compile* and the file
// still compiles — so this half of the harness drives the real `plan::run` pipeline and reads
// `CompileReport.diagnostics` (warnings are structured artifacts, never bolted-on stderr text).
// ---------------------------------------------------------------------------------------------

/// Compile `source` (all-supported) into a temp tree and return `(warnings, compiled_ok)`, where
/// `warnings` is each `Severity::Warning` diagnostic's `(canonical_class, line)`. Drives the public
/// `plan::run` so the test exercises the same path the CLI does.
fn zic_rs_warnings(dir: &std::path::Path, source: &str) -> (Vec<(&'static str, usize)>, bool) {
    let src = dir.join("warn.zi");
    std::fs::write(&src, source).unwrap();
    let db = load_database(std::slice::from_ref(&src)).expect("source parses");
    let config = CompileConfig {
        input_paths: vec![src],
        output_dir: dir.join("out"),
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
    let report = plan::run(&db, &config).expect("compiles (warnings are non-fatal)");
    let warnings = report
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .filter_map(|d| warning_canonical_class(d.code.as_str()).map(|c| (c, d.line)))
        .collect();
    (warnings, !report.zones_compiled.is_empty())
}

/// Map a zic-rs warning code to its canonical class.
fn warning_canonical_class(code: &str) -> Option<&'static str> {
    if code.contains("ZIC018_ABBREVIATION_POLICY_VIOLATION") {
        Some("AbbreviationPolicyViolation")
    } else if code.contains("ZIC019_ABBREVIATION_NOT_POSIX") {
        Some("AbbreviationNotPosix")
    } else {
        None
    }
}

/// Map reference `zic` stderr to the same canonical warning class (pinned tzcode 2026b wording). Both
/// length sides ("too many" / "fewer than 3") are the one length-policy class; the POSIX character-set
/// message is its own class.
fn reference_warning_class(stderr: &str) -> Option<&'static str> {
    if stderr.contains("too many characters") || stderr.contains("fewer than 3 characters") {
        Some("AbbreviationPolicyViolation")
    } else if stderr.contains("differs from POSIX standard") {
        Some("AbbreviationNotPosix")
    } else {
        None
    }
}

/// A 7-character abbreviation (> 6) — the pinned reference threshold for the always-on warning.
const WARN_SOURCE: &str = "Zone Test/Long 0 - ABCDEFG\n";

/// **(test 1)** Over-long abbreviation → a *warning*, not an error: the zone still compiles, and
/// exactly one `AbbreviationPolicyViolation` is collected at the zone's line. A 6-char abbreviation is
/// silent (the pinned threshold).
#[test]
fn abbreviation_too_long_emits_warning_not_error() {
    let dir = tempfile::tempdir().unwrap();
    let (warnings, compiled) = zic_rs_warnings(dir.path(), WARN_SOURCE);
    assert!(
        compiled,
        "the zone must still compile — warnings are non-fatal"
    );
    assert_eq!(warnings, vec![("AbbreviationPolicyViolation", 1)]);

    let dir2 = tempfile::tempdir().unwrap();
    let (none, ok) = zic_rs_warnings(dir2.path(), "Zone Test/Ok 0 - ABCDEF\n"); // 6 chars
    assert!(ok);
    assert!(
        none.is_empty(),
        "6-char abbreviation must be silent: {none:?}"
    );
}

/// **(test 2)** Warning-severity diagnostics compare by class/location before wording, vs
/// reference `zic` (auto-skips if `zic` absent).
#[test]
fn abbreviation_policy_violation_matches_reference_class_location() {
    let dir = tempfile::tempdir().unwrap();
    let (warnings, _) = zic_rs_warnings(dir.path(), WARN_SOURCE);
    let zr = warnings.first().copied();
    assert_eq!(
        zr,
        Some(("AbbreviationPolicyViolation", 1)),
        "zic-rs class/line"
    );

    if !zic_available() {
        eprintln!("note: reference `zic` absent — asserted zic-rs warning only (oracle skip)");
        return;
    }
    let p = dir.path().join("ref.zi");
    let out = dir.path().join("refout");
    std::fs::write(&p, WARN_SOURCE).unwrap();
    // `-v`: some abbreviation warnings ("fewer than 3 characters") are verbose-gated in reference
    // `zic`, while zic-rs (no verbosity levels) always collects them — so compare against `zic -v`.
    let output = Command::new("zic")
        .args(["-v", "-d"])
        .arg(&out)
        .arg(&p)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let rclass = reference_warning_class(&stderr);
    let rline = stderr
        .split("line ")
        .nth(1)
        .and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse::<usize>().ok());
    let rf = rclass.zip(rline);
    let v = verdict(zr, rf);
    eprintln!("  [abbreviation_too_long] zic-rs={zr:?} reference={rf:?} verdict={v:?}");
    assert_eq!(
        v,
        Verdict::ClassLocationMatch,
        "zic-rs={zr:?} reference={rf:?}"
    );
}

/// **(test 3)** The warning channel is a pure observation: collecting it does not change the
/// compiled TZif bytes — the zone produced under `plan::run` is byte-identical to a direct
/// `compile_zone_to_bytes` (so CORE.1 output is structurally untouched by the warning logic).
#[test]
fn warning_collection_does_not_change_core1_output() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("z.zi");
    std::fs::write(&src, WARN_SOURCE).unwrap();
    let db = load_database(&[src]).unwrap();

    // Bytes via the direct API (no warning collection path).
    let direct = compile_zone_to_bytes(&db, "Test/Long").unwrap();
    // Bytes via the full run (which *does* collect the warning).
    let (warnings, _) = zic_rs_warnings(dir.path(), WARN_SOURCE);
    assert_eq!(warnings.len(), 1, "precondition: the warning fired");
    let via_run = std::fs::read(dir.path().join("out/Test/Long")).unwrap();

    assert_eq!(
        direct, via_run,
        "warning collection must not alter compiled bytes"
    );
}

/// **(test 4)** Wording is *recorded, not optimized*: zic-rs keeps its own actionable message
/// (carrying the offending abbreviation) and does **not** copy reference `zic`'s exact string — the
/// difference is tracked in the wording ledger (`docs/zic-warning-parity.md`), not erased.
#[test]
fn warning_wording_recorded_not_optimized() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("z.zi");
    std::fs::write(&src, WARN_SOURCE).unwrap();
    let db = load_database(std::slice::from_ref(&src)).unwrap();
    let config = CompileConfig {
        input_paths: vec![src],
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
    let report = plan::run(&db, &config).unwrap();
    let w = report
        .diagnostics
        .iter()
        .find(|d| d.severity == Severity::Warning)
        .expect("a warning");
    // Actionable: our wording names the offending abbreviation.
    assert!(
        w.message.contains("ABCDEFG"),
        "message names the abbreviation: {}",
        w.message
    );
    // We did NOT copy reference `zic`'s exact string verbatim (wording compared last; ledger records it).
    assert_ne!(
        w.message, "time zone abbreviation has too many characters (ABCDEFG)",
        "zic-rs keeps its own wording; the diff is recorded in the ledger, not optimized away"
    );
}

/// **(test 5)** The reference half auto-skips when `zic` is unavailable: the zic-rs warning is
/// asserted without ever invoking the reference, so the harness never silently weakens.
#[test]
fn reference_absent_auto_skips_warning_harness() {
    // This assertion never calls `zic` — it is the always-on zic-rs half. (The reference comparison in
    // `abbreviation_policy_violation_matches_reference_class_location` is what auto-skips.)
    let dir = tempfile::tempdir().unwrap();
    let (warnings, compiled) = zic_rs_warnings(dir.path(), WARN_SOURCE);
    assert!(compiled);
    assert_eq!(warnings, vec![("AbbreviationPolicyViolation", 1)]);
}

// --- T13.4: widened warning surface (abbreviation length 3–6 + POSIX character set) ---

struct WarnFixture {
    name: &'static str,
    source: &'static str,
    /// Expected zic-rs canonical warning classes, in emit order (length policy before POSIX).
    classes: &'static [&'static str],
    line: usize,
}

const WARN_FIXTURES: &[WarnFixture] = &[
    WarnFixture {
        name: "too_long",
        source: "Zone Test/Long 0 - ABCDEFG\n",
        classes: &["AbbreviationPolicyViolation"],
        line: 1,
    },
    WarnFixture {
        name: "too_short",
        source: "Zone Test/Short 0 - AB\n",
        classes: &["AbbreviationPolicyViolation"],
        line: 1,
    },
    WarnFixture {
        name: "non_posix",
        source: "Zone Test/Px 0 - A_B\n",
        classes: &["AbbreviationNotPosix"],
        line: 1,
    },
    WarnFixture {
        // 2 chars AND a non-POSIX `_`: `zic.c::checkabbr` precedence makes POSIX win (one warning) —
        // the conforming prefix is "A" (len 1), but the trailing `_` overwrites `mp` to the POSIX
        // message. So *only* `AbbreviationNotPosix`, never also "fewer than 3" (matched, not guessed).
        name: "non_posix_wins_over_short",
        source: "Zone Test/Both 0 - A_\n",
        classes: &["AbbreviationNotPosix"],
        line: 1,
    },
];

/// **(T13.4)** The widened warning surface — abbreviation length (3–6) and POSIX character set — emits
/// the right coded classes (one rule may fire, or both), and each matches reference `zic` *by class*
/// (and location). Class/location-first; wording is not asserted. Auto-skips the reference half if
/// `zic` is absent.
#[test]
fn abbreviation_warnings_by_class_and_location() {
    let have_zic = zic_available();
    for f in WARN_FIXTURES {
        let dir = tempfile::tempdir().unwrap();
        let (warnings, compiled) = zic_rs_warnings(dir.path(), f.source);
        assert!(
            compiled,
            "{}: must still compile (warnings non-fatal)",
            f.name
        );

        let got: Vec<&str> = warnings.iter().map(|(c, _)| *c).collect();
        assert_eq!(got, f.classes, "{}: zic-rs warning classes", f.name);
        for (_, line) in &warnings {
            assert_eq!(*line, f.line, "{}: warning line", f.name);
        }

        if !have_zic {
            continue;
        }
        let p = dir.path().join("ref.zi");
        let out = dir.path().join("refout");
        std::fs::write(&p, f.source).unwrap();
        // `-v`: "fewer than 3 characters" is verbose-gated in reference `zic` (whereas "too many" and
        // "differs from POSIX" are always-on); zic-rs has no verbosity levels and always collects all
        // three, so the class/location comparison is against `zic -v`.
        let output = Command::new("zic")
            .args(["-v", "-d"])
            .arg(&out)
            .arg(&p)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Each zic-rs class must also be present in reference (set membership by class — reference
        // emit order is not constrained).
        for class in f.classes {
            let ref_has = match *class {
                "AbbreviationPolicyViolation" => {
                    stderr.contains("too many characters")
                        || stderr.contains("fewer than 3 characters")
                }
                "AbbreviationNotPosix" => stderr.contains("differs from POSIX standard"),
                other => panic!("unmapped class {other}"),
            };
            assert!(
                ref_has,
                "{}: reference missing {class}; stderr=[{stderr}]",
                f.name
            );
        }
        eprintln!("  [{}] zic-rs={got:?} reference-classes-present ✓", f.name);
    }
}

// --- T13.5: verbosity model (per-diagnostic, mirrors zic's `noise`/`-v` gating) ---

/// Compile `source` (all-supported) and return each warning's `(code_str, verbosity)`.
fn zic_rs_warning_verbosity(
    dir: &std::path::Path,
    source: &str,
) -> Vec<(String, tzcompile::DiagnosticVerbosity)> {
    let src = dir.join("v.zi");
    std::fs::write(&src, source).unwrap();
    let db = load_database(std::slice::from_ref(&src)).unwrap();
    let config = CompileConfig {
        input_paths: vec![src],
        output_dir: dir.join("out"),
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
    let report = plan::run(&db, &config).unwrap();
    report
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .map(|d| (d.code.as_str().to_string(), d.verbosity))
        .collect()
}

/// **(T13.5)** The verbosity model mirrors `zic`'s `noise` gating, set **per diagnostic**: the
/// "fewer than 3 characters" abbreviation warning is `VerboseOnly` (reference `-v`-gated), while
/// "too many characters" and "differs from POSIX" are `AlwaysOn`.
#[test]
fn abbreviation_fewer_than_3_is_verbose_only_others_always_on() {
    use tzcompile::DiagnosticVerbosity::{AlwaysOn, VerboseOnly};
    let d = tempfile::tempdir().unwrap();

    let short = zic_rs_warning_verbosity(d.path(), "Zone Test/S 0 - AB\n");
    assert_eq!(
        short,
        vec![("ZIC018_ABBREVIATION_POLICY_VIOLATION".into(), VerboseOnly)]
    );

    let d2 = tempfile::tempdir().unwrap();
    let long = zic_rs_warning_verbosity(d2.path(), "Zone Test/L 0 - ABCDEFG\n");
    assert_eq!(
        long,
        vec![("ZIC018_ABBREVIATION_POLICY_VIOLATION".into(), AlwaysOn)]
    );

    let d3 = tempfile::tempdir().unwrap();
    let posix = zic_rs_warning_verbosity(d3.path(), "Zone Test/P 0 - A_B\n");
    assert_eq!(
        posix,
        vec![("ZIC019_ABBREVIATION_NOT_POSIX".into(), AlwaysOn)]
    );
}

// --- T13.6: ZIC020 emission (verbose-only, output-preserving) ---

/// **(T13.6)** A zone whose emitted transition count exceeds the legacy-client threshold (>1200) emits
/// `ZIC020` as a **`VerboseOnly`** warning, the zone still compiles, and the compiled **bytes are
/// unchanged** (the warning is a pure observation over the finished `TzifData`).
#[test]
fn zic020_transition_count_is_verbose_only_and_output_preserving() {
    use tzcompile::DiagnosticVerbosity::VerboseOnly;
    // A finite DST rule over ~800 years → ~1600 transitions (> 1200).
    let source = "Rule R 1200 1999 - Mar Sun>=8 2:00 1:00 D\n\
                  Rule R 1200 1999 - Nov Sun>=1 2:00 0 S\n\
                  Zone Test/Many -5:00 R E%sT\n";
    let dir = tempfile::tempdir().unwrap();

    let warnings = zic_rs_warning_verbosity(dir.path(), source);
    assert_eq!(
        warnings,
        vec![(
            "ZIC020_TOO_MANY_TRANSITIONS_FOR_LEGACY_CLIENT".into(),
            VerboseOnly
        )],
        "a >1200-transition zone warns ZIC020 (verbose-only)"
    );

    // Byte-identity: the warning path did not change the compiled output.
    let src = dir.path().join("v.zi"); // written by zic_rs_warning_verbosity above
    let db = load_database(std::slice::from_ref(&src)).unwrap();
    let direct = compile_zone_to_bytes(&db, "Test/Many").unwrap();
    let via_run = std::fs::read(dir.path().join("out/Test/Many")).unwrap();
    assert_eq!(
        direct, via_run,
        "ZIC020 collection must not alter compiled bytes"
    );
}

// --- T15.5-remainder: ZIC026 "values over 24 hours" (verbose-only, output-preserving) ---

/// **(T15.5-remainder)** A `STDOFF` magnitude over 24:00:00 emits `ZIC026` as a **`VerboseOnly`**
/// warning (mirroring reference `zic`'s `gethms` `noise` check) while the zone still compiles — a pure
/// observation that never changes the compiled bytes. Exactly `24:00:00` does **not** warn.
#[test]
fn zic026_values_over_24h_is_verbose_only_and_output_preserving() {
    use tzcompile::DiagnosticVerbosity::VerboseOnly;
    let dir = tempfile::tempdir().unwrap();
    // STDOFF 25:00 (90000s) > 24h → exactly one ZIC026; the zone is otherwise a plain fixed offset.
    let warnings = zic_rs_warning_verbosity(dir.path(), "Zone Test/Big 25:00 - BIG\n");
    assert_eq!(
        warnings,
        vec![("ZIC026_VALUE_OVER_24_HOURS".into(), VerboseOnly)],
        "a >24h STDOFF warns ZIC026 (verbose-only)"
    );

    // Byte-identity: the warning path did not change the compiled output.
    let src = dir.path().join("v.zi");
    let db = load_database(std::slice::from_ref(&src)).unwrap();
    let direct = compile_zone_to_bytes(&db, "Test/Big").unwrap();
    let via_run = std::fs::read(dir.path().join("out/Test/Big")).unwrap();
    assert_eq!(direct, via_run, "ZIC026 must not alter compiled bytes");
}

/// **(T15.5-remainder)** Exactly `24:00:00` is the boundary reference allows silently — no `ZIC026`.
#[test]
fn zic026_exactly_24h_does_not_warn() {
    let dir = tempfile::tempdir().unwrap();
    // STDOFF 24:00 (86400s) == 24h → no warning (reference: hh==24 && mm==0 && ss==0 ⇒ no warn).
    let warnings = zic_rs_warning_verbosity(dir.path(), "Zone Test/Edge 24:00 - EDGE\n");
    assert!(
        warnings
            .iter()
            .all(|(c, _)| c != "ZIC026_VALUE_OVER_24_HOURS"),
        "exactly 24:00:00 must not warn ZIC026: {warnings:?}"
    );
}

// --- T13.6: reference-platform diagnostic oracle matrix (seed) ---

/// A diagnostic-oracle platform and its admission status (T13.5/T13.6). Vendor/platform diagnostics
/// are **admitted per platform, never inferred** from upstream IANA.
#[derive(Debug, PartialEq, Eq)]
enum OracleStatus {
    /// The pinned upstream reference — the active diagnostic oracle.
    AdmittedUpstream,
    /// A locally available `zic` (identity recorded); a separate oracle from upstream.
    AdmittedLocal,
    /// We have documented caveats but no runnable binary on this host.
    DocumentationOnly,
    /// No binary and no run on this host.
    UnavailableOnThisHost,
}

/// **(T13.6)** Seed the reference-platform diagnostic matrix. Records the upstream oracle, captures a
/// local `zic` as a *separate* oracle when present (with `skipped_with_reason` otherwise), and pins
/// every vendor/platform row with an explicit status — **no vendor row is ever `Admitted*` by
/// assumption** (the per-platform admission rule). Honest oracle accounting, not deferral.
#[test]
fn reference_platform_diagnostic_matrix_seed() {
    // upstream IANA / tzcode 2026b is the admitted oracle (the pinned reference the harness uses).
    let mut matrix: Vec<(&str, OracleStatus, String)> = vec![(
        "upstream_iana_2026b",
        OracleStatus::AdmittedUpstream,
        "pinned reference oracle".into(),
    )];

    // local_system_zic — admitted *separately* iff a binary is on PATH; record its identity.
    let local = std::process::Command::new("zic").arg("--version").output();
    match local {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            matrix.push(("local_system_zic", OracleStatus::AdmittedLocal, ver));
        }
        _ => matrix.push((
            "local_system_zic",
            OracleStatus::UnavailableOnThisHost,
            "skipped_with_reason: no `zic` on PATH".into(),
        )),
    }

    // Vendor/platform rows: documented caveats where we have them, else unavailable. NEVER admitted
    // without running their binary (per-platform admission, never inferred from upstream).
    matrix.push((
        "aix",
        OracleStatus::DocumentationOnly,
        "caveat: pre-1901 / 32-bit time_t limit".into(),
    ));
    matrix.push((
        "solaris_illumos",
        OracleStatus::DocumentationOnly,
        "caveat: vendor `-s` option history".into(),
    ));
    matrix.push((
        "bsd",
        OracleStatus::UnavailableOnThisHost,
        "skipped_with_reason: no binary on this host".into(),
    ));
    matrix.push((
        "macos",
        OracleStatus::UnavailableOnThisHost,
        "skipped_with_reason: no binary on this host".into(),
    ));
    matrix.push((
        "linux_glibc",
        OracleStatus::UnavailableOnThisHost,
        "skipped_with_reason: distinct glibc-packaged zic not pinned here".into(),
    ));

    for (name, status, note) in &matrix {
        eprintln!("  reference_platform[{name}] = {status:?} ({note})");
    }

    // Invariants: upstream is the admitted oracle; every row has an explicit status; and **no
    // vendor/platform row is admitted by assumption** (only upstream + a real local binary may be).
    assert_eq!(matrix[0].1, OracleStatus::AdmittedUpstream);
    for (name, status, _) in &matrix {
        let is_vendor = !matches!(*name, "upstream_iana_2026b" | "local_system_zic");
        if is_vendor {
            assert!(
                !matches!(status, OracleStatus::AdmittedUpstream | OracleStatus::AdmittedLocal),
                "{name}: vendor diagnostics must be admitted per-platform (by running its zic), never inferred"
            );
        }
    }
    // The full seed set is represented (none silently dropped).
    assert_eq!(matrix.len(), 7, "upstream + local + 5 vendor/platform rows");
}
