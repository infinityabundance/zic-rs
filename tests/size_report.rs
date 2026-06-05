//! T21.2 — `size-report`: the read-only bundle footprint + deterministic `bundle_hash`.
//!
//! Builds a tiny output tree (two real TZif zones + a non-TZif file + a symlink on Unix), measures it,
//! and pins the load-bearing properties: structural classification (tzif vs other vs symlink), the
//! version-histogram totals, and — the reproducibility claim the image builder depends on — that
//! `bundle_hash` is **deterministic** (same tree → same hash) and **sensitive** (any byte change → a
//! different hash). A missing tree is a config error, not a panic.

use std::path::PathBuf;

use tzcompile::size_report::{run_size_report, SizeReportOptions};

const DST_SRC: &str = "Rule R 1990 2030 - Mar lastSun 2:00 1:00 D\n\
                       Rule R 1990 2030 - Oct lastSun 2:00 0 S\n\
                       Zone Test/Z -5:00 R E%sT\n";
const FIXED_SRC: &str = "Zone Test/Fixed 5:00 - +05\n";

/// Build a fresh, uniquely-named temp dir.
fn tmpdir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "zic-rs-sizetest-{}-{}-{}",
        tag,
        std::process::id(),
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Compile `name` from `src` to real TZif bytes.
fn tzif(src: &str, name: &str) -> Vec<u8> {
    let dir = tmpdir("src");
    let p = dir.join("in.zi");
    std::fs::write(&p, src).unwrap();
    let db = tzcompile::load_database(&[p]).unwrap();
    tzcompile::compile_zone_to_bytes(&db, name).unwrap()
}

/// Populate a small output tree: `Test/Z`, `Test/Fixed`, a non-TZif `manifest.json`.
fn build_tree() -> PathBuf {
    let root = tmpdir("tree");
    std::fs::create_dir_all(root.join("Test")).unwrap();
    std::fs::write(root.join("Test/Z"), tzif(DST_SRC, "Test/Z")).unwrap();
    std::fs::write(root.join("Test/Fixed"), tzif(FIXED_SRC, "Test/Fixed")).unwrap();
    std::fs::write(root.join("manifest.json"), b"{\"schema\":\"x\"}\n").unwrap();
    root
}

#[test]
fn classifies_tzif_other_and_totals() {
    let root = build_tree();
    let r = run_size_report(&SizeReportOptions { out: root }).unwrap();
    assert_eq!(r.tzif_files, 2, "two real TZif zones");
    assert_eq!(r.other_files, 1, "manifest.json is not TZif");
    // every TZif file landed in the version histogram (our zones are v2/v3, never unclassified).
    let hist: u64 = r.version_histogram.iter().sum();
    assert_eq!(hist, r.tzif_files, "every TZif file is version-classified");
    assert!(r.total_tzif_bytes > 0 && r.total_bytes >= r.total_tzif_bytes);
    assert!(r.largest_tzif.is_some());
    assert_eq!(r.bundle_hash.len(), 64, "bundle_hash is a hex SHA-256");
}

#[test]
fn bundle_hash_is_deterministic_and_sensitive() {
    let root = build_tree();
    let h1 = run_size_report(&SizeReportOptions { out: root.clone() })
        .unwrap()
        .bundle_hash;
    // Same tree, measured again → identical hash (order-independent: sorted before hashing).
    let h2 = run_size_report(&SizeReportOptions { out: root.clone() })
        .unwrap()
        .bundle_hash;
    assert_eq!(h1, h2, "same tree must yield the same bundle_hash");
    // Change one byte of one file → the hash must change (it witnesses the whole tree).
    std::fs::write(root.join("manifest.json"), b"{\"schema\":\"y\"}\n").unwrap();
    let h3 = run_size_report(&SizeReportOptions { out: root })
        .unwrap()
        .bundle_hash;
    assert_ne!(h1, h3, "a changed file must change the bundle_hash");
}

#[test]
fn missing_tree_is_config_error_not_panic() {
    let missing = std::env::temp_dir().join(format!("zic-rs-size-absent-{}", std::process::id()));
    let err = run_size_report(&SizeReportOptions { out: missing });
    assert!(
        err.is_err(),
        "measuring a non-existent tree is a config error"
    );
}

#[cfg(unix)]
#[test]
fn symlink_is_counted_as_a_link_not_followed() {
    let root = build_tree();
    // a symlink alias `UTC` → `Test/Z` is a link, not a second TZif file.
    std::os::unix::fs::symlink("Test/Z", root.join("UTC")).unwrap();
    let r = run_size_report(&SizeReportOptions { out: root }).unwrap();
    assert_eq!(r.symlink_links, 1, "the symlink is one link");
    assert_eq!(
        r.tzif_files, 2,
        "the symlink is NOT followed into a 3rd TZif file"
    );
}

#[test]
fn json_carries_schema_and_non_claim() {
    let root = build_tree();
    let json = run_size_report(&SizeReportOptions { out: root })
        .unwrap()
        .to_json();
    assert!(json.contains("\"schema\": \"zic-rs-size-report-v1\""));
    assert!(json.contains("\"bundle_hash\""));
    assert!(json.contains("non_claim"), "the non-claim must be emitted");
}
