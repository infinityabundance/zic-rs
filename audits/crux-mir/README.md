# audits/crux-mir — symbolic execution of MIR

> **Status: ✅ RAN — `Overall status: Valid`, **5/5 goals proved** on 3 zic-rs `#[crux::test]` harnesses (cursor-bounds · type-index guard · indicator pairing), a second-engine (Crucible+z3) corroboration of the Kani proofs.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** properties holding over symbolic inputs on the written harnesses
- **Command:** `cargo crux-test`
- **Tool / version:** crux-mir 0.12.0.0.99 (Crux 0.9.0.0.99) + z3 4.15.3; **RAN 2026-06-05**.
- **Input / corpus:** symbolic-execution harnesses (authored — see harness/) — candidate: the bounded parser
- **Result:** ✅ **Overall status: Valid — 5/5 goals proved** on 3 zic-rs `#[crux::test]` harnesses (cursor-bounds · type-index guard · indicator pairing); second-engine (Crucible+z3) corroboration of Kani.
- **Cannot witness:** behaviour outside the harnessed surface / unbounded inputs
- **Non-claims:** no harness = no result; symbolic execution is bounded by the harnesses written
- **Environment requirement:** the crux-mir toolchain (Galois)
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (RAN + proved; harnesses in `harness/`).
- **Cross-reference:** `kani/` (bounded-model-checking companion) · `src/tzif/validate.rs`
