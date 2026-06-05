# audits/cargo-valgrind — runtime memory check (memcheck)

> **Status: ◐ RUN (inconclusive) — environment-incompatible (valgrind SIGILLs on `/bin/true` here).** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** runtime memory errors / leaks during the tests
- **Command:** `valgrind --leak-check=full ./target/release/zic-rs compile … --all-supported` (the `cargo-valgrind` subcommand is absent; `valgrind` itself is present).
- **Tool / version:** valgrind, run 2026-06-02 (kernel 7.0.9-cachyos).
- **Input / corpus:** a full compile under memcheck.
- **Result:** **INCONCLUSIVE** — valgrind SIGILLs in early startup (0 allocs reached), reproduced on a generic-CPU build **and on `/bin/true`** → valgrind is incompatible with this kernel/glibc, **not** a zic-rs memory finding → `receipts/RECEIPT-2026-06-02.md` (+ `raw-2026-06-02.txt`).
- **Cannot witness:** logic · performance · anything the tests do not exercise
- **Non-claims:** expected low-yield under safe Rust + `#![forbid(unsafe_code)]`, but still a real runtime witness
- **Environment requirement:** valgrind installed (Linux)
- **Receipt:** `receipts/RECEIPT-2026-06-02.md` (◐ RUN, inconclusive — env-incompatible; real memcheck needs a compatible host / static musl build).
- **Cross-reference:** `miri/` (static UB companion) · `docs/security-rewrite-evaluation.md` §F
