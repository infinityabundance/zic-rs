//! Portability: the **no-host-dependence** test (T16.4 / T17 fold).
//!
//! Proves the zic-rs *library* compiler core is platform-neutral — it turns explicit tzdata source
//! **bytes** into TZif **bytes** entirely in-process, with **no filesystem, environment, process, or
//! host-tzdata access**. This is the runnable backing for the `docs/platform-portability.md` claim
//! ("platform-neutral compiler core"). The OS-specific install surfaces (symlink / chmod / ownership)
//! live in `src/fs/output_tree.rs` behind `#[cfg(unix)]` gates and are *not* exercised here.

use std::path::Path;

use tzcompile::{compile_zone_to_bytes, model::Database, source::parse_into, tzif};

/// Compile a recurring DST zone from in-memory source bytes to TZif bytes, then decode the result —
/// no file is opened (the `Path` is diagnostic-only), no env var read, no process spawned.
#[test]
fn library_compiles_bytes_to_tzif_with_no_host() {
    // Explicit source bytes — never touches `/usr/share/zoneinfo`, `--input`, or any path on disk.
    let src = b"Rule US 2007 max - Mar Sun>=8 2:00 1:00 D\n\
                Rule US 2007 max - Nov Sun>=1 2:00 0 S\n\
                Zone Test/Eastern -5:00 US E%sT\n";

    let mut db = Database::default();
    parse_into(src, Path::new("<in-memory>"), &mut db).expect("parse in-memory source");

    let bytes = compile_zone_to_bytes(&db, "Test/Eastern").expect("compile to bytes");

    // It is a valid TZif artifact, decodable without any host context.
    let parsed = tzif::parse(&bytes).expect("decode compiled bytes");
    assert!(
        !parsed.types.is_empty(),
        "must have at least one local-time type"
    );
    assert!(
        parsed.transitions.len() >= 2,
        "a recurring DST zone has explicit transitions"
    );
    assert!(
        !parsed.footer.is_empty(),
        "a recurring zone carries a POSIX footer (EST5EDT,...)"
    );
    assert!(matches!(parsed.version, b'2' | b'3'));
}

/// A fixed-offset zone likewise compiles purely in-memory (the minimal portable case).
#[test]
fn library_compiles_fixed_zone_with_no_host() {
    let mut db = Database::default();
    parse_into(b"Zone Etc/UTC 0 - UTC\n", Path::new("<in-memory>"), &mut db).unwrap();
    let bytes = compile_zone_to_bytes(&db, "Etc/UTC").expect("compile fixed zone");
    let parsed = tzif::parse(&bytes).unwrap();
    assert_eq!(parsed.types[0].utoff, 0);
    assert_eq!(parsed.types[0].abbr, "UTC");
}
