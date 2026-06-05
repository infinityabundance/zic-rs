//! Regression witnesses for the three panic-on-hostile-input findings from `T23.cargo-fuzz.1`
//! (F1/F2/F3), fixed in `T23.cargo-fuzz.2`.
//!
//! Each test feeds the **exact minimized crash input** preserved under
//! `audits/cargo-fuzz/findings/` through the **same public entry the fuzz target used**, and
//! asserts the call returns gracefully instead of aborting the process. Reaching the assertions at
//! all is the proof: before the fix these inputs panicked (slice-index / `str`-slice /
//! multiply-overflow). The sharp per-function unit tests live next to each fix
//! (`tzif::validate`, `model::time`, `source::parser`); these replay the byte-for-byte seeds.

use std::path::Path;
use tzcompile::model::Database;

// The four minimized crash seeds, committed as regression fixtures.
const F1_POSIX_FOOTER: &[u8] =
    include_bytes!("../audits/cargo-fuzz/findings/F1-validate-footer-posix_footer.bin");
const F1_TZIF_VALIDATE: &[u8] =
    include_bytes!("../audits/cargo-fuzz/findings/F1-validate-footer-tzif_validate_bytes.bin");
const F2_RELEASE_DIFF: &[u8] =
    include_bytes!("../audits/cargo-fuzz/findings/F2-time-multiply-overflow-release_diff_tree.bin");
const F3_ZONE_RULE_LINK: &[u8] = include_bytes!(
    "../audits/cargo-fuzz/findings/F3-parser-char-boundary-zone_rule_link_parser.bin"
);

#[test]
fn f1_single_newline_footer_seed_does_not_panic() {
    // Targets `posix_footer` + `tzif_validate_bytes` — `tzif/validate.rs` single-`\n` footer slice.
    // The crafted TZif must be a typed rejection, never a panic.
    assert!(tzcompile::tzif::validate::parse(F1_POSIX_FOOTER).is_err());
    assert!(tzcompile::tzif::validate::parse(F1_TZIF_VALIDATE).is_err());
    // `tzif_validate_bytes` also drives the RFC-9636 validator; it must not panic either.
    let _ = tzcompile::tzif::rfc9636::validate(F1_TZIF_VALIDATE);
}

#[test]
fn f2_huge_hour_seed_does_not_overflow_panic() {
    // Target `release_diff_tree` splits the input in half and `parse_into`s each half before
    // diffing; the overflow panic (`model/time.rs` `h*3600`) fired during that parse. Replay it.
    let mid = F2_RELEASE_DIFF.len() / 2;
    let (old_bytes, new_bytes) = F2_RELEASE_DIFF.split_at(mid);
    let mut old_db = Database::default();
    let _ = tzcompile::source::parser::parse_into(old_bytes, Path::new("<old>"), &mut old_db);
    let mut new_db = Database::default();
    let _ = tzcompile::source::parser::parse_into(new_bytes, Path::new("<new>"), &mut new_db);
    // And the whole seed through the parser (the offset-parse path reaches the time parser).
    let mut db = Database::default();
    let _ = tzcompile::source::parser::parse_into(F2_RELEASE_DIFF, Path::new("<f2>"), &mut db);
}

#[test]
fn f3_multibyte_prefix_seed_does_not_panic() {
    // Target `zone_rule_link_parser` — `source/parser.rs` non-char-boundary `str` slice.
    let mut db = Database::default();
    let _ = tzcompile::source::parser::parse_into(F3_ZONE_RULE_LINK, Path::new("<fuzz>"), &mut db);
}
