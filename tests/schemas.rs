//! T17.7 — schema registry / drift guard (dep-free).
//!
//! The core crate carries **no JSON-Schema validator dependency** (no-new-deps posture), so full
//! instance-validation (emit a report → validate against its schema) is an audit-suite (T23) task. What
//! *is* gated here, with std only, is the **registry consistency + drift**: every `schema` id the code
//! emits/ingests has a published `schemas/<id>.schema.json`; every schema file maps to a real emitter;
//! and each id literal is still present in its emitter source, so bumping an emitter without updating the
//! registry + schema file fails this test. See `schemas/README.md` for the fidelity scope.

use std::fs;
use std::path::PathBuf;

/// The authoritative registry: (schema id, the source file that emits/ingests it).
const REGISTRY: &[(&str, &str)] = &[
    ("zic-rs-compile-manifest-v8", "src/manifest.rs"),
    ("zic-rs-alias-map-v1", "src/manifest.rs"),
    ("zic-rs-support-report-v4", "src/report.rs"),
    ("zic-rs-structural-report-v3", "src/structural.rs"),
    ("zic-rs-semantic-report-v1", "src/semantic_witness.rs"),
    ("zic-rs-tzif-validation-v1", "src/tzif/rfc9636.rs"),
    ("zic-rs-aux-table-validation-v1", "src/aux_tables.rs"),
    ("zic-rs-release-diff-v1", "src/release_diff.rs"),
    ("zic-rs-doctor-v2", "src/doctor.rs"),
    ("zic-rs-size-report-v1", "src/size_report.rs"),
    ("vendor-oracle-receipt-v1", "src/vendor_oracle.rs"),
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn every_schema_id_is_emitted_and_published_no_drift() {
    for (id, src) in REGISTRY {
        // Drift guard: the id literal must still appear in its emitter source. A bump (…-v4 → …-v5)
        // removes the old literal → this fails until the registry + schema file are updated together.
        let src_txt = fs::read_to_string(root().join(src))
            .unwrap_or_else(|e| panic!("cannot read emitter source {src}: {e}"));
        assert!(
            src_txt.contains(id),
            "schema id {id:?} not found in {src} — emitter bumped without updating the registry/schema?"
        );
        // A published schema file exists and declares the id.
        let path = root().join("schemas").join(format!("{id}.schema.json"));
        let schema_txt = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("missing published schema {}: {e}", path.display()));
        assert!(
            schema_txt.contains(id),
            "{} does not declare its own id {id:?}",
            path.display()
        );
    }
}

#[test]
fn no_orphan_schema_files() {
    // Every schemas/*.schema.json must map to a registry entry (no published schema without an emitter).
    let dir = root().join("schemas");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if let Some(id) = name.strip_suffix(".schema.json") {
            assert!(
                REGISTRY.iter().any(|(rid, _)| rid == &id),
                "orphan schema file {name:?} (no emitter in the registry)"
            );
        }
    }
}

#[test]
fn registry_ids_are_unique() {
    use std::collections::BTreeSet;
    let ids: BTreeSet<&str> = REGISTRY.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids.len(),
        REGISTRY.len(),
        "duplicate schema id in the registry"
    );
}
