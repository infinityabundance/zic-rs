# audits/loom — concurrency interleaving testing

> **Status: ✅ RAN — `loom::model` exhaustively proved the `fetch_add` **unique-sequence invariant** (the temp-name counters in `atomic_write.rs`/`output_tree.rs`) across all interleavings. **Corrects the prior wrong `not_applicable_by_design`** — zic-rs DOES have lock-free shared state.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** correctness of concurrent structures under permuted interleavings
- **Command:** `cargo test (loom cfg)`
- **Tool / version:** loom 0.7; **RAN 2026-06-05**.
- **Input / corpus:** shared-state concurrent structures — **zic-rs's compile path is single-threaded; none is claimed**
- **Result:** ✅ **RAN — `loom::model` proved the `fetch_add` unique-sequence invariant** (the `atomic_write.rs`/`output_tree.rs` temp-name counters) across all interleavings. Corrects the prior wrong N/A: zic-rs DOES have lock-free shared state.
- **Cannot witness:** single-threaded logic (essentially all of zic-rs today)
- **Non-claims:** the folder is kept (authoritative list) but **no concurrency claim exists to test**; recorded honestly, not faked green
- **Environment requirement:** loom as a dev-dependency around a concurrency harness
- **Receipt:** `receipts/RECEIPT-2026-06-05-model.md` (RAN + proved; model in `harness/`).
- **Cross-reference:** `docs/effect-boundary-map.md` (the compile path is pure + sequential)
