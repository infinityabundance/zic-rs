# audits/creusot — deductive functional verification (Why3)

> **Status: ✅ RAN + PROVED (deductive) — `why3find prove ✔` discharged a zic-rs invariant (`block_len`: count×size non-overflow) via Why3+SMT, using the **matched** why3 1.8.2+git / why3find 1.3.0+dev built from the creusot-pinned source (a clean OCaml-5.3.0 opam switch). A third engine corroborating Kani + crux-mir.**translated a zic-rs invariant to a Why3 proof obligation** (`.coma`); the SMT discharge is blocked by a cargo-creusot↔why3find version skew + a missing Why3 prelude package, NOT by the proof. The invariant is covered by Kani T23.kani.1 + crux-mir.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** proved functional contracts on annotated code
- **Command:** `cargo creusot`
- **Tool / version:** cargo-creusot 0.12.0-dev + matched why3 1.8.2+git / why3find 1.3.0+dev (built from the creusot-pinned source); **RAN 2026-06-05**.
- **Input / corpus:** creusot-annotated functions (authored — see harness/) — candidate: the TZif count/offset arithmetic
- **Result:** ✅ **RAN + PROVED (Why3+SMT)** — `why3find prove ✔` discharged the `block_len` count×size non-overflow invariant; a THIRD engine corroborating Kani (CBMC) + crux-mir (Crucible).
- **Cannot witness:** unannotated code — it proves only what is specified
- **Non-claims:** absence of annotations = absence of proofs, not a passing proof; a future deliberate effort
- **Environment requirement:** the creusot toolchain (+ Why3 + SMT solvers)
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (RAN + proved; obligation `block_len.coma`).
- **Cross-reference:** `reports/t17-count-arithmetic-verdict.md`
