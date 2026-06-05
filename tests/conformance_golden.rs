//! T15.5-remainder — **golden** + **failure-mode** report fixtures.
//!
//! Two complementary guards on the public conformance engine:
//!
//! 1. **Golden** (`support-report.golden.json`): the full `support-report` JSON is pinned byte-for-byte,
//!    after normalising the two *host-variant* fields (`compiler_identity.target` / `profile`) to
//!    placeholders. Everything else — including the deterministic `declared_scope_hash`, the bounded
//!    `conformance_level`, the typed claim-shape axes, and the negative-capabilities array — is fixed, so
//!    any unintended schema or value drift fails here, not silently in production. (`target`/`profile`
//!    are normalised precisely because a byte-exact golden across architectures would be a *false*
//!    failure — the honest golden pins what is genuinely deterministic.)
//!
//! 2. **Failure-mode** shapes: a reviewer trusts a report engine more when it shows *how failure looks*.
//!    Each distinct failure surface (oracle-unavailable, structural-violation, negative-capability-present,
//!    workspace-unknown, reference-unavailable) is asserted to render its honest, non-silent shape. Most
//!    are exercised in their own suites (`semantic_witness.rs`, `tzif_validation.rs`); this file gathers
//!    the *report-as-artifact* failure shapes in one place as the T15.5-remainder contract.

use tzcompile::load_database;
use tzcompile::report::build_support_report;

const SRC: &str = "Zone Etc/UTC 0:00 - UTC\nLink Etc/UTC UTC\n";

/// Build the support-report JSON, then normalise the two host-variant `compiler_identity` fields so the
/// golden is stable across architectures and build profiles (everything else is deterministic).
fn normalized_support_report_json() -> String {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, SRC).unwrap();
    let db = load_database(std::slice::from_ref(&p)).unwrap();
    let raw = build_support_report(&db, None).to_json();

    // Replace the host-specific target + profile with placeholders (regex-free, byte-stable).
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw.as_str();
    for (key, placeholder) in [
        ("\"target\": \"", "\"target\": \"<target>\""),
        ("\"profile\": \"", "\"profile\": \"<profile>\""),
    ] {
        let (head, tail) = rest.split_once(key).expect("field present");
        out.push_str(head);
        // Skip to the closing quote of the original value.
        let value_end = tail.find('"').expect("value closes");
        out.push_str(placeholder);
        rest = &tail[value_end + 1..];
    }
    out.push_str(rest);
    out
}

/// **(T15.5-remainder)** The full `support-report` JSON matches the committed golden (host-variant fields
/// normalised). This pins the entire public schema + values in one place — including the deterministic
/// `declared_scope_hash`; a drift in any field, ordering, or the hash fails here.
#[test]
fn support_report_matches_golden() {
    let golden = include_str!("../fixtures/conformance/support-report.golden.json");
    let actual = normalized_support_report_json();
    assert_eq!(
        actual.trim_end(),
        golden.trim_end(),
        "support-report JSON drifted from the golden fixture.\n--- actual ---\n{actual}\n\
         (if this change is intentional, regenerate fixtures/conformance/support-report.golden.json)"
    );
}

/// **(T15.5-remainder)** Failure-mode shape: the **negative-capabilities** array is present, non-empty,
/// and every entry names its `enforced_by` guard (a non-claim is never decorative). This is a *restraint*
/// surface — its presence in the golden is itself part of the contract.
#[test]
fn failure_mode_negative_capabilities_are_present_and_guarded() {
    let j = normalized_support_report_json();
    assert!(j.contains("\"negative_capabilities\""));
    assert!(j.contains("\"enforced_by\""));
    // A representative restraint that must always be advertised.
    assert!(j.contains("does_not_claim_full_toctou_resistance"));
}

/// **(T15.5-remainder)** Failure-mode shape: report provenance is honest about what it is **not** — an
/// unsigned local report with `unknown` workspace provenance (no git tree / no `build.rs`). The engine
/// never fabricates clean/signed provenance; the golden pins these honest-failure values.
#[test]
fn failure_mode_report_provenance_is_honestly_unsigned_and_unknown() {
    let j = normalized_support_report_json();
    assert!(j.contains("\"report_provenance\": \"unsigned_local_report\""));
    assert!(j.contains("\"workspace_provenance\": \"unknown\""));
    // And never an unbounded verdict.
    assert!(!j.contains("\"conformant\": true"));
    assert!(!j.contains("\"conformance_level\": \"compatible\""));
}
