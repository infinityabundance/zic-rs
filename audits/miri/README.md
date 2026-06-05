# audits/miri — undefined-behaviour detection

> **Status: ✅ RUN (advisory) — clean over the tzif core (0 UB).** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** undefined behaviour / leaks / some unsoundness classes exercised by the tests
- **Command:** `MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib tzif`
- **Tool / version:** miri (nightly component), run 2026-06-02.
- **Input / corpus:** the **`tzif` byte-handling core** unit tests (parse/validate/data-block/writer) under the MIR interpreter.
- **Result:** **16 passed · 0 failed — NO undefined behaviour**, incl. the bounds-guard + count-arithmetic + type-index-OOB tests → `receipts/RECEIPT-2026-06-02.md` (+ `raw-2026-06-02.txt`).
- **Cannot witness:** logic · performance · UB the tests do not reach
- **Non-claims:** expected low-yield under `#![forbid(unsafe_code)]` (no `unsafe` to mis-use), but it validates std-API usage; a clean run is not a logic proof
- **Environment requirement:** a nightly toolchain with the miri component
- **Receipt:** `receipts/RECEIPT-2026-06-02.md` (✅ RUN, clean over the tzif core; full-suite miri is a larger pass).
- **Cross-reference:** `cargo-valgrind/` · `src/lib.rs` (`#![forbid(unsafe_code)]`)
