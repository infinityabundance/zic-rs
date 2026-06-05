//! T15.1 — conformance-report **schema shape** witness (the inventory's executable twin).
//!
//! Pins the *current* public `support-report` (`zic-rs-support-report-v4`) JSON contract so that further
//! additions are **deliberate** schema changes, not accidental drift. The born-typed fields flipped from
//! absent→present on their substeps: `oracle_mode` + `negative_capabilities` (T15.2), `conformance_status`
//! (T15.5). This
//! is the conformance engine's baseline; no behaviour change here — it only observes
//! `build_support_report(...).to_json()`. See `docs/zic-conformance-engine.md`.
//!
//! It asserts *presence + the schema id*, not the full byte layout — wording/ordering inside the report
//! may evolve, but the **field contract** and the **schema version** are the stable public surface.

use tzcompile::load_database;
use tzcompile::report::build_support_report;

const SRC: &str = "Zone Etc/UTC 0:00 - UTC\nLink Etc/UTC UTC\n";

fn support_report_json() -> String {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, SRC).unwrap();
    let db = load_database(std::slice::from_ref(&p)).unwrap();
    build_support_report(&db, Some("test-1".to_string())).to_json()
}

#[test]
fn support_report_contract_is_present() {
    let j = support_report_json();

    // Schema id — the stable public version string (bumped v2→v3 in T15.2 for the additive
    // `oracle_mode` + `negative_capabilities` fields).
    assert!(
        j.contains("\"schema\": \"zic-rs-support-report-v4\""),
        "support-report schema id missing/changed:\n{j}"
    );

    // The top-level field contract (T15.1 inventory). Each is a claim-bearing surface the conformance
    // engine exposes; a removal/rename is a deliberate schema change, caught here.
    for field in [
        "\"provenance\"",
        "\"tzdb_version\"",
        "\"zones_parsed\"",
        "\"links_parsed\"",
        "\"supported_identifiers\"",
        "\"fully_accounted\"",
        "\"supported_zones\"",
        "\"unsupported\"",
        "\"links\"",
    ] {
        assert!(
            j.contains(field),
            "support-report missing field {field}:\n{j}"
        );
    }

    // The link-accounting sub-contract (the four distinct categories must stay distinct).
    for link_field in [
        "\"to_supported\"",
        "\"to_unsupported\"",
        "\"cycles\"",
        "\"missing\"",
    ] {
        assert!(
            j.contains(link_field),
            "support-report links.{link_field} missing:\n{j}"
        );
    }
}

#[test]
fn provenance_block_contract_is_present() {
    let j = support_report_json();
    // The shared provenance/capability block (T12.6) — the static admission/gate surface every report
    // carries. These are the fields T15.5 will join with the one-line machine status + ReferencePinGate.
    for field in [
        "\"manifest_schema\"",
        "\"source_variant_reference_pin_gate\"",
        "\"blocked_substeps\"",
        "\"unpinned_required_files\"",
        "\"source_variant_behavior_implemented\"",
    ] {
        assert!(j.contains(field), "provenance block missing {field}:\n{j}");
    }
    // The manifest schema the reports point at (provenance single-sources it).
    assert!(
        j.contains("\"zic-rs-compile-manifest-v8\""),
        "manifest schema id missing/changed:\n{j}"
    );
}

#[test]
fn born_typed_fields_present_through_t15_5() {
    // The "witness flips on purpose" discipline: T15.2 added `oracle_mode` + `negative_capabilities`,
    // T15.5 added `conformance_status` — all now PRESENT (the T15.1 baseline asserted them absent; each
    // flip is a deliberate, visible schema change, recorded with its substep).
    let j = support_report_json();
    assert!(
        j.contains("\"oracle_mode\""),
        "T15.2 oracle_mode missing:\n{j}"
    );
    assert!(
        j.contains("\"negative_capabilities\""),
        "T15.2 negative_capabilities missing:\n{j}"
    );
    assert!(
        j.contains("\"conformance_status\""),
        "T15.5 conformance_status missing:\n{j}"
    );
}

/// T15.5 — the conformance rollup is **bounded** (scope, not ambition) and is the report's own claim
/// envelope: a bounded level (never `compatible`), a `declared_scope_hash`, honest provenance, and the
/// report kind. The report is itself a claim-bearing artifact.
#[test]
fn conformance_status_is_bounded_and_carries_provenance() {
    let j = support_report_json();
    // Bounded level — support-report establishes compile-coverage over an admitted release, nothing more.
    assert!(
        j.contains("\"conformance_level\": \"release_admitted_compile_coverage\""),
        "{j}"
    );
    // Never an unbounded "compatible"/"conformant: true".
    assert!(!j.contains("\"conformant\": true"));
    assert!(!j.contains("\"conformance_level\": \"compatible\""));
    // Report-as-artifact provenance + a claim-envelope hash.
    for field in [
        "\"report_kind\": \"support\"",
        "\"declared_scope_hash\"",
        "\"workspace_provenance\"",
        "\"report_provenance\": \"unsigned_local_report\"",
        "\"compiler_identity\"",
        "\"available_surfaces\"",
    ] {
        assert!(
            j.contains(field),
            "conformance_status missing {field}:\n{j}"
        );
    }
}

/// T15.5-remainder — the conformance rollup now carries the typed **claim-shape** axes, so the *shape*
/// of the claim (not just its result) is machine-readable: `claim_portability` · `evidence_authority` ·
/// a `claim_boundary` (proves / does_not_prove / depends_on) · the `valid_disambiguation` array (the
/// distinct senses of "valid", impossible to blur). The pin gate renders the typed `ReferencePinGate`.
#[test]
fn conformance_status_carries_typed_claim_shape() {
    let j = support_report_json();
    for field in [
        "\"claim_portability\": \"release_specific\"",
        "\"evidence_authority\": \"implementation_observation\"",
        "\"claim_boundary\"",
        "\"proves\"",
        "\"does_not_prove\"",
        "\"depends_on\"",
        "\"valid_disambiguation\"",
        // The gate is rendered from the typed ReferencePinGate (same literal as the shared block).
        "\"admitted_release_gate\": \"lifted_for_2026b\"",
    ] {
        assert!(
            j.contains(field),
            "conformance_status missing {field}:\n{j}"
        );
    }
    // The valid-disambiguation must keep the live behaviour claim *separate* from structural validity.
    assert!(j.contains("structurally_valid:") && j.contains("behaviour_matched:"));
}

/// T15.5-remainder — `ReferencePinGate` is the typed owner of the admission-gate vocabulary; it renders
/// the *same* literal as the legacy `SOURCE_VARIANT_GATE_STATUS` string (a drift guard, the boundary-shim
/// discipline reused from `OracleMode`).
#[test]
fn reference_pin_gate_does_not_drift_from_legacy_const() {
    use tzcompile::manifest::{ReferencePinGate, SOURCE_VARIANT_GATE_STATUS};
    assert_eq!(
        ReferencePinGate::current().as_str(),
        SOURCE_VARIANT_GATE_STATUS,
        "the typed gate must render the same literal as the shared provenance const"
    );
    // Totality: the two variants render distinct, stable ids.
    assert_eq!(ReferencePinGate::Open.as_str(), "open");
    assert_eq!(
        ReferencePinGate::LiftedFor2026b.as_str(),
        "lifted_for_2026b"
    );
}

/// T16.5-core — `declared_scope_hash` **tracks the negative-capability set** (the sorted non-claim ids
/// are part of the hashed envelope). This pins the design property behind the expected golden change when
/// a `NegativeCapability` is added: the new id appears in the emitted (sorted) array, and the hash is a
/// deterministic 64-hex function of that envelope — so adding/removing a non-claim *must* move the hash.
#[test]
fn declared_scope_hash_tracks_negative_capability_set() {
    use tzcompile::manifest::{ConformanceStatus, NEGATIVE_CAPABILITIES};
    // The set that feeds the hash is non-empty, sorted, and contains the T16.5 QEMU-external non-claim.
    let ids: Vec<&str> = NEGATIVE_CAPABILITIES.iter().map(|n| n.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(
        ids, sorted,
        "the hashed non-claim set must be canonically sorted"
    );
    assert!(ids.contains(&"does_not_ship_or_operate_vendor_qemu_labs_in_core_repo"));
    // The hash is a deterministic 64-hex function of the (fixed) envelope.
    let h = ConformanceStatus::support().declared_scope_hash();
    assert_eq!(h.len(), 64);
    assert_eq!(h, ConformanceStatus::support().declared_scope_hash());
    // And it is reflected in the emitted support-report alongside the non-claims array.
    let j = support_report_json();
    assert!(j.contains("does_not_ship_or_operate_vendor_qemu_labs_in_core_repo"));
    assert!(j.contains(&format!("\"declared_scope_hash\": \"{h}\"")));
}

/// T15.5 — `declared_scope_hash` is deterministic for a fixed claim envelope (a reviewer can pin a claim
/// to it; it changes only when a scope element changes).
#[test]
fn declared_scope_hash_is_deterministic() {
    use tzcompile::manifest::ConformanceStatus;
    let a = ConformanceStatus::support().declared_scope_hash();
    let b = ConformanceStatus::support().declared_scope_hash();
    assert_eq!(a, b, "scope hash must be stable for a fixed envelope");
    assert_eq!(a.len(), 64, "sha-256 hex");
}

/// T15.2 — `oracle_mode` is typed and **visible**: `support-report` makes no oracle-backed claim, so it
/// honestly renders `not_run` (with no `skipped_with_reason`), never silence.
#[test]
fn support_report_oracle_mode_is_not_run_and_visible() {
    let j = support_report_json();
    assert!(
        j.contains("\"oracle_mode\": { \"mode\": \"not_run\", \"skipped_with_reason\": null }"),
        "support-report oracle_mode shape wrong:\n{j}"
    );
}

/// T15.2 — `negative_capabilities` is a sorted, guard-bearing array (each entry names what enforces it).
#[test]
fn negative_capabilities_are_canonical_sorted_and_guarded() {
    use tzcompile::manifest::{NegativeCapability, NEGATIVE_CAPABILITIES};
    // Non-empty, sorted by the rendered id, unique, and every entry has a non-empty enforcing reference.
    assert!(!NEGATIVE_CAPABILITIES.is_empty());
    let ids: Vec<&str> = NEGATIVE_CAPABILITIES.iter().map(|n| n.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "negative_capabilities must be sorted by id");
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        ids.len(),
        "negative_capabilities must be unique"
    );
    for nc in NEGATIVE_CAPABILITIES {
        assert!(
            !NegativeCapability::enforced_by(*nc).is_empty(),
            "{} has no enforcing guard/test/receipt — a non-claim must not be decorative",
            nc.as_str()
        );
    }
    // And the array is actually emitted in the report.
    let j = support_report_json();
    assert!(j.contains("does_not_claim_full_toctou_resistance"));
    assert!(j.contains("\"enforced_by\""));
}

/// T15.2 — oracle **absence is visible**: the `Unavailable` mode renders a non-null `skipped_with_reason`
/// (the rule that a verdict can never *silently* weaken when reference tools are missing — used by T15.3).
#[test]
fn oracle_absence_renders_skipped_with_reason() {
    use tzcompile::manifest::OracleMode;
    let j = OracleMode::Unavailable("zdump not found on PATH".to_string()).to_json_field();
    assert!(j.contains("\"mode\": \"unavailable\""), "{j}");
    assert!(
        j.contains("\"skipped_with_reason\": \"zdump not found on PATH\""),
        "oracle absence must render a reason, never null/silence:\n{j}"
    );
    // And the present modes carry no skip reason.
    assert!(OracleMode::ReferenceZic.skipped_with_reason().is_none());
    assert_eq!(OracleMode::NotRun.mode_str(), "not_run");
}
