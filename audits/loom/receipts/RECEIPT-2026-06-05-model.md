# RECEIPT — loom — state: RAN (clean, bounded interleaving model) — 2026-06-05

**Disposition (typed): `ran_clean` — corrects the prior wrong `not_applicable_by_design`.** zic-rs DOES have
lock-free shared state, and loom modeled its invariant across all interleavings.

- **What was wrong before:** the old receipt said "no concurrent shared state." False — zic-rs has **two
  `static AtomicU64` sequence counters** (`src/fs/atomic_write.rs:45 TEMP_SEQ`, `src/fs/output_tree.rs:194
  SEQ`) that generate unique temp-file names during atomic writes (`fetch_add(1, Relaxed)` -> `.{base}.tmp.{pid}.{seq}`).
  (dsfb-gray independently flagged the same statics as P10-6 / JPL-R4.)
- **Tool:** loom 0.7. **Command:** `RUSTFLAGS="--cfg loom" cargo test` in the **detached** harness
  `audits/loom/harness/` (own `[workspace]` so loom never enters the main crate's dep/vet/audit surface —
  mirrors `fuzz/`). **Host:** x86_64 Linux. **Exit:** 0. **Raw:** `../loom/harness/receipts/raw-2026-06-05.txt`.
- **Model:** `loom::model(|| …)` spawns 2 `loom::thread`s, each `fetch_add(1, Relaxed)` on a shared
  `Arc<AtomicU64>`; asserts the two returned values are **distinct across every interleaving loom explores**
  (a duplicate would mean a temp-name collision). **Result: 1 test passed, 0 failed** — the unique-sequence
  invariant holds under all interleavings.
- **What this proves:** the intra-process **temp-name uniqueness invariant** the atomic-write TOCTOU-safe
  path relies on (`create_new`/`hard_link` exclusivity) cannot be broken by concurrent `fetch_add` callers.
- **HONEST scope/non-claims:** loom can't instrument a `static` directly (loom atomics aren't const-static),
  so this models the exact `fetch_add(1, Relaxed)` ALGORITHM the real counters use (standard loom
  methodology), not the whole fs pipeline. **Inter-process** collisions are handled separately by PID +
  `create_new` exclusivity (not a loom concern). A bounded model of the concurrency-relevant invariant, not
  a proof of the entire write path.
