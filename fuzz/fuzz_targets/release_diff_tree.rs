#![no_main]
//! Fuzz the **release-diff** logic over two parsed releases. The input is split in half; each half is
//! parsed as `.zi` source into a `Database`, then `build_release_diff` compares them **structurally
//! only** (`zdump_program: None` ⇒ no external process, fully deterministic, no host dependence).
//! Contract: never panic; `Ok`/`Err` only. (Lower throughput than the pure targets — it compiles zones
//! and creates a scratch tempdir per iteration; that is acceptable for a scaffold, the operator can
//! tune `-max_len`/`-runs`.)
use libfuzzer_sys::fuzz_target;
use std::path::Path;
use tzcompile::model::Database;
use tzcompile::release_diff::{build_release_diff, ReleaseDiffOptions};

fuzz_target!(|data: &[u8]| {
    let mid = data.len() / 2;
    let (old_bytes, new_bytes) = data.split_at(mid);

    let mut old_db = Database::default();
    if tzcompile::source::parser::parse_into(old_bytes, Path::new("<old>"), &mut old_db).is_err() {
        return;
    }
    let mut new_db = Database::default();
    if tzcompile::source::parser::parse_into(new_bytes, Path::new("<new>"), &mut new_db).is_err() {
        return;
    }

    let opts = ReleaseDiffOptions {
        horizon: (1900, 2040),
        split: 2025,
        zone_filter: None,
        zdump_program: None, // structural only — no external process, deterministic
    };
    let _ = build_release_diff(&old_db, &new_db, &opts);
});
