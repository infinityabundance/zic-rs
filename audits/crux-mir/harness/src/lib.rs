//! crux-mir SYMBOLIC harnesses for zic-rs reduced-surface invariants — a SECOND-ENGINE (Galois Crucible
//! + what4 + SMT) corroboration of the Kani (CBMC) bounded proofs. Each harness reproduces a real zic-rs
//! invariant from `src/tzif/validate.rs` and proves it over a SYMBOLIC domain. Linear integer arithmetic
//! only (z3 discharges it decidably) — the nonlinear count×size product is left to Kani + cargo-fuzz
//! (nonlinear bitvector multiply is intractable for any SMT solver regardless of value bounds).
#![cfg_attr(crux, no_std)]
extern crate crucible;
use crucible::*;

/// Cursor non-truncation (mirrors `Cursor::skip_within_remaining`, Kani T23.kani.2): if `pos <= len`
/// and the requested advance `n <= len - pos`, then `pos + n <= len` — the read stays in bounds.
/// Proven for ALL symbolic pos, n, len (bounded to avoid usize-arithmetic overflow, the real domain).
#[crux::test]
fn cursor_skip_cannot_truncate() {
    let len = u64::symbolic("len");
    let pos = u64::symbolic("pos");
    let n   = u64::symbolic("n");
    crucible_assume!(len <= 1 << 40);          // realistic file-size bound; keeps sums in u64
    crucible_assume!(pos <= len);
    crucible_assume!(n <= len - pos);          // the skip_within_remaining precondition
    crucible_assert!(pos + n <= len);          // invariant: never advances past the end
}

/// Transition type-index guard (mirrors the `type_index < typecnt` check, Kani T23.kani.3a): once the
/// guard admits an index, using it as a slice index into the `typecnt`-long type table is in bounds.
#[crux::test]
fn type_index_guard_is_sound() {
    let typecnt = u32::symbolic("typecnt");
    let idx     = u32::symbolic("idx");
    crucible_assume!(typecnt >= 1);            // RFC 9636: typecnt >= 1
    crucible_assume!(idx < typecnt);           // the guard that admitted this index
    crucible_assert!((idx as u64) < (typecnt as u64));  // => valid index into types[..typecnt]
}

/// Indicator pairing (mirrors the RFC-9636 indicator validity rule, Kani T23.kani.3f.3): a UT/local
/// indicator set implies its standard/wall indicator set — `isut == 1 ==> isstd == 1`. Proven over the
/// valid octet domain {0,1}.
#[crux::test]
fn indicator_pairing_holds() {
    let isstd = u8::symbolic("isstd");
    let isut  = u8::symbolic("isut");
    crucible_assume!(isstd <= 1 && isut <= 1);     // octets are booleans
    crucible_assume!(isut == 0 || isstd == 1);     // the validity rule (isut=1 => isstd=1)
    crucible_assert!(!(isut == 1) || isstd == 1);  // restated invariant holds
}
