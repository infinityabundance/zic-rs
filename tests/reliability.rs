//! T17.7 — reliability: determinism, locale/TZ isolation, and name-as-path rejection.
//!
//! These pin three "boring but load-bearing" properties:
//! * **determinism** — the same source compiles to byte-identical TZif every time (no map-iteration /
//!   allocation-address nondeterminism leaking into output);
//! * **locale/host-time isolation** — the compiled bytes do not depend on `TZ` / `LC_ALL` (the compile
//!   path takes only explicit source bytes; host env is not an input). Tested via the **binary** with
//!   distinct child-process env (no process-global env mutation → no parallel-test flakiness);
//! * **name-as-path rejection** — traversal/symlinky/absolute identifiers are refused (`ZIC008`), the
//!   lexical half of the materialization boundary (the runtime fs half is T17.4).

use std::path::Path;
use std::process::Command;

const DST_SRC: &str = "Rule R 1990 2030 - Mar lastSun 2:00 1:00 D\n\
                       Rule R 1990 2030 - Oct lastSun 2:00 0 S\n\
                       Zone Test/Z -5:00 R E%sT\n";

fn db_from(src: &str) -> tzcompile::model::Database {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("zic-rs-reltest-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("in.zi");
    std::fs::write(&p, src).unwrap();
    tzcompile::load_database(&[p]).unwrap()
}

#[test]
fn compile_is_deterministic_byte_for_byte() {
    // Compiling the same zone repeatedly (and from independently-parsed databases) yields identical
    // bytes — no nondeterminism from hash-map iteration order, allocation addresses, or timing.
    let db1 = db_from(DST_SRC);
    let a = tzcompile::compile_zone_to_bytes(&db1, "Test/Z").unwrap();
    let b = tzcompile::compile_zone_to_bytes(&db1, "Test/Z").unwrap();
    assert_eq!(a, b, "two compiles of the same db must be byte-identical");
    let db2 = db_from(DST_SRC);
    let c = tzcompile::compile_zone_to_bytes(&db2, "Test/Z").unwrap();
    assert_eq!(a, c, "a freshly-parsed db must compile to the same bytes");
}

/// Compile `DST_SRC`'s `Test/Z` to an `--out` tree via the built binary, under the given env, and
/// return the compiled file's bytes. Uses a child process so `TZ`/`LC_ALL` never touch this (parallel)
/// test process's global env.
fn compile_via_binary_with_env(env: &[(&str, &str)]) -> Vec<u8> {
    let bin = env!("CARGO_BIN_EXE_zic-rs");
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.zi");
    std::fs::write(&input, DST_SRC).unwrap();
    let out = dir.path().join("out");
    let mut cmd = Command::new(bin);
    cmd.arg("compile")
        .arg("--input")
        .arg(&input)
        .arg("--out")
        .arg(&out)
        .arg("--all-supported");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let status = cmd.status().expect("run zic-rs");
    assert!(status.success(), "compile failed under env {env:?}");
    std::fs::read(out.join("Test/Z")).expect("compiled Test/Z")
}

#[test]
fn output_is_independent_of_tz_and_lc_all() {
    // Two hostile-looking locale/timezone environments must produce byte-identical output: the compile
    // path reads source bytes, never the host clock/zone/locale.
    let a = compile_via_binary_with_env(&[("TZ", "UTC"), ("LC_ALL", "C")]);
    let b = compile_via_binary_with_env(&[
        ("TZ", "Pacific/Kiritimati"), // UTC+14, a maximal swing from UTC
        ("LC_ALL", "tr_TR.UTF-8"),    // the classic dotted-i locale foot-gun
    ]);
    assert_eq!(a, b, "compiled bytes must not depend on TZ / LC_ALL");
}

#[test]
fn traversal_and_symlinky_names_are_rejected() {
    // The name-as-path boundary (lexical half): a Link/Zone name may not escape `--out`.
    for bad in [
        "../etc/passwd",
        "/abs",
        "a/../../b",
        "a//b",
        "-flag",
        "..",
        "",
        "a/./b",
    ] {
        assert!(
            tzcompile::fs::output_tree::safe_relative_path(bad).is_err(),
            "must reject unsafe name {bad:?}"
        );
    }
    // A normal name is accepted and stays contained.
    let rel = tzcompile::fs::output_tree::safe_relative_path("Europe/London").unwrap();
    let root = Path::new("/out");
    assert!(tzcompile::fs::output_tree::is_contained(
        root,
        &root.join(rel)
    ));
}
