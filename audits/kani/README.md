# audits/kani — bounded model checking (CBMC)

> **Status: ✅ RUN — 10 bounded helper proofs VERIFIED · 0 failed · 0 counterexamples.** An audit folder
> existing is not an audit result — a result is admitted only by a receipt in `receipts/` (tool · version ·
> host · command · input+hash · duration · exit · findings · fixed · residual · non-claims · next-owner).
> See `../README.md`.

**Doctrine (load-bearing):** *Kani proves **sharp, reduced-surface invariants**, not broad parser vibes.*
Every harness here is a small, alloc-free, format-free helper over symbolic inputs that **converges in
well under a second**. Future Kani work stays on reduced-surface helper invariants only.

- **Scope (what it witnesses for zic-rs):** within each harness's bound — no panic / no overflow-wrap / no
  OOB / the asserted property — on the harnessed pure helper.
- **Command:** `cargo kani` (or `--harness <name>` per proof).
- **Tool / version:** `cargo-kani` 0.67.0 (CBMC; nightly-2025-11-21). Runs 2026-06-02 / 2026-06-03.
- **Harnesses** (`src/tzif/validate.rs` `#[cfg(kani)] mod kani_harness`; `cfg(kani)`-gated so the 497-test
  gate + clippy `-D warnings` stay green via `Cargo.toml [lints.rust] check-cfg`):

| ID | Helper / proof | Result |
|----|----------------|--------|
| T23.kani.1 | `checked_block_len_never_panics` — count×size arithmetic never wraps/OOMs over `u32⁶ × {4,8}` | ✅ 0/549, 3.8 s |
| T23.kani.2 | `take` / `skip` / `skip_within_remaining_cannot_truncate` — `Cursor` bounds (×3) | ✅ 0/(472,444,461) |
| T23.kani.3a | `type_index_guard_is_sound` — transition `type_index < typecnt` (`first_oob_type_index`) | ✅ 0/127, 0.15 s |
| T23.kani.3b | `abbr_index_guard_prevents_oob_slice` — designation index **slice-safety** (`abbr_index_slice_safe`) | ✅ 0/15, 0.07 s |
| T23.kani.3f.1 | `rfc_designation_index_valid…` — designation-index **RFC validity** (`idx<charcnt`+NUL+`charcnt≠0`), proven to strictly imply .3b slice-safety | ✅ 0-fail, 0.01 s |
| T23.kani.3f.2 | `isdst_byte_valid_is_exact` — `isdst` octet ∈ {0,1} | ✅ 0-fail, 0.01 s |
| T23.kani.3f.3 | `indicator_pair_valid_is_exact` — indicator octets ∈ {0,1} + `isut=1 ⇒ isstd=1` | ✅ 0-fail, 0.01 s |
| T23.kani.3f.4 | `utoff_structural_valid_excludes_unnegatable_min` — `utoff ≠ i32::MIN` (⇒ negation can't overflow) | ✅ 0-fail, 0.01 s |

- **Cannot witness:** unbounded behaviour · anything outside each harness's bound · semantic / civil-time
  correctness.
- **Non-claims:** a bounded proof is bounded; not a universal correctness proof; the `3f.*` proofs verify
  the **predicates** — *enforcing* them inside `rfc9636::validate` is the tracked next step
  (`../claim-boundary-map.md`).
- **Receipts:** `receipts/RECEIPT-2026-06-02.md` (kani.1/.2/.3a/.3b — 6 proofs) · `receipts/RECEIPT-2026-06-03.md`
  (the 4 reduced-surface standards-precision helpers, 3f.1–3f.4).
- **Cross-reference:** `reports/t17-count-arithmetic-verdict.md` · `docs/panic-policy.md` · `src/tzif/validate.rs`.
- **Next (planned, reduced-surface only):** leap-record block sizing · section-offset accumulation.
