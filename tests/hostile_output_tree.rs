//! T14.6 — hostile-output-tree (TOCTOU) boundary, executable where mechanically feasible.
//!
//! **Honest scope.** T9 guarantees *no partial install* under **normal** filesystem errors. T14.6
//! classifies what happens under a **hostile/pre-mutated** output tree and pins the cases that are
//! deterministically testable with std-only, `#![forbid(unsafe_code)]` primitives. It **does not** claim
//! full concurrent-TOCTOU resistance: closing a parent-component symlink-swap *race* needs fd-relative
//! (`openat`/`O_NOFOLLOW`) materialization, which std does not portably expose — those cases are
//! `NotClaimed`/`RequiresOpenatStyleHardening` in `docs/zic-hostile-output-tree.md`, recorded, not faked.
//!
//! What *is* demonstrable (and tested here) rests on the materialization primitives
//! (`fs::atomic_write`): the default publish is `hard_link` (atomic exclusive create — `EEXIST` if the
//! leaf already exists, **including a symlink, without following it**); the temp file uses
//! `create_new` (`O_EXCL`). So a pre-planted symlink/file/dir at the output leaf fails closed and is
//! never written *through*.

use std::path::Path;

use tzcompile::compile::plan;
use tzcompile::{
    load_database, CompileConfig, CompileReport, EmitStyle, LinkMode, UnsupportedPolicy,
    ZoneSelection, DEFAULT_TRANSITION_LIMIT,
};

const UTC_SRC: &str = "Zone Etc/UTC 0:00 - UTC\n";

/// Compile `Etc/UTC` into a caller-controlled (possibly pre-seeded/hostile) `out` directory.
fn compile_utc_into(out: &Path, overwrite: bool) -> tzcompile::Result<CompileReport> {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.zi");
    std::fs::write(&input, UTC_SRC).unwrap();
    let inputs = vec![input];
    let db = load_database(&inputs)?;
    let config = CompileConfig {
        input_paths: inputs.clone(),
        output_dir: out.to_path_buf(),
        zones: ZoneSelection::One("Etc/UTC".to_string()),
        link_mode: LinkMode::Copy,
        overwrite,
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

/// A pre-existing **file** at the output leaf is not clobbered in the default (no-`--force`) mode.
#[test]
fn preexisting_file_at_leaf_is_fail_closed_no_clobber() {
    let out = tempfile::tempdir().unwrap();
    let leaf = out.path().join("Etc/UTC");
    std::fs::create_dir_all(leaf.parent().unwrap()).unwrap();
    std::fs::write(&leaf, b"PRECIOUS").unwrap();

    assert!(
        compile_utc_into(out.path(), false).is_err(),
        "must fail closed rather than clobber an existing leaf"
    );
    assert_eq!(
        std::fs::read(&leaf).unwrap(),
        b"PRECIOUS",
        "the pre-existing file must be untouched",
    );
}

/// A pre-existing **directory** at the output leaf fails closed (can't hard_link/rename over a dir).
#[test]
fn target_leaf_is_a_directory_is_fail_closed() {
    let out = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(out.path().join("Etc/UTC")).unwrap();
    assert!(
        compile_utc_into(out.path(), false).is_err(),
        "a directory occupying the leaf path must fail closed",
    );
}

/// A **file** where a parent directory component is needed fails closed (`create_dir_all` errors) —
/// no write happens.
#[test]
fn parent_component_is_a_file_is_fail_closed() {
    let out = tempfile::tempdir().unwrap();
    // Put a regular file at `Etc`, so `Etc/UTC`'s parent cannot be created.
    std::fs::write(out.path().join("Etc"), b"not a dir").unwrap();
    assert!(
        compile_utc_into(out.path(), false).is_err(),
        "a file blocking a needed parent directory must fail closed",
    );
}

/// **The headline TOCTOU-relevant property.** A symlink pre-planted at the output leaf is **not
/// followed**: the default no-clobber publish (`hard_link`, `O_EXCL`) fails `EEXIST`, the symlink's
/// target (the "victim") is never written through, and the symlink itself is left intact. This is the
/// concrete, demonstrable part of the hostile-output-tree boundary (a *race* mid-write is a separate,
/// `NotClaimed` case — see the doc).
#[cfg(unix)]
#[test]
fn preexisting_symlink_at_leaf_is_not_followed_through() {
    let out = tempfile::tempdir().unwrap();
    let victim_dir = tempfile::tempdir().unwrap();
    let victim = victim_dir.path().join("victim");
    std::fs::write(&victim, b"VICTIM-ORIGINAL").unwrap();

    let leaf = out.path().join("Etc/UTC");
    std::fs::create_dir_all(leaf.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&victim, &leaf).unwrap();

    // Default (no --force): must fail closed, never following the symlink to the victim.
    assert!(
        compile_utc_into(out.path(), false).is_err(),
        "a planted symlink at the leaf must fail closed (no-clobber), not be written through",
    );
    assert_eq!(
        std::fs::read(&victim).unwrap(),
        b"VICTIM-ORIGINAL",
        "the symlink's target must NOT be written through",
    );
    assert!(
        std::fs::symlink_metadata(&leaf)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the planted symlink itself must be left intact (not replaced) in no-clobber mode",
    );
}

/// With `--force`, a planted symlink at the leaf is **replaced** (via `rename`), still **not written
/// through** — the victim stays untouched. (`--force` is opt-in overwrite, not "follow and clobber".)
#[cfg(unix)]
#[test]
fn force_replaces_symlink_without_writing_through_it() {
    let out = tempfile::tempdir().unwrap();
    let victim_dir = tempfile::tempdir().unwrap();
    let victim = victim_dir.path().join("victim");
    std::fs::write(&victim, b"VICTIM-ORIGINAL").unwrap();

    let leaf = out.path().join("Etc/UTC");
    std::fs::create_dir_all(leaf.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&victim, &leaf).unwrap();

    compile_utc_into(out.path(), true).expect("--force overwrite succeeds");
    assert_eq!(
        std::fs::read(&victim).unwrap(),
        b"VICTIM-ORIGINAL",
        "even with --force, the symlink target must not be written through",
    );
    // The leaf is now a real (regular) TZif file, not the symlink.
    assert!(
        std::fs::symlink_metadata(&leaf)
            .unwrap()
            .file_type()
            .is_file(),
        "--force replaces the symlink with the compiled regular file",
    );
}

/// Sanity: the T9 no-partial-install guarantee under **normal** errors still holds — a clean output
/// dir compiles successfully and writes the leaf (the baseline the hostile cases deviate from).
#[test]
fn clean_output_dir_compiles_and_writes_leaf() {
    let out = tempfile::tempdir().unwrap();
    compile_utc_into(out.path(), false).expect("clean compile succeeds");
    assert!(out.path().join("Etc/UTC").is_file());
}
