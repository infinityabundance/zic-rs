# T17.5 — CountArithmeticVerdict

> **Rule.** *Every count, size, and offset derived from input is **hostile until checked**.* TZif header
> count fields (`timecnt`/`typecnt`/`charcnt`/`leapcnt`/`isstdcnt`/`isutcnt`) are untrusted `u32`s; the
> arithmetic that turns them into allocations, slice lengths, indexes, and cursor advances must be
> **checked before** it is used — never wrap (which on a 32-bit `usize` would *under*-compute a length and
> mis-slice), never panic (an `overflow-checks` trap is a controlled abort, still a DoS), and never
> pre-allocate gigabytes from a tiny file.
>
> **Scope: this is reliability arithmetic — safe rejection, not semantics.** It is
> *not* full TZif semantic validation, *not* reader-compatibility, *not* civil-time truth. A
> `CountArithmeticVerdict` failure means "the declared counts/sizes are not physically admissible against
> the bytes present," producing a typed `Err`, not a crash. Semantic/structural verdicts remain the
> separate T15.4 / CORE.1 surfaces. Built on the T17.1a transition-`type_index < typecnt` guard, which
> this generalizes to the whole count surface.

All of this lives at the single decode choke point `src/tzif/validate.rs::parse` (used by every consumer:
`compare::semantic`, `compile::leap`, the `tzif-validate` RFC-9636 validator over arbitrary `--input`, and
the writer's round-trip tests), so hardening it protects the entire crate at once.

## Checked count / size / offset surfaces

| # | Surface | Hazard if unchecked | Guard (T17.5 unless noted) | Status |
|---|---|---|---|---|
| 1 | `block_len = Σ (count × element-size)` | `u32 × size` overflows `usize` on 32-bit → wrap → under-computed length → mis-slice; or `overflow-checks` panic | `checked_block_len` — every term `count_mul` (`checked_mul`) + `checked_add` accumulation → typed `Err` on overflow | **guarded** |
| 2 | `Vec::with_capacity(timecnt)` (transition times) | a huge declared `timecnt` pre-reserves multi-GB **before** any byte is read → OOM/abort even on a tiny file | **pre-allocation bound**: `checked_block_len ≤ cursor.remaining()` checked *before* any `with_capacity`, so every count is bounded by the bytes actually present | **guarded** |
| 3 | `Vec::with_capacity(typecnt)` (ttinfo) | same OOM-from-declared-count | covered by the same #2 pre-check (the block length includes `typecnt × 6`) | **guarded** |
| 4 | `Vec::with_capacity(leapcnt)` (leap records) | same OOM-from-declared-count | covered by the same #2 pre-check (includes `leapcnt × (time_size+4)`) | **guarded** |
| 5 | per-element reads `take(time_size)` / `take(1)` / `take(6)` / `take(charcnt)` | read past end of buffer → panic / OOB slice | `Cursor::take` — `checked_add(pos, n)` + `end > buf.len()` → `Err` (pre-existing, bounds-safe) | **guarded** |
| 6 | v1-block skip `pos += block_len` | `pos += huge` overflows on 32-bit / lands a silent out-of-range cursor | `Cursor::skip(checked_block_len(..)?)` — `checked_add` + `end > buf.len()` → `Err` (T17.5) | **guarded** |
| 7 | transition `type_index` → `types[idx]` | OOB index → panic in `compare::semantic`/`compile::leap` | `parse` rejects `type_index ≥ typecnt` at the choke point | **guarded (T17.1a)** |
| 8 | designation `desigidx` → `table[idx..]` | OOB index → panic | `read_cstr` rejects `idx > table.len()` | **guarded (pre-existing)** |
| 9 | `u32 as usize` / `i32`/`i64 from_be_bytes` conversions | lossy on 32-bit / sign confusion | conversions are all *widening* on 64-bit; on 32-bit the #1/#2 checks bound the values before any narrowing use; `from_be_bytes` is exact fixed-width | **guarded (by #1/#2/#5)** |
| 10 | `overflow-checks = true` (all profiles) | wrapping arithmetic elsewhere | a last-resort backstop: any unchecked overflow traps rather than wraps (a controlled abort, not corruption) — but the checks above mean untrusted counts never reach it | **backstop** |

**ttisstd/ttisut consistency** (the indicator counts must be 0 or `typecnt`) and the **ascending /
typecnt≥1** invariants are *structural* checks owned by the T15.4 `rfc9636` validator (a separate verdict
axis), not count-*arithmetic* — listed here only to mark the boundary: T17.5 makes the counts *safe to
read*; T15.4 judges whether they are *RFC-conformant*.

## Hostile fixtures / tests

In `src/tzif/validate.rs::tests`:

- **`implausibly_large_declared_count_is_rejected_not_ooming`** — a v1 header claiming `timecnt =
  1_000_000_000` with an 8-byte body → typed `Err` ("declared counts require more bytes than the input
  contains") **before** any billion-element allocation. This is the headline DoS guard.
- **`count_block_len_is_checked_arithmetic`** — `checked_block_len` with `timecnt = u32::MAX` returns a
  large finite value (~38.6 e9), proving the arithmetic does not wrap (a wrap would yield a small value
  that then *passes* a `≤ remaining` check and mis-slices).
- **`out_of_range_transition_type_index_is_rejected_not_panic`** (T17.1a) — OOB `type_index` (1 and 255 vs
  `typecnt 1`) → `Err`, never a panic.
- **`valid_transition_type_index_parses` / `round_trip_*`** — the in-range / valid paths still parse
  (the checks reject only the implausible, never the legitimate; CORE.1 unchanged at 341/0/0).

## Acceptance (met)

TZif count/size/offset arithmetic is **checked before allocation, slicing, indexing, iteration, and cursor
advance**; hostile count fixtures **reject with typed `Err`** rather than panic, wrap, or OOM; the only
input-derived indexing (`type_index`, `desigidx`) is guarded before use; `overflow-checks` remains a
backstop, not the primary defence. Gate: `fmt` ✅ · `clippy --all-targets -D warnings` ✅ · **474 tests** ✅
· `doc` ✅ · **CORE.1 341/0/0** ✅. No schema change (a reader-hardening pass; the bytes/JSON are
unchanged). Pairs with `docs/panic-policy.md` (no panic on hostile input) and
`docs/install-materialization-contract.md` (T17.4).

> **Boundary, one line:** *T17.5 makes every count/size/offset from input safe to compute with; it does
> not judge whether the file is semantically correct — that stays CORE.1 (behaviour) and T15.4
> (RFC-9636 structure).*

## Formally proven (T23 / Kani — 2026-06-02)

The T17.5 arithmetic is no longer only tested on hostile fixtures — it is **bounded-model-checked**
(`audits/kani/receipts/RECEIPT-2026-06-02.md`):

- **T23.kani.1 `checked_block_len_never_panics` ✅** — over the **entire** symbolic count space
  (`timecnt`/`typecnt`/`charcnt`/`leapcnt`/`isstdcnt`/`isutcnt` = arbitrary `u32`, `time_size ∈ {4,8}`), the
  guard never panics, never overflow-wraps, never allocates (549 checks, 0 failed).
- **T23.kani.2 cursor decomposition ✅** — `take`/`skip` position arithmetic cannot overflow-wrap or read/advance
  past the buffer end, and `skip(n) | n ≤ remaining()` is always `Ok` — i.e. the **`count×size ≤ remaining`
  pre-allocation bound is formally sound** (a passing check guarantees the bytes are physically present, so a
  later read cannot run off the end). All 0-fail.

These are **bounded** proofs (their exact domains/bounds are in the Kani receipt) and do **not** establish
full-parser correctness, semantic/RFC-9636 validity, or reference parity — the full-parser entrypoint harness
(T23.kani.3) is inconclusive (solver timeout). They prove precisely the *arithmetic-safety* claim this verdict makes.
