//! Tests for `support-report` — the frontier-map command.
//!
//! The crafted fixture deliberately mixes a supported zone with one zone per unsupported bucket
//! and a spread of link outcomes, so the bucketing, the canonical-vs-link accounting, and the
//! exact accounting invariant (`supported + Σ unsupported == zones parsed`) are all pinned. A
//! separate auto-skipping smoke test runs the report over the real installed `tzdata.zi`.

use std::path::PathBuf;

use tzcompile::load_database;
use tzcompile::report::build_support_report;

/// Write `src` to a temp `.zi` and build a support report from it.
fn report_of(src: &str) -> (tempfile::TempDir, tzcompile::report::SupportReport) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.zi");
    std::fs::write(&p, src).unwrap();
    let db = load_database(&[p]).expect("parses");
    let r = build_support_report(&db, Some("test-1".to_string()));
    (dir, r)
}

// Two supported zones (incl. a *negative* inline SAVE — now supported, law 7) + one zone per
// still-unsupported bucket, plus a spread of link outcomes. (`%z` no-rules and negative inline SAVE
// both COMPILE now; the unsupported examples use `%s`, which still fails closed.)
const MIXED: &str = "\
# version test-1
Zone Test/OK 0 - UTC
Zone Test/Neg 8:00 -1:00 XYZ
Zone Test/NS 5:00 - E%sT
Zone Test/PS 8:00 1:00 E%sT
Link Test/OK Test/Alias
Link Test/NS Test/AliasBad
Link Nowhere Test/Missing
Link Test/CycB Test/CycA
Link Test/CycA Test/CycB
";

#[test]
fn buckets_supported_and_each_unsupported_reason() {
    let (_d, r) = report_of(MIXED);
    assert_eq!(r.zones_parsed, 4);
    // `Test/Neg` (negative inline SAVE) now compiles — law 7 (signed SAVE).
    assert_eq!(r.supported_zones, vec!["Test/Neg", "Test/OK"]);
    // One zone per still-unsupported reason; two distinct buckets (both `%s`).
    assert_eq!(
        r.unsupported.len(),
        2,
        "buckets: {:?}",
        r.unsupported.keys().collect::<Vec<_>>()
    );
    let labels: Vec<&str> = r.unsupported.keys().map(String::as_str).collect();
    assert!(
        labels.iter().any(|l| l.contains("no-rules era")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|l| l.contains("%s or STD/DST slash")),
        "{labels:?}"
    );
}

#[test]
fn accounting_invariant_every_zone_in_exactly_one_bucket() {
    let (_d, r) = report_of(MIXED);
    assert!(r.is_fully_accounted());
    assert_eq!(
        r.supported_zones.len() + r.unsupported_zone_count(),
        r.zones_parsed
    );
}

#[test]
fn links_accounted_separately_from_zones() {
    let (_d, r) = report_of(MIXED);
    assert_eq!(r.links_parsed, 5);
    assert_eq!(r.links.to_supported, vec!["Test/Alias"]); // → Test/OK (supported)
    assert_eq!(r.links.to_unsupported, vec!["Test/AliasBad"]); // → Test/NS (unsupported)
    assert_eq!(r.links.missing, vec!["Test/Missing"]); // target "Nowhere" is nothing
    assert_eq!(r.links.cycles, vec!["Test/CycA", "Test/CycB"]);
    // Total supported identifiers = supported zones (2: Test/OK + Test/Neg) + links to supported (1).
    assert_eq!(r.supported_identifiers(), 3);
    assert_eq!(r.identifiers(), 9);
}

#[test]
fn largest_bucket_points_at_the_biggest_unlock() {
    // Two zones in the no-rules-%s bucket, one in inline-save-%s → no-rules is strictly largest.
    let (_d, r) = report_of("Zone A 5:00 - E%sT\nZone B 6:00 - E%sT\nZone C 8:00 1:00 E%sT\n");
    let (label, n) = r.largest_bucket().expect("a bucket");
    assert!(label.contains("no-rules era"), "{label}");
    assert_eq!(n, 2);
}

#[test]
fn text_report_surfaces_accounting_and_contract_note() {
    let (_d, r) = report_of(MIXED);
    let t = r.to_text();
    assert!(
        t.contains("COMPILE support"),
        "must state it is compile-support, not correctness"
    );
    assert!(t.contains("accounting:") && t.contains("[OK]"));
    assert!(t.contains("biggest unlock"));
    assert!(t.contains("tzdb release: test-1"));
}

#[test]
fn json_report_is_shaped_and_self_describing() {
    let (_d, r) = report_of(MIXED);
    let j = r.to_json();
    assert!(j.contains("\"schema\": \"zic-rs-support-report-v4\""));
    assert!(j.contains("\"zones_parsed\": 4"));
    assert!(j.contains("\"fully_accounted\": true"));
    assert!(j.contains("\"tzdb_version\": \"test-1\""));
    // T12.6 — the static provenance/capability block: manifest schema + source-variant pin gate.
    // T12.5a.2 lifted the gate (2026b reference admitted + signature-verified + pinned); the
    // reference is admitted but no variant *behaviour* is implemented yet.
    assert!(j.contains("\"provenance\""));
    assert!(j.contains("\"manifest_schema\": \"zic-rs-compile-manifest-v8\""));
    assert!(j.contains("\"source_variant_reference_pin_gate\": \"lifted_for_2026b\""));
    assert!(j.contains("\"source_variant_behavior_implemented\": false"));
    assert!(j.contains("\"to_supported\": [\"Test/Alias\"]"));
}

/// `--explain-buckets` (T6#3): each unsupported bucket maps to its deep `zic` law via the
/// `deep_semantic` audit map shared with `docs/zic-deep-semantics.md`. The plain `to_text()` stays
/// free of law lines (so the existing text shape is unchanged); the explained form and the JSON
/// `deep_semantic` field carry the law. The unit assertions exercise the full map (laws 7/9/10);
/// MIXED's live buckets are both law 9 (`%s` FORMAT paths) now that negative SAVE compiles.
#[test]
fn explain_buckets_maps_each_bucket_to_a_deep_law() {
    use tzcompile::report::deep_semantic;

    // The map itself (single source of truth, also used by the docs).
    assert!(
        deep_semantic("ZIC001_UNSUPPORTED_DIRECTIVE: inline-save: negative SAVE")
            .unwrap()
            .starts_with("law 7")
    );
    assert!(
        deep_semantic("ZIC001_UNSUPPORTED_DIRECTIVE: recurring footer: non-POSIX day form")
            .unwrap()
            .starts_with("law 10")
    );
    assert!(deep_semantic(
        "ZIC001_UNSUPPORTED_DIRECTIVE: no-rules era: %s or STD/DST slash FORMAT"
    )
    .unwrap()
    .starts_with("law 9"));
    assert!(deep_semantic("ZIC042_SOMETHING_UNMAPPED").is_none());

    let (_d, r) = report_of(MIXED);

    // Plain text is unchanged (no law lines); explained text adds them. MIXED's two unsupported
    // buckets (no-rules `%s`, inline-save `%s`) both map to law 9 (FORMAT paths).
    assert!(!r.to_text().contains("deep law:"));
    let ex = r.to_text_explained();
    assert!(
        ex.contains("↳ deep law: law 9"),
        "the `%s` buckets → law 9:\n{ex}"
    );
    // JSON always carries the field.
    assert!(
        r.to_json().contains("\"deep_semantic\": \"law 9"),
        "{}",
        r.to_json()
    );
}

/// Smoke test against the real installed source — auto-skips if absent (keeps CI portable).
#[test]
fn frontier_map_over_installed_tzdata_zi() {
    let sys = PathBuf::from("/usr/share/zoneinfo/tzdata.zi");
    if !sys.exists() {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    }
    let db = load_database(&[sys]).expect("installed tzdata.zi parses");
    let r = build_support_report(&db, Some("installed".to_string()));
    assert!(
        r.zones_parsed > 300,
        "expected >300 zones, got {}",
        r.zones_parsed
    );
    assert!(
        r.links_parsed > 200,
        "expected >200 links, got {}",
        r.links_parsed
    );
    assert!(r.is_fully_accounted(), "every zone must be accounted");
    assert!(!r.supported_zones.is_empty());
}
