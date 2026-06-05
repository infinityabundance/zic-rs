# audits/cargo-crev — dependency code-review trust

> **Status: ✅ RAN (cargo-crev 0.27.1) — `cargo crev verify` enumerated all 66 deps; **no trusted WoT Ids configured → nothing verified** (an empty-but-real result, not a pass). cargo-vet carries the first-party review layer.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** community code-review / trust coverage of the dependencies
- **Command:** `cargo crev verify`
- **Tool / version:** cargo-crev 0.27.1; **RAN 2026-06-05**.
- **Input / corpus:** the dependency tree
- **Result:** ✅ **RAN** — `cargo crev verify` enumerated all 66 resolved crates; **no trusted WoT Ids → nothing verified** (empty-but-real, not a pass). cargo-vet carries the first-party review layer.
- **Cannot witness:** absence of bugs — trust ≠ correctness
- **Non-claims:** review coverage is social evidence, never a proof of dependency correctness
- **Environment requirement:** network + a configured cargo-crev trust set
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (RAN; empty web-of-trust).
- **Cross-reference:** `docs/security-rewrite-evaluation.md` §D
