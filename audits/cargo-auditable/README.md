# audits/cargo-auditable — embedded dependency manifest

> **Status: ✅ RUN (advisory) — embedded manifest produced.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** that the artifact carries a recoverable embedded dependency list (SBOM-ish)
- **Command:** `cargo auditable build --release`
- **Tool / version:** cargo-auditable, run 2026-06-02.
- **Input / corpus:** the built `zic-rs` release binary (2,246,536 bytes).
- **Result:** built a release binary with a **recoverable embedded dependency manifest** (the 67-crate set) → `receipts/RECEIPT-2026-06-02.md`.
- **Cannot witness:** vulnerabilities themselves (that is cargo-audit over the embedded list)
- **Non-claims:** embedding a manifest is not an attestation; pairs with the T20 §D SBOM item (planned, not present)
- **Environment requirement:** the cargo-auditable build wrapper
- **Receipt:** `receipts/RECEIPT-2026-06-02.md` (✅ RUN).
- **Cross-reference:** `docs/security-rewrite-evaluation.md` §D
