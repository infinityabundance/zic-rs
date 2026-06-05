# audits/panic-analysis — the panic-primitive surface vs the panic policy

> **Status: ✅ RUN here (advisory; project-internal).** A real receipt lives in `receipts/`. This audit is a
> static census of panic-capable primitives in `src/`, classified against `docs/panic-policy.md`. It is the
> *grep-level* witness; a **proof** of no-panic-on-hostile-input is the job of `../kani/` (bounded model
> checking) and `../miri/` (UB) — see their folders.

- **Scope:** every `panic!` / `todo!` / `unimplemented!` / `unreachable!` / `.unwrap()` / `.expect(` site in
  `src/`, classified **untrusted-reachable** (must return `Result`) vs **internal-invariant** (may assert
  with a named reason) vs **test-only**.
- **Command:** `grep -rn` over `src/` for each primitive (see the receipt for exact commands) + manual
  classification against `docs/panic-policy.md`'s audited allow-list.
- **What it witnesses:** that no *prohibited* primitive (`panic!`/`todo!`/`unimplemented!`) exists on the
  untrusted-input path, and that every `expect`/`unreachable!` is a documented internal invariant.
- **Cannot witness:** that a *reachable* panic is truly unreachable (grep sees syntax, not reachability) —
  that is the bounded proof in `../kani/`. It also does not see panics from indexing/slicing/arithmetic
  (those are covered by `overflow-checks` + the T17.5 count-arithmetic guards + kani).
- **Non-claims:** a clean census is **not** a proof of no-panic; it is evidence that the policy is *held by
  construction at the source level*, to be strengthened by kani/miri receipts.
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (real reproducible `scan.sh` + raw output; supersedes the 2026-06-02 assertion).
- **Cross-reference:** `docs/panic-policy.md` · `reports/t17-count-arithmetic-verdict.md` · `../kani/` · `../miri/`.
