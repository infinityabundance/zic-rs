#![no_main]
//! Fuzz the **zone/link name → output-path policy** (`fs::output_tree::safe_relative_path`) — the choke
//! point that turns an untrusted identifier into a relative path strictly under `--out` (rejects
//! traversal/absolute/`..`/empty-component/leading-`-`/NUL → `ZIC008`). This is the *name-as-path* model
//! (the lexical half of the materialization story; the runtime fs half is in T17.4's contract).
//! Contract: every name yields `Ok(path)` or `Err(ZIC008)` — never a panic, and an accepted path is
//! relative + contained. The pure, fast target — ideal for high-throughput fuzzing.
use libfuzzer_sys::fuzz_target;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    if let Ok(name) = std::str::from_utf8(data) {
        if let Ok(rel) = tzcompile::fs::output_tree::safe_relative_path(name) {
            // An accepted name must be relative and lexically contained under any root (invariant).
            assert!(rel.is_relative(), "accepted name produced a non-relative path: {name:?}");
            let root = Path::new("/out");
            assert!(
                tzcompile::fs::output_tree::is_contained(root, &root.join(&rel)),
                "accepted name escapes root: {name:?}"
            );
        }
    }
});
