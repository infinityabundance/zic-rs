//! CREUSOT deductive proof of a real zic-rs invariant: the count-arithmetic non-overflow underpinning
//! `tzif::validate::checked_block_len` (T17.5 RISK.COUNT.1; also Kani-proven T23.kani.1 + crux-mir-adjacent).
//! A TZif header's `count: u32` x element `size: u32` must not wrap before allocation; widening both to u64
//! makes the product EXACT and the result is bounded by u32::MAX^2 < u64::MAX. Kani bounded-proves it (CBMC);
//! creusot proves it deductively here via Why3 + SMT (alt-ergo/z3/cvc5) — a third-engine corroboration.
use creusot_std::prelude::*;

#[ensures(result@ == count@ * size@)]
pub fn block_len(count: u32, size: u32) -> u64 {
    (count as u64) * (size as u64)
}
