//! Tests for the producer-side artifacts (T3.4b/c): the alias/canonical map and the
//! compile-provenance manifest. These pin the schema, the zone-vs-link accounting + hashes,
//! the honest generation-option reporting, and — critically — that a `compile`-only run never
//! claims an oracle match it did not perform.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tzcompile::compile::plan;
use tzcompile::manifest::{self, AliasEntry, AliasMap, SourceVariantArgs};
use tzcompile::{
    load_database, CompileConfig, LinkMode, UnsupportedPolicy, ZoneSelection,
    DEFAULT_TRANSITION_LIMIT,
};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Compile `utc.zi` (zone `Etc/UTC` + link `UTC`) into a tempdir; return report + dir + inputs +
/// the [`CompileConfig`] + the parsed [`Database`] (the manifest builder needs the config for the
/// build-profile identity and the db for the link/alias profile).
fn compile_utc() -> (
    tzcompile::CompileReport,
    tempfile::TempDir,
    Vec<PathBuf>,
    CompileConfig,
    tzcompile::model::Database,
) {
    let inputs = vec![fixture("fixtures/minimal/utc.zi")];
    let db = load_database(&inputs).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let config = CompileConfig {
        input_paths: inputs.clone(),
        output_dir: dir.path().to_path_buf(),
        zones: ZoneSelection::One("Etc/UTC".to_string()),
        link_mode: LinkMode::Copy,
        overwrite: true,
        unsupported_policy: UnsupportedPolicy::Error,
        transition_limit: DEFAULT_TRANSITION_LIMIT,
        emit_style: tzcompile::EmitStyle::Default,
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
    (report, dir, inputs, config, db)
}

#[test]
fn alias_map_counts_zone_and_link() {
    let (report, dir, _, _, _) = compile_utc();
    let m = manifest::build(&report, dir.path()).unwrap();
    assert_eq!(m.identifiers, 2);
    assert_eq!(m.canonical_zones, 1);
    assert_eq!(m.links, 1);
    // The link was materialised as a copy → its bytes are duplicated (the jiff#258 figure).
    assert_eq!(m.duplicated_byte_links, 1);
}

#[test]
fn alias_map_resolves_link_target_hash() {
    let (report, dir, _, _, _) = compile_utc();
    let m = manifest::build(&report, dir.path()).unwrap();

    let zone_sha = match m.entries.get("Etc/UTC").unwrap() {
        AliasEntry::Zone { sha256 } => sha256.clone(),
        _ => panic!("Etc/UTC should be a zone"),
    };
    match m.entries.get("UTC").unwrap() {
        AliasEntry::Link {
            target,
            target_sha256,
            materialised,
        } => {
            assert_eq!(target, "Etc/UTC");
            // A copied link duplicates the target's exact bytes → identical hash.
            assert_eq!(target_sha256, &zone_sha);
            assert_eq!(*materialised, tzcompile::LinkMode::Copy);
        }
        _ => panic!("UTC should be a link"),
    }
    // sha256 is a 64-char lowercase hex string.
    assert_eq!(zone_sha.len(), 64);
    assert!(zone_sha
        .bytes()
        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
}

#[test]
fn manifest_records_compiled_zones_and_links() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let requested = vec!["Etc/UTC".to_string()];
    let m = manifest::build_compile_manifest(
        &requested,
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m.zones_requested, vec!["Etc/UTC"]);
    assert_eq!(m.zones_compiled, vec!["Etc/UTC"]);
    assert_eq!(m.links_materialized, vec!["UTC"]);
    assert!(m.unsupported_zones.is_empty());
    assert_eq!(m.zic_rs_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(m.tzdb.source_sha256.len(), 64);
}

#[test]
fn manifest_records_build_profile_of_this_run() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let json = m.to_json();
    // Build-profile describes THIS run: ordinary compile → posix tree, no leaps, copy links, default
    // emit. Not-yet-detected axes are explicit `"unknown"` — never an aspirational "unsupported".
    assert_eq!(m.build_profile.output_tree.as_str(), "posix");
    assert_eq!(m.build_profile.leap_source.mode.as_str(), "none");
    assert_eq!(m.build_profile.emit_style, tzcompile::EmitStyle::Default);
    assert_eq!(m.build_profile.link_mode, tzcompile::LinkMode::Copy);
    assert!(json.contains("\"output_tree\": \"posix\""));
    assert!(
        json.contains("\"mode\": \"none\""),
        "leap_source mode none, not 'unsupported'"
    );
    // As of T12.5d, `build_profile` carries NO source-variant placeholders: `backward` (T12.4d),
    // `backzone` (T12.5b), `PACKRATLIST` (T12.5c) and `DATAFORM`=`rearguard`/`vanguard` (T12.5d) are all
    // authoritative `source_profile` evidence axes. None appears as a `build_profile` `"unknown"` stub.
    assert!(!json.contains("\"backzone\": \"unknown\""));
    assert!(!json.contains("\"rearguard\": \"unknown\""));
    assert!(!json.contains("\"vanguard\": \"unknown\""));
    assert_eq!(m.source_profile.backzone.status(), "unknown_no_evidence");
    assert_eq!(m.source_profile.dataform.status(), "unknown_no_evidence");
    // No field *value* is aspirational `"unsupported"` (the `"unsupported_zones"` key is fine — it is
    // a real per-run accounting list, not a capability claim).
    assert!(
        !json.contains(": \"unsupported\""),
        "manifest describes the run, not capabilities"
    );
}

/// Detected vs claimed tzdb version: the manifest never silently stamps a release.
#[test]
fn manifest_reconciles_detected_and_claimed_version() {
    let (report, _dir, inputs, config, db) = compile_utc();
    // `utc.zi` has no `# version` header → detected is None; with no claim, status is "unknown".
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m.tzdb.detected_version, None);
    assert_eq!(m.tzdb.version_status(), "unknown");
    // A user claim with no detection → "claimed_only" (recorded, not silently trusted as detected).
    let m2 = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        Some("2026b"),
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m2.tzdb.claimed_version.as_deref(), Some("2026b"));
    assert_eq!(m2.tzdb.version_status(), "claimed_only");
}

#[test]
fn manifest_schema_is_stable() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert!(m
        .to_json()
        .contains("\"schema\": \"zic-rs-compile-manifest-v8\""));
    let am = manifest::build(&report, _dir.path()).unwrap();
    assert!(am.to_json().contains("\"schema\": \"zic-rs-alias-map-v1\""));
}

// ---------------------------------------------------------------------------------------------
// T12.3 — source-input identity: which files, in what order, with what hashes, under what kind.
// ---------------------------------------------------------------------------------------------

/// A single `.zi` input is recorded as `tzdata_zi`, one file at `order_index` 0, with a logical
/// (base)name — never a machine-local absolute path.
#[test]
fn manifest_records_single_file_input() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m.source_inputs.kind.as_str(), "tzdata_zi");
    assert_eq!(m.source_inputs.files.len(), 1);
    let f = &m.source_inputs.files[0];
    assert_eq!(f.order_index, 0);
    assert_eq!(f.logical_name, "utc.zi"); // basename, not the absolute fixture path
    assert!(
        !f.logical_name.contains('/'),
        "logical name is a portable basename"
    );
    assert_eq!(f.sha256.len(), 64);
    assert!(f.bytes > 0);
    // The order-sensitive aggregate hash is present and distinct from the empty digest.
    assert_eq!(m.source_inputs.aggregate_hash.len(), 64);
}

/// Multiple inputs are recorded **in input order** with monotone `order_index`, and the structural
/// kind is `multi_file`.
#[test]
fn manifest_records_multi_file_inputs_in_order() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    let ordered = vec![
        fixture("fixtures/minimal/utc.zi"),
        fixture("fixtures/minimal/fixed.zi"),
        fixture("fixtures/minimal/euro.zi"),
    ];
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &ordered,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m.source_inputs.kind.as_str(), "multi_file");
    let names: Vec<&str> = m
        .source_inputs
        .files
        .iter()
        .map(|f| f.logical_name.as_str())
        .collect();
    assert_eq!(names, vec!["utc.zi", "fixed.zi", "euro.zi"]); // faithful input order
    for (i, f) in m.source_inputs.files.iter().enumerate() {
        assert_eq!(f.order_index, i);
    }
}

/// **Source order is part of the build identity.** Reordering the same files changes the
/// order-sensitive `aggregate_hash`, while the explicitly-canonicalized (sorted) `source_sha256`
/// stays identical — the manifest neither conflates the two orderings nor pretends they differ in
/// content.
#[test]
fn manifest_aggregate_hash_changes_when_file_order_changes() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    let a = vec![
        fixture("fixtures/minimal/utc.zi"),
        fixture("fixtures/minimal/euro.zi"),
    ];
    let b = vec![
        fixture("fixtures/minimal/euro.zi"),
        fixture("fixtures/minimal/utc.zi"),
    ];
    let ma = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &a,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let mb = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &b,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_ne!(
        ma.source_inputs.aggregate_hash, mb.source_inputs.aggregate_hash,
        "reordering inputs must change the order-sensitive aggregate hash"
    );
    assert_eq!(
        ma.tzdb.source_sha256, mb.tzdb.source_sha256,
        "the sorted (canonicalized) content hash is order-independent by design"
    );
}

/// A single non-`.zi` file is `single_file` — the manifest does **not** guess "synthetic fixture"
/// or any richer label it cannot deterministically prove.
#[test]
fn manifest_single_non_zi_input_is_single_file_not_guessed() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    // `eastern.zi` would be `tzdata_zi`; a basename without the `.zi` extension is just one file.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("northamerica");
    std::fs::copy(fixture("fixtures/minimal/utc.zi"), &p).unwrap();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &[p],
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m.source_inputs.kind.as_str(), "single_file");
}

/// The structural `kind` distinguishes a single `tzdata.zi` from a multi-file source set — but it
/// is *form*, not membership.
#[test]
fn manifest_distinguishes_tzdata_zi_from_multi_file() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    let one = vec![fixture("fixtures/minimal/utc.zi")];
    let many = vec![
        fixture("fixtures/minimal/utc.zi"),
        fixture("fixtures/minimal/fixed.zi"),
    ];
    let m1 = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &one,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let mn = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &many,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m1.source_inputs.kind.as_str(), "tzdata_zi");
    assert_eq!(mn.source_inputs.kind.as_str(), "multi_file");
}

/// The `aggregate_hash` is **content-sensitive**: two different files produce different identities
/// (distinct from the order-sensitivity above).
#[test]
fn manifest_aggregate_hash_changes_when_bytes_change() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    let a = vec![fixture("fixtures/minimal/utc.zi")];
    let b = vec![fixture("fixtures/minimal/fixed.zi")];
    let ma = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &a,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let mb = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &b,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_ne!(
        ma.source_inputs.aggregate_hash,
        mb.source_inputs.aggregate_hash
    );
    assert_ne!(
        ma.source_inputs.files[0].sha256,
        mb.source_inputs.files[0].sha256
    );
}

/// **Absolute paths are not portable identity.** The same bytes at two different machine-local
/// paths yield the same per-file hash + aggregate identity; only the non-identity `source_path`
/// (environment context) differs.
#[test]
fn manifest_identity_is_path_independent() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    let d1 = tempfile::tempdir().unwrap();
    let d2 = tempfile::tempdir().unwrap();
    let p1 = d1.path().join("northamerica");
    let p2 = d2.path().join("northamerica");
    std::fs::copy(fixture("fixtures/minimal/utc.zi"), &p1).unwrap();
    std::fs::copy(fixture("fixtures/minimal/utc.zi"), &p2).unwrap();
    let m1 = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &[p1],
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let m2 = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &[p2],
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(
        m1.source_inputs.aggregate_hash,
        m2.source_inputs.aggregate_hash
    );
    assert_eq!(
        m1.source_inputs.files[0].sha256,
        m2.source_inputs.files[0].sha256
    );
    assert_eq!(
        m1.source_inputs.files[0].logical_name,
        m2.source_inputs.files[0].logical_name
    );
    // …while the environment-context path (deliberately NOT identity) differs.
    assert_ne!(m1.tzdb.source_path, m2.tzdb.source_path);
}

/// A synthetic `.zi` fixture is recorded by its structural form (`tzdata_zi`) but is **never
/// claimed as the upstream release**: with no `# version` header, `version_status` stays
/// `"unknown"`. Synthetic-vs-pristine is answered by detected/claimed version reconciliation, not
/// by guessing a `kind` we cannot prove.
#[test]
fn manifest_synthetic_fixture_not_claimed_as_upstream_release() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert_eq!(m.source_inputs.kind.as_str(), "tzdata_zi"); // structural form only
    assert_eq!(m.tzdb.detected_version, None); // utc.zi has no version header
    assert_eq!(m.tzdb.version_status(), "unknown"); // not claimed as any release
}

/// A file literally named `backzone` must **not** make the manifest claim a `backzone` source set.
/// Source-set membership is never inferred from a filename — it stays an honest `"unknown"`.
#[test]
fn manifest_does_not_infer_backzone_from_filename() {
    let (report, _dir, _inputs, config, db) = compile_utc();
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("backzone");
    std::fs::copy(fixture("fixtures/minimal/utc.zi"), &p).unwrap();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &[p],
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    // Recorded as a plain single file…
    assert_eq!(m.source_inputs.kind.as_str(), "single_file");
    // …and the membership axes are still explicit unknowns, never flipped by the name. `backward` and
    // `backzone` are now `source_profile` evidence axes (not build_profile stubs); a file *named*
    // `backzone` does not flip the `backzone` axis.
    let json = m.to_json();
    assert!(
        !json.contains("\"backward\": \"unknown\"") && !json.contains("\"backzone\": \"unknown\""),
        "backward/backzone are not build_profile stubs anymore"
    );
    assert_eq!(m.source_profile.backward.status(), "unknown_no_evidence");
    assert_eq!(m.source_profile.backzone.status(), "unknown_no_evidence");
}

// ---------------------------------------------------------------------------------------------
// T12.5a — source-variant policy inventory: the manifest never INFERS backzone/rearguard/vanguard
// from filenames, aliases, or link counts; and `backward` is gone from build_profile (it is the
// source_profile evidence axis). Reference-first inventory stage — no variant behaviour exists yet,
// so these are regression guards that keep the honest "unknown" classification from drifting.
// ---------------------------------------------------------------------------------------------

/// As of T12.5d, `build_profile` carries **no** source-variant placeholders: `backward` (T12.4d),
/// `backzone` (T12.5b), `PACKRATLIST` (T12.5c) and `DATAFORM`=`rearguard`/`vanguard` (T12.5d) are all
/// `source_profile` evidence axes. `build_profile` describes only *how this run emitted*.
#[test]
fn build_profile_carries_no_source_variant_placeholders() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let json = m.to_json();
    for stub in [
        "\"backward\": \"unknown\"",
        "\"backzone\": \"unknown\"",
        "\"rearguard\": \"unknown\"",
        "\"vanguard\": \"unknown\"",
    ] {
        assert!(!json.contains(stub), "build_profile must not carry {stub}");
    }
    // The encoding axis lives in source_profile and is honest-unknown for an ordinary compile.
    assert!(json.contains("\"dataform_evidence\""));
    assert_eq!(m.source_profile.dataform.status(), "unknown_no_evidence");
}

/// Source-variant membership/encoding is **never inferred from a filename**: inputs literally named
/// `vanguard.zi` / `rearguard.zi` / `backzone` (with non-matching content) leave the axes unknown — the
/// name proves nothing; only a hash-backed match or an explicit claim could. (DATAFORM detection reads
/// the file *hash*, so a mis-named file never flips it.)
#[test]
fn source_variants_not_inferred_from_filename() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["vanguard.zi", "rearguard.zi", "backzone"] {
        let p = dir.path().join(name);
        std::fs::copy(fixture("fixtures/minimal/utc.zi"), &p).unwrap();
        let (report, _o, inputs, config, db) =
            compile_sel(vec![p], ZoneSelection::AllSupported, LinkMode::Copy);
        let m = manifest_of(&report, &inputs, &config, &db);
        assert_eq!(
            m.source_profile.backzone.status(),
            "unknown_no_evidence",
            "{name}: backzone not from filename"
        );
        assert_eq!(
            m.source_profile.dataform.status(),
            "unknown_no_evidence",
            "{name}: dataform not from filename (hash, not name, decides)"
        );
    }
}

/// A rich alias/link surface (zones + aliases + a cyclic/failed link) does not flip any source-variant
/// axis — membership/encoding is never inferred from alias names or selected/omitted/failed counts.
#[test]
fn source_variants_not_inferred_from_aliases_or_link_counts() {
    let (report, _dir, inputs, config, db) = compile_sel(
        vec![fixture("fixtures/minimal/link_cycle.zi")],
        ZoneSelection::AllSupported,
        LinkMode::Copy,
    );
    let m = manifest_of(&report, &inputs, &config, &db);
    assert!(m.link_profile.links_failed_count > 0); // precondition: non-trivial link surface
    assert_eq!(m.source_profile.backzone.status(), "unknown_no_evidence");
    assert_eq!(m.source_profile.packratlist.status(), "unknown_no_evidence");
    assert_eq!(m.source_profile.dataform.status(), "unknown_no_evidence");
}

// ---------------------------------------------------------------------------------------------
// T12.4b — link / alias profile: counts + policy + stable hashes binding the build to its
// alias-map. Links are output identifiers, never source-set membership evidence.
// ---------------------------------------------------------------------------------------------

/// Compile `inputs` with selection `sel` and link `mode`; return everything the manifest builder
/// needs. (A general sibling of `compile_utc` for the multi-zone/link cases.)
fn compile_sel(
    inputs: Vec<PathBuf>,
    sel: ZoneSelection,
    mode: LinkMode,
) -> (
    tzcompile::CompileReport,
    tempfile::TempDir,
    Vec<PathBuf>,
    CompileConfig,
    tzcompile::model::Database,
) {
    let db = load_database(&inputs).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let config = CompileConfig {
        input_paths: inputs.clone(),
        output_dir: dir.path().to_path_buf(),
        zones: sel,
        link_mode: mode,
        overwrite: true,
        unsupported_policy: UnsupportedPolicy::Error,
        transition_limit: DEFAULT_TRANSITION_LIMIT,
        emit_style: tzcompile::EmitStyle::Default,
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
    (report, dir, inputs, config, db)
}

fn manifest_of(
    report: &tzcompile::CompileReport,
    inputs: &[PathBuf],
    config: &CompileConfig,
    db: &tzcompile::model::Database,
) -> manifest::CompileManifest {
    let requested = match &config.zones {
        ZoneSelection::One(z) => vec![z.clone()],
        ZoneSelection::Many(zs) => zs.clone(),
        ZoneSelection::AllSupported => db.zones.iter().map(|z| z.name.clone()).collect(),
    };
    manifest::build_compile_manifest(
        &requested,
        inputs,
        report,
        config,
        db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap()
}

/// `utc.zi` = 1 zone (`Etc/UTC`) + 1 link (`UTC` → it). Selecting the zone materialises the link;
/// nothing is omitted or failed.
#[test]
fn manifest_records_link_counts() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest_of(&report, &inputs, &config, &db);
    assert_eq!(m.link_profile.zones_compiled_count, 1);
    assert_eq!(m.link_profile.links_selected_count, 1);
    assert_eq!(m.link_profile.links_materialized_count, 1);
    assert_eq!(m.link_profile.links_omitted_count, 0);
    assert_eq!(m.link_profile.links_failed_count, 0);
}

/// The alias-map hash is recorded (64-hex) and appears in the JSON.
#[test]
fn manifest_records_alias_map_hash() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest_of(&report, &inputs, &config, &db);
    assert_eq!(m.link_profile.alias_map_sha256.len(), 64);
    assert!(m.to_json().contains("\"alias_map_sha256\""));
}

/// Same link set → same alias-map hash (the serialization is deterministic).
#[test]
fn alias_map_hash_is_stable_for_same_link_set() {
    let (r1, _d1, i1, c1, db1) = compile_utc();
    let (r2, _d2, i2, c2, db2) = compile_utc();
    let m1 = manifest_of(&r1, &i1, &c1, &db1);
    let m2 = manifest_of(&r2, &i2, &c2, &db2);
    assert_eq!(
        m1.link_profile.alias_map_sha256,
        m2.link_profile.alias_map_sha256
    );
}

/// A different materialised link set → a different alias-map hash. Selecting one zone of a
/// two-zone/two-link source materialises only that zone's link; `--all-supported` materialises
/// both → the alias maps (and their hashes) differ.
#[test]
fn alias_map_hash_changes_when_link_set_changes() {
    let inputs = vec![fixture("fixtures/minimal/links_two.zi")];
    let (r1, _d1, i1, c1, db1) = compile_sel(
        inputs.clone(),
        ZoneSelection::One("Etc/UTC".into()),
        LinkMode::Copy,
    );
    let (r2, _d2, i2, c2, db2) = compile_sel(inputs, ZoneSelection::AllSupported, LinkMode::Copy);
    let m1 = manifest_of(&r1, &i1, &c1, &db1);
    let m2 = manifest_of(&r2, &i2, &c2, &db2);
    assert_ne!(
        m1.link_profile.alias_map_sha256,
        m2.link_profile.alias_map_sha256
    );
}

/// Selected and omitted links are **distinct** sets, with distinct hashes. Selecting only
/// `Etc/UTC` from the two-zone/two-link source materialises `UTC` (selected) but excludes
/// `GMTLINK` (omitted because its target `Etc/GMT` was not compiled — a policy outcome, not an
/// error).
#[test]
fn selected_and_omitted_links_are_distinct() {
    let inputs = vec![fixture("fixtures/minimal/links_two.zi")];
    let (report, _dir, inputs, config, db) =
        compile_sel(inputs, ZoneSelection::One("Etc/UTC".into()), LinkMode::Copy);
    let m = manifest_of(&report, &inputs, &config, &db);
    assert_eq!(m.link_profile.links_selected_count, 1); // UTC
    assert_eq!(m.link_profile.links_omitted_count, 1); // GMTLINK
    assert_eq!(m.link_profile.links_failed_count, 0); // an omission is not a failure
    assert_ne!(
        m.link_profile.selected_links_sha256, m.link_profile.omitted_links_sha256,
        "selected and omitted link sets are different, so their hashes differ"
    );
}

/// The link policy (`copy`/`symlink`) is recorded as a fact of the run.
#[cfg(unix)]
#[test]
fn copy_vs_symlink_link_policy_recorded() {
    let inputs = vec![fixture("fixtures/minimal/utc.zi")];
    let (rc, _dc, ic, cc, dbc) = compile_sel(
        inputs.clone(),
        ZoneSelection::One("Etc/UTC".into()),
        LinkMode::Copy,
    );
    let (rs, _ds, is, cs, dbs) = compile_sel(
        inputs,
        ZoneSelection::One("Etc/UTC".into()),
        LinkMode::Symlink,
    );
    assert_eq!(
        manifest_of(&rc, &ic, &cc, &dbc).link_profile.link_policy,
        "copy"
    );
    assert_eq!(
        manifest_of(&rs, &is, &cs, &dbs).link_profile.link_policy,
        "symlink"
    );
}

/// The subtle trap: selecting a **link name** (not a canonical zone) compiles its
/// *target* and materialises the link. So `--zone UTC` yields a compiled `Etc/UTC` and a written
/// `UTC` link — not an empty build.
#[test]
fn link_name_selection_compiles_target_and_materializes_link() {
    let inputs = vec![fixture("fixtures/minimal/utc.zi")];
    let (report, _dir, inputs, config, db) =
        compile_sel(inputs, ZoneSelection::One("UTC".into()), LinkMode::Copy);
    assert!(report.zones_compiled.iter().any(|z| z.name == "Etc/UTC"));
    assert!(report.links_written.iter().any(|l| l.link_name == "UTC"));
    let m = manifest_of(&report, &inputs, &config, &db);
    assert_eq!(m.link_profile.zones_compiled_count, 1);
    assert_eq!(m.link_profile.links_materialized_count, 1);
}

// ---------------------------------------------------------------------------------------------
// T12.4c — alias-map validation: entries correspond to materialised links/copies; a link to a
// non-compiled zone fails; cycle/self-link coverage stays preserved.
// ---------------------------------------------------------------------------------------------

/// A normal compile produces an internally-consistent alias map (and `build` validates it, so a
/// successful `build` already proves consistency — this asserts it explicitly).
#[test]
fn alias_map_validate_accepts_clean_map() {
    let (report, dir, _, _, _) = compile_utc();
    let m = manifest::build(&report, dir.path()).unwrap();
    assert!(m.validate().is_ok());
}

/// Build a minimal valid map (`Etc/UTC` zone + `UTC → Etc/UTC` copy link) for mutation in the
/// negative cases below. The 64-char hash is a stand-in; only its *consistency* is what we test.
fn sample_map() -> AliasMap {
    let h = "a".repeat(64);
    let mut entries: BTreeMap<String, AliasEntry> = BTreeMap::new();
    entries.insert("Etc/UTC".into(), AliasEntry::Zone { sha256: h.clone() });
    entries.insert(
        "UTC".into(),
        AliasEntry::Link {
            target: "Etc/UTC".into(),
            target_sha256: h,
            materialised: tzcompile::LinkMode::Copy,
        },
    );
    AliasMap {
        entries,
        identifiers: 2,
        canonical_zones: 1,
        links: 1,
        duplicated_byte_links: 1,
    }
}

/// "Missing target fails": a link pointing at a zone that is **not** in the map is rejected.
#[test]
fn alias_map_validate_rejects_dangling_link() {
    let mut m = sample_map();
    m.entries.insert(
        "Ghost".into(),
        AliasEntry::Link {
            target: "Not/Compiled".into(),
            target_sha256: "b".repeat(64),
            materialised: tzcompile::LinkMode::Copy,
        },
    );
    m.identifiers = 3;
    m.links = 2;
    let err = m.validate().unwrap_err().to_string();
    assert!(err.contains("not a compiled zone"), "got: {err}");
}

/// A link whose recorded target hash disagrees with its zone's hash is rejected (the alias must
/// genuinely name those bytes).
#[test]
fn alias_map_validate_rejects_hash_mismatch() {
    let mut m = sample_map();
    if let Some(AliasEntry::Link { target_sha256, .. }) = m.entries.get_mut("UTC") {
        *target_sha256 = "c".repeat(64); // != the zone's "aaaa…"
    }
    let err = m.validate().unwrap_err().to_string();
    assert!(err.contains("target hash"), "got: {err}");
}

/// A self-link (`name == target`) is rejected.
#[test]
fn alias_map_validate_rejects_self_link() {
    let mut m = sample_map();
    m.entries.insert(
        "Loop".into(),
        AliasEntry::Link {
            target: "Loop".into(),
            target_sha256: "a".repeat(64),
            materialised: tzcompile::LinkMode::Copy,
        },
    );
    m.identifiers = 3;
    m.links = 2;
    let err = m.validate().unwrap_err().to_string();
    assert!(err.contains("self-link"), "got: {err}");
}

/// Summary counts that disagree with the entries are rejected.
#[test]
fn alias_map_validate_rejects_count_mismatch() {
    let mut m = sample_map();
    m.identifiers = 99; // lies about the entry count
    assert!(m.validate().is_err());
}

/// **Cycle/self-link coverage preserved (T12.4a/b carried into T12.4c).** A source with a cyclic
/// link pair compiles successfully (the cyclic links are skipped, never materialised) and the
/// manifest counts them as `failed` — not `selected`, not `omitted`, not a silent drop.
#[test]
fn cyclic_links_counted_failed_not_selected_or_omitted() {
    let inputs = vec![fixture("fixtures/minimal/link_cycle.zi")];
    let (report, _dir, inputs, config, db) =
        compile_sel(inputs, ZoneSelection::AllSupported, LinkMode::Copy);
    // The valid link (UTC → Etc/UTC) materialises; the cyclic Foo/Bar pair does not.
    assert!(report.links_written.iter().any(|l| l.link_name == "UTC"));
    assert!(!report
        .links_written
        .iter()
        .any(|l| l.link_name == "Foo" || l.link_name == "Bar"));
    let m = manifest_of(&report, &inputs, &config, &db);
    assert_eq!(m.link_profile.links_selected_count, 1); // UTC
    assert_eq!(m.link_profile.links_failed_count, 2); // Foo, Bar (cycle) — distinct from omitted
    assert_eq!(m.link_profile.links_omitted_count, 0);
    // The materialised alias map is still internally consistent.
    assert!(manifest::build(&report, _dir.path())
        .unwrap()
        .validate()
        .is_ok());
}

// ---------------------------------------------------------------------------------------------
// T12.4d — backward evidence axis: detected (hash-backed) vs claimed (bare) vs reconciled status.
// The alias surface is output identity, NOT source provenance — backward is never inferred from it.
// ---------------------------------------------------------------------------------------------

/// Compile `inputs` (`--all-supported`) and return the reconciled backward evidence for the given
/// `SourceVariantArgs`. (The output/input tempdirs stay alive through the manifest build inside.)
fn backward_evidence(
    inputs: Vec<PathBuf>,
    variants: SourceVariantArgs,
) -> manifest::BackwardEvidence {
    let (report, _dir, inputs, config, db) =
        compile_sel(inputs, ZoneSelection::AllSupported, LinkMode::Copy);
    let req: Vec<String> = db.zones.iter().map(|z| z.name.clone()).collect();
    manifest::build_compile_manifest(&req, &inputs, &report, &config, &db, None, None, &variants)
        .unwrap()
        .source_profile
        .backward
}

/// No claim and no admitted source → the honest default: `unknown_no_evidence`, no evidence hash.
#[test]
fn backward_unknown_when_no_evidence() {
    let e = backward_evidence(
        vec![fixture("fixtures/minimal/utc.zi")],
        SourceVariantArgs::default(),
    );
    assert_eq!(e.status(), "unknown_no_evidence");
    assert!(e.evidence_sha256.is_none());
}

/// A bare claim is recorded separately and never promoted to detection (`*_unverified`).
#[test]
fn backward_claim_recorded_separately_from_detection() {
    let inc = backward_evidence(
        vec![fixture("fixtures/minimal/utc.zi")],
        SourceVariantArgs {
            backward_claim: Some(true),
            ..Default::default()
        },
    );
    assert_eq!(inc.status(), "claimed_present_unverified");
    let exc = backward_evidence(
        vec![fixture("fixtures/minimal/utc.zi")],
        SourceVariantArgs {
            backward_claim: Some(false),
            ..Default::default()
        },
    );
    assert_eq!(exc.status(), "claimed_absent_unverified");
}

/// An admitted backward source whose bytes ARE in the build → `detected_present` (hash-backed).
#[test]
fn backward_detected_present_from_hash_backed_source() {
    let f = fixture("fixtures/minimal/utc.zi");
    let e = backward_evidence(
        vec![f.clone()],
        SourceVariantArgs {
            backward_source: Some(f),
            ..Default::default()
        },
    );
    assert_eq!(e.status(), "detected_present");
    assert!(e.evidence_sha256.is_some());
}

/// An admitted backward source whose bytes are NOT in the build → `detected_absent` (hash-backed;
/// bounded to *this* artifact, not a universal "no backward exists" claim).
#[test]
fn backward_detected_absent_when_admitted_source_not_in_build() {
    let e = backward_evidence(
        vec![fixture("fixtures/minimal/utc.zi")],
        SourceVariantArgs {
            backward_source: Some(fixture("fixtures/minimal/fixed.zi")),
            ..Default::default()
        },
    );
    assert_eq!(e.status(), "detected_absent");
    assert!(e.evidence_sha256.is_some());
}

/// Detection reconciles with a claim: agreement → `detected_matches_claim`; conflict →
/// `detected_contradicts_claim`.
#[test]
fn backward_detected_matches_and_contradicts_claim() {
    let f = fixture("fixtures/minimal/utc.zi");
    let agree = backward_evidence(
        vec![f.clone()],
        SourceVariantArgs {
            backward_claim: Some(true),
            backward_source: Some(f.clone()),
            ..Default::default()
        },
    );
    assert_eq!(agree.status(), "detected_matches_claim"); // present + included
    let conflict = backward_evidence(
        vec![f.clone()],
        SourceVariantArgs {
            backward_claim: Some(false),
            backward_source: Some(f),
            ..Default::default()
        },
    );
    assert_eq!(conflict.status(), "detected_contradicts_claim"); // present + excluded
}

/// **Non-inference (the heart of T12.4d):** a source with zones + aliases (links) but no backward
/// claim/admission keeps `unknown_no_evidence` — alias presence and link-profile counts never flip
/// the axis.
#[test]
fn backward_not_inferred_from_alias_presence_or_link_counts() {
    let e = backward_evidence(
        vec![fixture("fixtures/minimal/links_two.zi")],
        SourceVariantArgs::default(),
    );
    assert_eq!(e.status(), "unknown_no_evidence");
}

/// **Non-inference:** a source file literally named `backward`, present in the build but **not
/// admitted** via `--backward-source` and **not claimed**, stays `unknown` — the *name* proves
/// nothing (detection is hash-membership of an admitted artifact, never a filename).
#[test]
fn backward_not_inferred_from_filename() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("backward");
    std::fs::copy(fixture("fixtures/minimal/utc.zi"), &p).unwrap();
    let e = backward_evidence(vec![p], SourceVariantArgs::default());
    assert_eq!(e.status(), "unknown_no_evidence");
}

/// Detection is **order-independent**: the same admitted source yields the same status + evidence
/// hash regardless of input order.
#[test]
fn backward_status_stable_across_reordered_source_inputs() {
    let utc = fixture("fixtures/minimal/utc.zi");
    let fixed = fixture("fixtures/minimal/fixed.zi");
    let a = backward_evidence(
        vec![utc.clone(), fixed.clone()],
        SourceVariantArgs {
            backward_source: Some(utc.clone()),
            ..Default::default()
        },
    );
    let b = backward_evidence(
        vec![fixed, utc.clone()],
        SourceVariantArgs {
            backward_source: Some(utc),
            ..Default::default()
        },
    );
    assert_eq!(a.status(), "detected_present");
    assert_eq!(b.status(), "detected_present");
    assert_eq!(a.evidence_sha256, b.evidence_sha256);
}

/// **Non-inference, consolidated:** even a full manifest with a non-trivial link profile (selected
/// *and* failed links from the cyclic fixture) and a populated `alias_map_sha256` leaves `backward`
/// at `unknown` — failed/selected/omitted counts and the alias-map hash never feed the axis.
#[test]
fn backward_not_inferred_from_link_profile_counts_or_alias_hash() {
    let (report, _dir, inputs, config, db) = compile_sel(
        vec![fixture("fixtures/minimal/link_cycle.zi")],
        ZoneSelection::AllSupported,
        LinkMode::Copy,
    );
    let req: Vec<String> = db.zones.iter().map(|z| z.name.clone()).collect();
    let m = manifest::build_compile_manifest(
        &req,
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    // Preconditions: the link profile is non-trivial and the alias-map hash is populated…
    assert_eq!(m.link_profile.links_failed_count, 2);
    assert_eq!(m.link_profile.alias_map_sha256.len(), 64);
    // …yet backward evidence is untouched.
    assert_eq!(m.source_profile.backward.status(), "unknown_no_evidence");
}

/// The block renders in the JSON with the four fields.
#[test]
fn backward_evidence_renders_in_manifest_json() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    let json = m.to_json();
    assert!(json.contains("\"source_profile\""));
    assert!(json.contains("\"backward_evidence\""));
    assert!(json.contains("\"status\": \"unknown_no_evidence\""));
    // Never a boolean, never aspirational.
    assert!(!json.contains("\"backward_evidence\": true"));
}

// ---------------------------------------------------------------------------------------------
// T12.5b — backzone / PACKRATDATA source-membership evidence axis. Detection is hash-anchored to
// the pinned 2026b reference backzone; presence is hash-backed, absence is never asserted, and the
// axis is never inferred from aliases/link counts/filenames/output shape. (PACKRATLIST subset →
// T12.5c; DATAFORM → T12.5d.) Reconcile-level tests are hermetic: a synthetic reference hash + a
// constructed `SourceInputs`, so no large reference file need be vendored.
// ---------------------------------------------------------------------------------------------

/// Build a `SourceInputs` whose files carry the given per-file hashes (everything else is filler).
fn source_inputs_with_hashes(hashes: &[&str]) -> manifest::SourceInputs {
    manifest::SourceInputs {
        kind: manifest::SourceInputKind::MultiFile,
        files: hashes
            .iter()
            .enumerate()
            .map(|(i, h)| manifest::SourceFile {
                logical_name: format!("f{i}"),
                sha256: (*h).to_string(),
                bytes: 1,
                order_index: i,
            })
            .collect(),
        aggregate_hash: "agg".into(),
    }
}

/// No claim and the reference backzone hash not among the inputs → `unknown_no_evidence`.
#[test]
fn backzone_unknown_without_reference() {
    let refh = "a".repeat(64);
    let si = source_inputs_with_hashes(&["b".repeat(64).as_str()]);
    let e = manifest::BackzoneEvidence::reconcile(&si, None, &refh);
    assert_eq!(e.status(), "unknown_no_evidence");
    assert!(e.evidence_sha256.is_none());
}

/// The reference backzone hash IS among the inputs → `detected_present` (hash-backed), evidence set.
#[test]
fn backzone_detected_present_from_pinned_hash() {
    let refh = "a".repeat(64);
    let si = source_inputs_with_hashes(&[refh.as_str(), "c".repeat(64).as_str()]);
    let e = manifest::BackzoneEvidence::reconcile(&si, None, &refh);
    assert_eq!(e.status(), "detected_present");
    assert_eq!(e.evidence_sha256.as_deref(), Some(refh.as_str()));
}

/// A claim with no detection is recorded separately, never promoted (`claimed_*_unverified`).
#[test]
fn backzone_claimed_present_without_detection_is_claim_only() {
    let refh = "a".repeat(64);
    let si = source_inputs_with_hashes(&["b".repeat(64).as_str()]);
    assert_eq!(
        manifest::BackzoneEvidence::reconcile(&si, Some(true), &refh).status(),
        "claimed_present_unverified"
    );
    assert_eq!(
        manifest::BackzoneEvidence::reconcile(&si, Some(false), &refh).status(),
        "claimed_absent_unverified"
    );
}

/// Detection reconciles with a claim: agreement → matches; conflict → contradicts.
#[test]
fn backzone_detected_matches_and_contradicts_claim() {
    let refh = "a".repeat(64);
    let si = source_inputs_with_hashes(&[refh.as_str()]);
    assert_eq!(
        manifest::BackzoneEvidence::reconcile(&si, Some(true), &refh).status(),
        "detected_matches_claim"
    );
    assert_eq!(
        manifest::BackzoneEvidence::reconcile(&si, Some(false), &refh).status(),
        "detected_contradicts_claim"
    );
}

/// **Non-inference:** a real build with a rich alias/link surface (the cyclic fixture) — whose inputs
/// do not contain the pinned 2026b backzone — leaves `backzone` at `unknown_no_evidence`. Alias names,
/// selected/omitted/failed link counts, and the alias-map hash never flip it.
#[test]
fn backzone_not_inferred_from_aliases_or_link_counts() {
    let (report, _dir, inputs, config, db) = compile_sel(
        vec![fixture("fixtures/minimal/link_cycle.zi")],
        ZoneSelection::AllSupported,
        LinkMode::Copy,
    );
    let m = manifest_of(&report, &inputs, &config, &db);
    assert!(m.link_profile.links_failed_count > 0); // precondition: non-trivial link surface
    assert_eq!(m.source_profile.backzone.status(), "unknown_no_evidence");
    // And it renders in the JSON as its own axis (never a boolean).
    let json = m.to_json();
    assert!(json.contains("\"backzone_evidence\""));
    assert!(!json.contains("\"backzone_evidence\": true"));
}

/// The pinned reference hash matches the T12.5a.2 admission receipt — guards against silent drift of
/// the single source of truth (`reports/t12_5a2-reference-admission.md`).
#[test]
fn backzone_ref_hash_matches_admission_receipt() {
    assert_eq!(
        manifest::REF_2026B_BACKZONE_SHA256,
        "63fb39adae0b0d8b2179629725a9dfb694c7a386b99750b636a017d896d28dfa"
    );
    assert_eq!(manifest::REF_2026B_BACKZONE_SHA256.len(), 64);
}

// ---------------------------------------------------------------------------------------------
// T12.5c — PACKRATLIST backzone-*scope* evidence. **Category boundary:** `PACKRATLIST` is a
// *generation-policy* input (its list, `zone.tab`, is NOT a compilable `zic` source). So detection is
// keyed off an **admitted policy input** matching the pinned 2026b `zone.tab` (+ backzone present) —
// NEVER off `source_inputs` (compile inputs) or output shape. `full`/`none` are claim-only. Reconcile
// is hermetic: a synthetic reference hash + an admitted-policy hash + a `backzone_present` bool.
// ---------------------------------------------------------------------------------------------

/// A bare `--packratlist subset` claim with **no admitted policy input** → `claimed_subset_not_hash_backed`
/// (the realistic command-line path; detection stays `unknown`).
#[test]
fn packratlist_subset_claim_without_source_is_claimed_not_hash_backed() {
    let refh = "a".repeat(64);
    let e = manifest::PackratlistEvidence::reconcile(Some("subset"), None, &refh, true);
    assert_eq!(e.status(), "claimed_subset_not_hash_backed");
    assert!(e.evidence_sha256.is_none());
}

/// An admitted policy input whose hash **matches the pinned `zone.tab`** (+ backzone present) →
/// `detected_subset_from_policy_input`, evidence = the pinned hash.
#[test]
fn packratlist_subset_detected_from_policy_input_hash() {
    let refh = "a".repeat(64);
    let e = manifest::PackratlistEvidence::reconcile(None, Some(&refh), &refh, true);
    assert_eq!(e.status(), "detected_subset_from_policy_input");
    assert_eq!(e.evidence_sha256.as_deref(), Some(refh.as_str()));
}

/// **Category boundary:** `PACKRATLIST` detection never uses compile `source_inputs`. Even if
/// `zone.tab`'s hash is among the compile inputs, with **no admitted policy input** the scope stays
/// `unknown` — `reconcile` does not even take `source_inputs`.
#[test]
fn packratlist_zone_tab_not_detected_from_compile_source_inputs() {
    let refh = manifest::REF_2026B_ZONE_TAB_SHA256;
    // No admitted policy input (the `zone.tab`-in-compile-inputs scenario is simply not a parameter):
    let e = manifest::PackratlistEvidence::reconcile(None, None, refh, true);
    assert_eq!(e.status(), "unknown_no_evidence");
    assert!(e.evidence_sha256.is_none());
}

/// An admitted policy input that is **not** the pinned `zone.tab` (different hash) → `unknown`
/// (the recognized 2026b selector is required; arbitrary files don't count).
#[test]
fn packratlist_unrecognized_policy_input_is_unknown() {
    let e = manifest::PackratlistEvidence::reconcile(
        None,
        Some(&"b".repeat(64)),
        &"a".repeat(64),
        true,
    );
    assert_eq!(e.status(), "unknown_no_evidence");
}

/// Subset requires backzone present: the pinned policy input admitted but `backzone_present=false`
/// → `unknown` (a subset list is meaningless without backzone data).
#[test]
fn packratlist_subset_requires_backzone_present() {
    let refh = "a".repeat(64);
    let e = manifest::PackratlistEvidence::reconcile(None, Some(&refh), &refh, false);
    assert_eq!(e.status(), "unknown_no_evidence");
}

/// Bare claims with no admitted policy input are `claimed_*_not_hash_backed`, never promoted.
#[test]
fn packratlist_claim_recorded_separately_from_detection() {
    let refh = "a".repeat(64);
    assert_eq!(
        manifest::PackratlistEvidence::reconcile(Some("full"), None, &refh, true).status(),
        "claimed_full_not_hash_backed"
    );
    assert_eq!(
        manifest::PackratlistEvidence::reconcile(Some("none"), None, &refh, false).status(),
        "claimed_none_not_hash_backed"
    );
}

/// Detection (from the pinned policy input) reconciles with a claim: agreement → matches; conflict →
/// contradicts.
#[test]
fn packratlist_detected_matches_and_contradicts_claim() {
    let refh = "a".repeat(64);
    assert_eq!(
        manifest::PackratlistEvidence::reconcile(Some("subset"), Some(&refh), &refh, true).status(),
        "detected_matches_claim"
    );
    assert_eq!(
        manifest::PackratlistEvidence::reconcile(Some("full"), Some(&refh), &refh, true).status(),
        "detected_contradicts_claim"
    );
}

/// The pinned `zone.tab` reference hash matches the T12.5a.2 admission receipt (anti-drift guard).
#[test]
fn packratlist_zone_tab_ref_hash_matches_receipt() {
    assert_eq!(
        manifest::REF_2026B_ZONE_TAB_SHA256,
        "4d8e389e5f4b0ec0466d5b14f42e5dfb0308c4376165fcf478339afd9ddcb00c"
    );
}

// ---------------------------------------------------------------------------------------------
// T12.5d — DATAFORM (`main`/`vanguard`/`rearguard`) *encoding* evidence. Unlike PACKRATLIST's
// `zone.tab` (a non-compilable policy table), the three `.zi` artifacts ARE compilable `zic` sources,
// so detection is category-correct from `source_inputs` membership (mirrors `backzone`). It is
// hash-backed against the pinned 2026b artifacts or claim-only — **never** inferred from source
// syntax (e.g. negative SAVE), output shape, zone names, filenames, PACKRATLIST, or backzone.
// Reconcile is hermetic: a `DataformReference` with synthetic hashes + a constructed `SourceInputs`.
// ---------------------------------------------------------------------------------------------

/// Build a `DataformReference` with synthetic artifact hashes (recipe/generated_from are fixed test
/// sentinels) so the detector is exercised without vendoring the large `.zi` files.
fn dataform_ref<'a>(
    main: &'a str,
    vanguard: &'a str,
    rearguard: &'a str,
) -> manifest::DataformReference<'a> {
    manifest::DataformReference {
        main_sha256: main,
        vanguard_sha256: vanguard,
        rearguard_sha256: rearguard,
        recipe_hash: "recipe-hash-xyz",
        generated_from: "tzdb-test",
    }
}

/// No matching artifact among inputs and no claim → `unknown_no_evidence`; no provenance fields set.
#[test]
fn dataform_unknown_without_evidence() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&["d".repeat(64).as_str()]);
    let e = manifest::DataformEvidence::reconcile(&si, None, &r);
    assert_eq!(e.status(), "unknown_no_evidence");
    assert!(e.evidence_sha256.is_none());
    assert!(e.recipe_hash.is_none());
    assert!(e.generated_from.is_none());
}

/// The pinned `main.zi` hash IS among inputs → detected `main`; evidence + recipe + generated_from set.
#[test]
fn dataform_detected_main_from_pinned_main_zi_hash() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&[main.as_str(), "z".repeat(64).as_str()]);
    let e = manifest::DataformEvidence::reconcile(&si, None, &r);
    assert_eq!(e.status(), "detected_only");
    assert_eq!(e.detected, manifest::DataformDetected::Main);
    assert_eq!(e.evidence_sha256.as_deref(), Some(main.as_str()));
    assert_eq!(e.recipe_hash.as_deref(), Some("recipe-hash-xyz"));
    assert_eq!(e.generated_from.as_deref(), Some("tzdb-test"));
}

/// The pinned `vanguard.zi` hash IS among inputs → detected `vanguard`.
#[test]
fn dataform_detected_vanguard_from_pinned_vanguard_zi_hash() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&[vanguard.as_str()]);
    let e = manifest::DataformEvidence::reconcile(&si, None, &r);
    assert_eq!(e.detected, manifest::DataformDetected::Vanguard);
    assert_eq!(e.status(), "detected_only");
    assert_eq!(e.evidence_sha256.as_deref(), Some(vanguard.as_str()));
}

/// The pinned `rearguard.zi` hash IS among inputs → detected `rearguard`.
#[test]
fn dataform_detected_rearguard_from_pinned_rearguard_zi_hash() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&[rearguard.as_str()]);
    let e = manifest::DataformEvidence::reconcile(&si, None, &r);
    assert_eq!(e.detected, manifest::DataformDetected::Rearguard);
    assert_eq!(e.status(), "detected_only");
}

/// A claim with no hash-backed detection is recorded separately, never promoted (`claim_only`).
#[test]
fn dataform_claim_recorded_without_detection() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&["d".repeat(64).as_str()]);
    let e = manifest::DataformEvidence::reconcile(&si, Some("main"), &r);
    assert_eq!(e.status(), "claim_only");
    assert!(e.evidence_sha256.is_none());
    assert!(e.recipe_hash.is_none());
}

/// Detection agreeing with the claim → `detected_matches_claim`.
#[test]
fn dataform_detected_matches_claim() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&[vanguard.as_str()]);
    assert_eq!(
        manifest::DataformEvidence::reconcile(&si, Some("vanguard"), &r).status(),
        "detected_matches_claim"
    );
}

/// Detection conflicting with the claim → `detected_contradicts_claim` (detection wins; the claim is
/// never silently believed).
#[test]
fn dataform_detected_contradicts_claim() {
    let (main, vanguard, rearguard) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
    let r = dataform_ref(&main, &vanguard, &rearguard);
    let si = source_inputs_with_hashes(&[main.as_str()]);
    assert_eq!(
        manifest::DataformEvidence::reconcile(&si, Some("vanguard"), &r).status(),
        "detected_contradicts_claim"
    );
}

/// **Non-inference (the doctrine anchor):** a real compile of a source that *uses negative SAVE* (the
/// "vanguard-looking" feature), DST rules + a rich transition stream (output shape), a
/// `Backzone/`-suggestive zone name, and a link — none of which is DATAFORM evidence. Because the
/// compiled bytes are not a pinned `.zi` artifact, the encoding form stays `unknown_no_evidence`.
#[test]
fn dataform_not_inferred_from_negative_save_zone_names_or_output_shape() {
    let (report, _dir, inputs, config, db) = compile_sel(
        vec![fixture("fixtures/minimal/dataform_noninference.zi")],
        ZoneSelection::AllSupported,
        LinkMode::Copy,
    );
    let m = manifest_of(&report, &inputs, &config, &db);
    // Preconditions: the rich surface really is present (negative-SAVE + DST zones compiled, a link).
    assert!(m.zones_compiled.len() >= 2, "zones: {:?}", m.zones_compiled);
    assert!(m.link_profile.links_materialized_count >= 1);
    // Yet DATAFORM is not inferred from any of it.
    assert_eq!(m.source_profile.dataform.status(), "unknown_no_evidence");
    let json = m.to_json();
    assert!(json.contains("\"dataform_evidence\""));
    assert!(!json.contains("\"dataform_evidence\": \"vanguard\""));
    assert!(json.contains("\"recipe_hash\": null"));
}

/// **Non-inference across axes:** the other source-variant claims (`backzone` + `packratlist`) must
/// not leak into DATAFORM. With both set but no matching `.zi` artifact and no `--dataform`, the
/// encoding axis stays `unknown_no_evidence`.
#[test]
fn dataform_not_inferred_from_packratlist_or_backzone() {
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs {
            backzone_claim: Some(true),
            packratlist_claim: Some("subset".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(m.source_profile.dataform.status(), "unknown_no_evidence");
}

/// `recipe_hash` is a function of its inputs: changing the generation command changes the hash.
#[test]
fn dataform_recipe_hash_changes_when_generation_command_changes() {
    let a = manifest::dataform_recipe_hash("arch", "mk", "zg", "make main.zi", "tc");
    let b = manifest::dataform_recipe_hash("arch", "mk", "zg", "make vanguard.zi", "tc");
    assert_ne!(a, b);
    assert_eq!(a.len(), 64);
}

/// `recipe_hash` hashes **raw bytes** — line endings are part of identity, never normalized: a CRLF
/// command and an LF command hash differently (a transformed copy is a different artifact).
#[test]
fn dataform_recipe_hash_uses_raw_bytes_not_normalized_text() {
    let lf = manifest::dataform_recipe_hash("arch", "mk", "zg", "make main.zi\n", "tc");
    let crlf = manifest::dataform_recipe_hash("arch", "mk", "zg", "make main.zi\r\n", "tc");
    assert_ne!(lf, crlf);
}

/// The pinned DATAFORM artifact + `ziguard.awk` hashes match the T12.5a.2 admission receipt
/// (anti-drift guard for the single source of truth).
#[test]
fn dataform_ref_hashes_match_admission_receipt() {
    assert_eq!(
        manifest::REF_2026B_MAIN_ZI_SHA256,
        "e0225823ae0c3a99a016a4afd7e3c48cfd948132b65fbaa596a47c53ae45e4e1"
    );
    assert_eq!(
        manifest::REF_2026B_VANGUARD_ZI_SHA256,
        "49e16da4a6252a2e432fc1f68bf6daac9a6f73507dde3e3bdbcbbf78e86727ce"
    );
    assert_eq!(
        manifest::REF_2026B_REARGUARD_ZI_SHA256,
        "91c4f362a6bb297efd3cd35bce6b62367a4c00a9721a773bae0cbb0d1bf9fe23"
    );
    assert_eq!(
        manifest::REF_2026B_ZIGUARD_AWK_SHA256,
        "e4600a2360b692242d6da76666411ece8ada76b61e6f8fb69cec79592b261785"
    );
}

#[test]
fn no_manifest_claims_oracle_match_when_compare_was_not_run() {
    // A plain `compile` never invokes the oracle; the manifest must say so, regardless of the
    // repo's test status. This is the "manifest must not lie" rule.
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    // T15.2a — `mode` is now the typed owner enum, not a free string.
    assert_eq!(m.oracle.mode, tzcompile::manifest::OracleMode::NotRun);
    assert_eq!(m.oracle.result.as_str(), "not-run");
    assert!(m.oracle.horizon.is_none());
    let json = m.to_json();
    // Manifest renders the legacy `"not-run"` (boundary shim) — value unchanged, no schema bump.
    assert!(json.contains("\"mode\": \"not-run\""));
    assert!(json.contains("\"result\": \"not-run\""));
    assert!(!json.contains("\"result\": \"match\""));
}

/// T15.2a — `OracleResult.mode` is backed by the `OracleMode` owner enum (no free string), and the
/// manifest renders it through that owner via the boundary shim. (`manifest_oracle_result_uses_oracle_mode_owner`)
#[test]
fn manifest_oracle_result_uses_oracle_mode_owner() {
    use tzcompile::manifest::{OracleMode, OracleResult};
    let r = OracleResult::not_run();
    assert_eq!(r.mode, OracleMode::NotRun); // the field IS the enum, not a `String`
}

/// T15.2a — all oracle-mode rendering is single-sourced through `OracleMode`, and the manifest boundary
/// shim diverges from the canonical `mode_str` for **exactly one** value (`NotRun` → legacy `"not-run"`),
/// with no other drift. (`oracle_mode_rendering_is_single_sourced` + the drift guard.)
#[test]
fn oracle_mode_rendering_is_single_sourced_and_shim_drifts_for_one_value_only() {
    use tzcompile::manifest::OracleMode;
    let all = [
        OracleMode::NotRun,
        OracleMode::ReferenceZic,
        OracleMode::ReferenceZdump,
        OracleMode::StructuralDecode,
        OracleMode::Unavailable("x".into()),
    ];
    for m in &all {
        if matches!(m, OracleMode::NotRun) {
            assert_eq!(m.manifest_str(), "not-run", "legacy v8 boundary value");
            assert_eq!(m.mode_str(), "not_run", "canonical report value");
        } else {
            // No legacy manifest form → boundary == canonical (the shim diverges for NotRun only).
            assert_eq!(
                m.manifest_str(),
                m.mode_str(),
                "shim must not introduce drift beyond the one documented legacy value"
            );
        }
    }
}

/// T15.2a — the support-report (`mode_str`) and the manifest (`manifest_str`) oracle-mode fields are
/// produced by the **same** `OracleMode` owner; neither is a hand-written free string.
/// (`support_report_and_manifest_oracle_modes_share_enum` / `no_legacy_oracle_mode_free_string_path`.)
#[test]
fn support_report_and_manifest_oracle_modes_share_the_oracle_mode_owner() {
    use tzcompile::manifest::OracleMode;
    // The manifest's emitted value is exactly the owner's boundary rendering of NotRun…
    let (report, _dir, inputs, config, db) = compile_utc();
    let m = manifest::build_compile_manifest(
        &["Etc/UTC".into()],
        &inputs,
        &report,
        &config,
        &db,
        None,
        None,
        &SourceVariantArgs::default(),
    )
    .unwrap();
    assert!(m.to_json().contains(&format!(
        "\"mode\": \"{}\"",
        OracleMode::NotRun.manifest_str()
    )));
    // …and the support-report's value is exactly the owner's canonical rendering of NotRun.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, "Zone Etc/UTC 0:00 - UTC\n").unwrap();
    let sdb = load_database(std::slice::from_ref(&p)).unwrap();
    let sj = tzcompile::report::build_support_report(&sdb, None).to_json();
    assert!(sj.contains(&format!("\"mode\": \"{}\"", OracleMode::NotRun.mode_str())));
}
