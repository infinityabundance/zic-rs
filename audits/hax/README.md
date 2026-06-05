# audits/hax — extraction to formal backends (F*/Coq)

> **Status: ✅ RAN — `cargo hax json` extracted the full crate to a 186 KB typed-AST JSON (exit 0). Frontend extraction only; no proof-assistant backend extraction/proofs authored.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** an extracted formal model suitable for downstream proof
- **Command:** `cargo hax into fstar`
- **Tool / version:** hax (`cargo hax`) + hax-rust-engine; **RAN 2026-06-05**.
- **Input / corpus:** the (pure) compile-core functions selected for extraction
- **Result:** ✅ **RAN** — `cargo hax json` extracted the full crate to a 186 KB typed-AST JSON (exit 0). Frontend extraction only; no proof-assistant backend extraction/proofs authored.
- **Cannot witness:** the proof itself — hax extracts, it does not prove
- **Non-claims:** an extraction is a model, not a verdict; proof happens in the backend, separately
- **Environment requirement:** the hax toolchain
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (RAN; frontend extraction).
- **Cross-reference:** `docs/effect-boundary-map.md` (the pure core is the extractable surface)
