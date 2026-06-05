# audits/cargo-geiger — unsafe-usage census

> **Status: ✅ RUN (advisory) — census captured (zic-rs own code 0 unsafe).** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** the count/location of `unsafe` in the tree
- **Command:** `cargo geiger`
- **Tool / version:** cargo-geiger (`--all-features`), run 2026-06-02.
- **Input / corpus:** the whole dependency tree + zic-rs itself.
- **Result:** zic-rs own code **0 unsafe** (forbid); dependency surface quantified (unsafe in libc/rustix/getrandom/linux-raw-sys) → `receipts/RECEIPT-2026-06-02.md` (+ `raw-2026-06-02.txt`).
- **Cannot witness:** whether any `unsafe` is *incorrect* — a census, not a verifier
- **Non-claims:** core is `#![forbid(unsafe_code)]` (0 unsafe by construction); geiger quantifies the *dependency* unsafe surface, which forbid does not cover
- **Environment requirement:** the cargo-geiger tool
- **Receipt:** `receipts/RECEIPT-2026-06-02.md` (✅ RUN).
- **Cross-reference:** `src/lib.rs` (`#![forbid(unsafe_code)]`) · `docs/security-rewrite-evaluation.md` §F
