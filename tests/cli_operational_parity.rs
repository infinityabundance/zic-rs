//! Operational (CLI / filesystem / install-policy) parity — campaigns T9 + T10.
//!
//! - **T9.2 exit-status parity:** pin the **CLI contract** — the success/failure *classification* a
//!   script depends on — not diagnostic wording (which may evolve). Taxonomy (see `src/main.rs`):
//!   `0` success · `1` operational/compiler/config failure · `2` clap usage error.
//! - **T9.3 filesystem materialization:** no partial install after a fatal; `-D`/`--no-create-dirs`.
//! - **T9.4 localtime install policy:** `-l`/`--localtime` (+ `-t`/`--localtime-name`) within `--out`.
//! - **T9.5 file mode:** `-m`/`--mode` (octal, Unix-only) on created files.
//! - **T10.2 emission bloat:** `-b {slim|fat}` as an alias onto `--emit-style`.
//! - **T10.3 redundant tail:** `-R @hi` / `--redundant-until` (slim-only, independent of `-b`).
//! - **T10.4d range:** `-r` truncation (clamp + `-00` boundaries); well-formed succeeds, malformed → 1.
//! - **T11.6 `right/` profile:** `-L`/`--leapseconds` applies a leap table; opt-in, never default.
//!
//! We assert process exit codes (and, where relevant, the output tree) via a real binary invocation.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn zic_rs(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zic-rs"))
        .args(args)
        .output()
        .expect("spawn zic-rs")
}

/// Write `src` to a fresh temp `.zi` and return (dir, path) — the dir guards lifetime.
fn source(src: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, src).unwrap();
    (dir, p.to_string_lossy().into_owned())
}

fn out_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

// --- 0: success --------------------------------------------------------------------------------

#[test]
fn help_exits_zero() {
    assert_eq!(zic_rs(&["--help"]).status.code(), Some(0));
}

#[test]
fn version_exits_zero() {
    assert_eq!(zic_rs(&["--version"]).status.code(), Some(0));
}

#[test]
fn supported_syntax_exits_zero() {
    assert_eq!(zic_rs(&["supported-syntax"]).status.code(), Some(0));
}

#[test]
fn valid_compile_exits_zero() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
    ]);
    assert_eq!(
        st.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
}

// --- 1: operational / compiler / config failure ------------------------------------------------

#[test]
fn parse_error_exits_one() {
    // A `Zone` line missing required fields is a parse error.
    let (_d, src) = source("Zone Broken\n");
    let out = out_dir();
    let code = zic_rs(&[
        "compile",
        "--input",
        &src,
        "--all-supported",
        "--out",
        &out.path().to_string_lossy(),
    ])
    .status
    .code();
    assert_eq!(
        code,
        Some(1),
        "parse error should be operational failure (1)"
    );
}

#[test]
fn unsupported_feature_exits_one() {
    // `%s` on a no-rules era has no LETTER → fail closed (ZIC001), default `--unsupported error`.
    let (_d, src) = source("Zone Test/NS 5:00 - E%sT\n");
    let out = out_dir();
    let code = zic_rs(&[
        "compile",
        "--input",
        &src,
        "--zone",
        "Test/NS",
        "--out",
        &out.path().to_string_lossy(),
    ])
    .status
    .code();
    assert_eq!(
        code,
        Some(1),
        "unsupported construct should fail closed (1)"
    );
}

#[test]
fn output_path_traversal_exits_one() {
    // A zone whose name escapes the output tree → ZIC008 hard reject (operational failure).
    let (_d, src) = source("Zone ../escape 0 - X\n");
    let out = out_dir();
    let code = zic_rs(&[
        "compile",
        "--input",
        &src,
        "--all-supported",
        "--out",
        &out.path().to_string_lossy(),
    ])
    .status
    .code();
    assert_eq!(
        code,
        Some(1),
        "path traversal must be rejected (1), never written"
    );
}

#[test]
fn output_exists_without_force_exits_one() {
    let out = out_dir();
    let args = [
        "compile",
        "--input",
        &*fixture("fixtures/minimal/utc.zi")
            .to_string_lossy()
            .into_owned(),
        "--zone",
        "Etc/UTC",
        "--out",
        &*out.path().to_string_lossy().into_owned(),
    ];
    assert_eq!(zic_rs(&args).status.code(), Some(0), "first write succeeds");
    assert_eq!(
        zic_rs(&args).status.code(),
        Some(1),
        "re-compile without --force must fail (no clobber)"
    );
}

#[test]
fn missing_out_exits_one() {
    // `--out` has no implicit default; its absence is a config error (1), not a clap usage error.
    let code = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
    ])
    .status
    .code();
    assert_eq!(code, Some(1));
}

#[test]
fn missing_input_file_exits_one() {
    let out = out_dir();
    let code = zic_rs(&[
        "compile",
        "--input",
        "/nonexistent/does-not-exist.zi",
        "--all-supported",
        "--out",
        &out.path().to_string_lossy(),
    ])
    .status
    .code();
    assert_eq!(
        code,
        Some(1),
        "unreadable input is an operational failure (1)"
    );
}

// --- 2: clap usage error -----------------------------------------------------------------------

#[test]
fn invalid_option_exits_two() {
    assert_eq!(
        zic_rs(&["compile", "--no-such-flag"]).status.code(),
        Some(2)
    );
}

#[test]
fn missing_required_input_exits_two() {
    // `--input` is structurally required by clap (unlike `--out`), so its absence is a usage error.
    let out = out_dir();
    let code = zic_rs(&[
        "compile",
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
    ])
    .status
    .code();
    assert_eq!(
        code,
        Some(2),
        "missing clap-required --input is a usage error (2)"
    );
}

// --- T9.3: filesystem materialization — no partial install after a fatal -----------------------

fn out_entry_count(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir).map(|rd| rd.count()).unwrap_or(0)
}

/// **The operationally-serious guarantee.** A multi-zone run where a *later* zone is fatal must leave
/// the output tree with **no partial install** — the earlier, good zone is never written, because
/// zic-rs compiles everything to memory first (phase 1) and only materialises if all succeed.
#[test]
fn unsupported_later_zone_leaves_no_partial_install() {
    // Source order: a good fixed-offset zone, then an unsupported `%s` no-rules era (ZIC001).
    let (_d, src) = source("Zone Good/Zone 0 - UTC\nZone Bad/Zone 5:00 - E%sT\n");
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &src,
        "--all-supported",
        "--out",
        &out.path().to_string_lossy(),
    ]);
    assert_eq!(st.status.code(), Some(1), "the run must fail (1)");
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "no zone file may be written — not even the good one that compiled first"
    );
}

/// A path-traversal zone name is caught in phase 1 (pre-write), so an earlier good zone is also
/// never materialised — the tree stays empty.
#[test]
fn traversal_zone_leaves_no_partial_install() {
    let (_d, src) = source("Zone Good/Zone 0 - UTC\nZone ../escape 0 - X\n");
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &src,
        "--all-supported",
        "--out",
        &out.path().to_string_lossy(),
    ]);
    assert_eq!(st.status.code(), Some(1), "traversal must fail (1)");
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "nothing written after a traversal reject"
    );
}

/// `--no-create-dirs` (`-D`): the output dir must already exist; a missing one is a config error (1),
/// and — crucially — nothing is written.
#[test]
fn no_create_dirs_missing_dir_exits_one() {
    let out = out_dir();
    let missing = out.path().join("does-not-exist");
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &missing.to_string_lossy(),
        "--no-create-dirs",
    ]);
    assert_eq!(st.status.code(), Some(1));
    assert!(
        !missing.exists(),
        "--no-create-dirs must not create the directory"
    );
}

/// `--no-create-dirs` with an existing output dir **and the zone's parent subdir pre-created**
/// succeeds. Under `-D` no directory is ever created (ZIC-MATRIX.1.D — reference `zic -D` semantics),
/// so `Etc/UTC` only materialises if `Etc/` already exists.
#[test]
fn no_create_dirs_existing_dir_succeeds() {
    let out = out_dir();
    std::fs::create_dir_all(out.path().join("Etc")).unwrap(); // the parent subdir must pre-exist under -D
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--no-create-dirs",
    ]);
    assert_eq!(
        st.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
}

/// ZIC-MATRIX.1.D regression — under `-D`, a zone whose parent subdirectory does **not** already
/// exist is NOT materialised (no directory is created) and the run exits 1; a zone whose parent
/// *does* exist (root-level here) is still written. This matches reference `zic -D` "write what
/// fits, skip the rest, fail if any skipped" — closing the one `divergent` ZIC-MATRIX.1 row.
#[test]
fn no_create_dirs_skips_zone_needing_missing_subdir() {
    let out = out_dir(); // flat: no `Sub/` subdirectory
    let st = zic_rs(&[
        "compile",
        "--all-supported",
        "--input",
        &fixture("reports/zic-matrix/subdir.zi").to_string_lossy(),
        "--out",
        &out.path().to_string_lossy(),
        "--no-create-dirs",
    ]);
    // exit 1 because `Sub/Deep` was skipped (its `Sub/` parent was absent and -D forbids creating it)
    assert_eq!(
        st.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    // the root-level zone IS written; the subdir is NEVER created, and its zone is NOT materialised
    assert!(
        out.path().join("UTCONLY").exists(),
        "root-level zone must be written under -D"
    );
    assert!(
        !out.path().join("Sub").exists(),
        "-D must not create the missing Sub/ subdirectory"
    );
    assert!(
        !out.path().join("Sub/Deep").exists(),
        "the subdir-needing zone must be skipped under -D"
    );
}

// --- T9.4: localtime install policy (`-l`/`--localtime`, `-t`/`--localtime-name`) ---------------

/// `--localtime <zone>` creates a `localtime` entry in `--out` pointing at the (also-selected) zone.
/// Opt-in install policy; it never affects the canonical-zone behaviour sweep.
#[test]
fn localtime_creates_link() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--localtime",
        "Etc/UTC",
    ]);
    assert_eq!(
        st.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    assert!(
        out.path().join("localtime").exists(),
        "a `localtime` entry should have been written"
    );
}

/// `--localtime-name` (`-t`) chooses the link name (a safe relative name under `--out`).
#[test]
fn localtime_custom_name() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--localtime",
        "Etc/UTC",
        "--localtime-name",
        "mytime",
    ]);
    assert_eq!(st.status.code(), Some(0));
    assert!(out.path().join("mytime").exists());
    assert!(
        !out.path().join("localtime").exists(),
        "the default name must not also be written"
    );
}

/// A `--localtime` target that was **not selected** is a phase-1 config error (1) — and, crucially,
/// nothing is written (no partial install: the selected zone is staged but never materialised).
#[test]
fn localtime_unselected_target_fails_no_partial_install() {
    let (_d, src) = source("Zone One/Zone 0 - UTC\nZone Two/Zone 1 - PLUS1\n");
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &src,
        "--zone",
        "One/Zone",
        "--out",
        &out.path().to_string_lossy(),
        "--localtime",
        "Two/Zone", // a real zone, but not selected
    ]);
    assert_eq!(
        st.status.code(),
        Some(1),
        "an unselected localtime target is a config error (1)"
    );
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "nothing may be written when the localtime target is invalid"
    );
}

/// An unsafe `--localtime-name` (traversal/absolute) is rejected up front (1), nothing written —
/// the *intentional safer divergence*: reference `zic` would write to an arbitrary path; zic-rs
/// only ever writes a safe relative name under `--out`.
#[test]
fn localtime_unsafe_name_rejected_no_partial_install() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--localtime",
        "Etc/UTC",
        "--localtime-name",
        "../escape",
    ]);
    assert_eq!(st.status.code(), Some(1), "unsafe link name must fail (1)");
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "nothing written when the localtime name is unsafe"
    );
}

// --- T9.5: file mode (`-m`/`--mode`) — Unix-only install metadata --------------------------------

/// `--mode <octal>` sets the permission bits of the created TZif file (Unix only).
#[cfg(unix)]
#[test]
fn mode_applied_to_compiled_file() {
    use std::os::unix::fs::PermissionsExt;
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--mode",
        "600",
    ]);
    assert_eq!(
        st.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    let perms = std::fs::metadata(out.path().join("Etc/UTC"))
        .unwrap()
        .permissions();
    assert_eq!(
        perms.mode() & 0o777,
        0o600,
        "the TZif file should carry the requested mode"
    );
}

/// A copied link entry also receives `--mode`; the compiled file and its copy both carry it.
#[cfg(unix)]
#[test]
fn mode_applied_to_copied_link() {
    use std::os::unix::fs::PermissionsExt;
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--link-mode",
        "copy",
        "--mode",
        "640",
    ]);
    assert_eq!(st.status.code(), Some(0));
    // `utc.zi` has `Link Etc/UTC UTC`, materialised as a copy.
    let link_perms = std::fs::metadata(out.path().join("UTC"))
        .unwrap()
        .permissions();
    assert_eq!(link_perms.mode() & 0o777, 0o640);
}

/// INSTALL-SEMANTICS.1: the default base file mode is **0644** (reference `zic`'s data-file mode), so the
/// output is **never world-writable** regardless of umask. Run under a permissive umask 000 — Rust's default
/// `create` mode 0666 would leave a 0666 (world-writable) file here; 0644 base must yield exactly 0644.
#[cfg(unix)]
#[test]
fn default_file_mode_is_0644_base_under_permissive_umask() {
    use std::os::unix::fs::PermissionsExt;
    let out = out_dir();
    let bin = env!("CARGO_BIN_EXE_zic-rs");
    let fix = fixture("fixtures/minimal/utc.zi");
    let cmd = format!(
        "umask 000 && '{}' compile --input '{}' --zone Etc/UTC --out '{}'",
        bin,
        fix.display(),
        out.path().display()
    );
    let st = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("spawn sh");
    assert_eq!(
        st.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    let mode = std::fs::metadata(out.path().join("Etc/UTC"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o644,
        "default base mode must be 0644 under umask 000 (the 0666 default would be world-writable)"
    );
    assert_eq!(mode & 0o022, 0, "must never be group/other-writable");
}

/// A non-octal `--mode` is a config error (1), caught before any write — no partial install.
#[test]
fn bad_octal_mode_fails_no_partial_install() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--mode",
        "999", // 9 is not an octal digit
    ]);
    assert_eq!(
        st.status.code(),
        Some(1),
        "invalid octal mode must fail (1)"
    );
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "nothing written when --mode is invalid"
    );
}

// --- T10.2: emission bloat (`-b {slim|fat}`) — an alias onto `--emit-style` ----------------------

/// `-b slim` and `--emit-style zic-slim` route to the same emission policy → byte-identical output.
/// (The slim-vs-fat *difference* itself is proven over real recurring zones in `tests/emit_style.rs`;
/// here we only pin that the `-b` alias maps onto the same machinery.)
#[test]
fn bloat_alias_matches_emit_style() {
    let input = fixture("fixtures/minimal/utc.zi")
        .to_string_lossy()
        .into_owned();
    let run = |last: &[&str]| -> Vec<u8> {
        let out = out_dir();
        let out_s = out.path().to_string_lossy().into_owned();
        let mut a = vec![
            "compile", "--input", &input, "--zone", "Etc/UTC", "--out", &out_s,
        ];
        a.extend_from_slice(last);
        assert_eq!(zic_rs(&a).status.code(), Some(0));
        std::fs::read(out.path().join("Etc/UTC")).unwrap()
    };
    assert_eq!(
        run(&["-b", "slim"]),
        run(&["--emit-style", "zic-slim"]),
        "`-b slim` must equal `--emit-style zic-slim`"
    );
}

/// `-b` and a *conflicting* `--emit-style` is a config error (1) — mirroring `zic`'s own
/// "incompatible -b options" check — and nothing is written.
#[test]
fn bloat_conflicts_with_emit_style_exits_one() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "-b",
        "fat",
        "--emit-style",
        "zic-slim",
    ]);
    assert_eq!(st.status.code(), Some(1), "conflicting emission knobs → 1");
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "nothing written on conflict"
    );
}

// --- T10.3: redundant-tail bound (`-R @hi`) ------------------------------------------------------

/// `-R` without the `@` prefix is a config error (1), caught before any write.
#[test]
fn redundant_without_at_prefix_exits_one() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "-R",
        "946684800", // missing the leading '@'
    ]);
    assert_eq!(
        st.status.code(),
        Some(1),
        "-R without @ is a config error (1)"
    );
    assert_eq!(out_entry_count(out.path()), 0, "nothing written");
}

/// `-R` with no value at all is a clap usage error (2).
#[test]
fn redundant_missing_value_exits_two() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "-R",
    ]);
    assert_eq!(
        st.status.code(),
        Some(2),
        "missing -R value is a usage error (2)"
    );
}

// --- T10.4d: range truncation (`-r`) — implemented (clamp + `-00` boundaries) --------------------

/// A **well-formed** `-r` now truncates and succeeds (exit 0), writing the file (T10.4d).
#[test]
fn range_well_formed_succeeds_and_writes() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "-r",
        "@0/@4102444800",
    ]);
    assert_eq!(
        st.status.code(),
        Some(0),
        "well-formed -r truncates and succeeds; stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    assert!(
        out.path().join("Etc/UTC").exists(),
        "the truncated file is written"
    );
}

/// A **malformed** `-r` is a config error (1), nothing written.
#[test]
fn range_malformed_exits_one() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/minimal/utc.zi").to_string_lossy(),
        "--zone",
        "Etc/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "-r",
        "@10/@5", // hi < lo
    ]);
    assert_eq!(
        st.status.code(),
        Some(1),
        "malformed -r is a config error (1)"
    );
    assert_eq!(out_entry_count(out.path()), 0, "nothing written");
}

// --- T11.6: `right/` build profile (`-L`/`--leapseconds`) — opt-in leap table ---------------------

/// `--leapseconds <file>` applies the leap table to compiled zones (the `right/` profile); without it,
/// ordinary output has **no** leap table. Opt-in, never default.
#[test]
fn leapseconds_right_profile_applies_table() {
    // Right profile: leap table present.
    let rp = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/leap/src/utc.zi").to_string_lossy(),
        "--zone",
        "Test/UTC",
        "--out",
        &rp.path().to_string_lossy(),
        "--leapseconds",
        &fixture("fixtures/leap/src/stationary.list").to_string_lossy(),
    ]);
    assert_eq!(
        st.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&st.stderr)
    );
    let right =
        tzcompile::tzif::parse(&std::fs::read(rp.path().join("Test/UTC")).unwrap()).unwrap();
    assert!(
        !right.leaps.is_empty(),
        "right profile must carry the leap table"
    );

    // Ordinary profile (no -L): no leap table.
    let op = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/leap/src/utc.zi").to_string_lossy(),
        "--zone",
        "Test/UTC",
        "--out",
        &op.path().to_string_lossy(),
    ]);
    assert_eq!(st.status.code(), Some(0));
    let ord = tzcompile::tzif::parse(&std::fs::read(op.path().join("Test/UTC")).unwrap()).unwrap();
    assert!(
        ord.leaps.is_empty(),
        "ordinary profile must NOT carry a leap table"
    );
}

/// `--leapseconds` with a missing/unreadable file is a config error (1), nothing written.
#[test]
fn leapseconds_missing_file_exits_one() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/leap/src/utc.zi").to_string_lossy(),
        "--zone",
        "Test/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--leapseconds",
        "/nonexistent/leapseconds",
    ]);
    assert_eq!(
        st.status.code(),
        Some(1),
        "missing leap source is a config error (1)"
    );
    assert_eq!(out_entry_count(out.path()), 0, "nothing written");
}

/// `Rolling` leaps + `-r` (range) is a hard error caught before any write (T11.4 / T9.3 guarantee).
#[test]
fn rolling_leaps_with_range_exits_one_no_output() {
    let out = out_dir();
    let st = zic_rs(&[
        "compile",
        "--input",
        &fixture("fixtures/leap/src/utc.zi").to_string_lossy(),
        "--zone",
        "Test/UTC",
        "--out",
        &out.path().to_string_lossy(),
        "--leapseconds",
        &fixture("fixtures/leap/src/rolling.list").to_string_lossy(),
        "-r",
        "@0",
    ]);
    assert_eq!(st.status.code(), Some(1), "Rolling + -r must fail (1)");
    assert_eq!(
        out_entry_count(out.path()),
        0,
        "no partial install after the fatal"
    );
}
