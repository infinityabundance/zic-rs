//! Byte-parity tests against pinned reference-`zic` output.
//!
//! These assert the *strongest* contract — exact bytes — and they are honest about it: they
//! only compare against blobs we checked in under `fixtures/expected/`, which were produced
//! by reference `zic` (tzcode 2026b; see `fixtures/MANIFEST.toml`). We do not assert byte
//! parity for anything without such a pinned blob.

use std::path::PathBuf;

use tzcompile::{compile_zone_to_bytes, load_database};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn compile(source: &str, zone: &str) -> Vec<u8> {
    let db = load_database(&[fixture(source)]).expect("load");
    compile_zone_to_bytes(&db, zone).expect("compile")
}

#[test]
fn utc_is_byte_identical_to_reference() {
    let ours = compile("fixtures/minimal/utc.zi", "Etc/UTC");
    let expected = std::fs::read(fixture("fixtures/expected/Etc_UTC.tzif")).unwrap();
    assert_eq!(ours, expected, "Etc/UTC must byte-match reference zic");
}

#[test]
fn fixed_offset_is_byte_identical_to_reference() {
    let ours = compile("fixtures/minimal/fixed.zi", "Test/Fixed");
    let expected = std::fs::read(fixture("fixtures/expected/Test_Fixed.tzif")).unwrap();
    assert_eq!(ours, expected, "Test/Fixed must byte-match reference zic");
}

#[test]
fn output_is_deterministic() {
    // Determinism is a hard requirement: compiling twice yields identical bytes.
    let a = compile("fixtures/minimal/utc.zi", "Etc/UTC");
    let b = compile("fixtures/minimal/utc.zi", "Etc/UTC");
    assert_eq!(a, b);
}

#[test]
fn output_parses_back_to_expected_semantics() {
    // Round-trip our own bytes through the reader to confirm internal consistency.
    let bytes = compile("fixtures/minimal/fixed.zi", "Test/Fixed");
    let parsed = tzcompile::tzif::parse(&bytes).unwrap();
    assert_eq!(parsed.version, b'2');
    assert_eq!(parsed.types[0].utoff, -18000);
    assert_eq!(parsed.types[0].abbr, "EST");
    assert_eq!(parsed.footer, "EST5");
}
