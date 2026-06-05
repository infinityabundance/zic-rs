# audits/flux — refinement types for Rust

> **Status: ✅ RAN clean — `cargo flux` checked zic-rs to completion (exit 0, no errors). No `#[flux::...]` refinement annotations authored, so no specific refinement is proven — a clean pass of the refinement checker, not a verification claim.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** refinement-typed properties (e.g. `idx < len`) on annotated code
- **Command:** `cargo flux`
- **Tool / version:** flux `4d329f2` (2026-05-23); **RAN 2026-06-05**.
- **Input / corpus:** flux-refined functions (authored — see harness/) — candidate: index/length bounds in the TZif parser
- **Result:** ✅ **RAN clean** — `cargo flux` checked zic-rs to completion (exit 0, no errors). No `#[flux::...]` refinement annotations authored → no specific refinement proven (a clean checker pass, not a verification claim).
- **Cannot witness:** unrefined code
- **Non-claims:** no refinements yet = no flux proof; a future deliberate effort
- **Environment requirement:** the flux toolchain
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (RAN clean; no refinements authored).
- **Cross-reference:** `reports/t17-count-arithmetic-verdict.md` · `src/tzif/validate.rs`
