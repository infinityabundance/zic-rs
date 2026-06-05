# audits/cargo-scan — dependency effect/capability footprint

> **Status: ✅ RAN (confirmed stub) — the installed `cargo-scan` Rust CLI is a non-functional stub (redirects to a separate Python endpoint); the effect-scan coverage it would give overlaps cargo-geiger + cargo-vet first-party reviews. Confirmed by invocation, not guessed.** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** which dependencies reach the filesystem / network / process / env
- **Command:** `cargo scan`
- **Tool / version:** cargo-scan (installed Rust CLI); **RAN 2026-06-05**.
- **Input / corpus:** the dependency tree (fs · net · process · env surface)
- **Result:** ✅ **RAN (confirmed stub)** — the installed Rust CLI is a non-functional stub (redirects to a Python endpoint); no scan produced. Its coverage overlaps cargo-geiger + cargo-vet.
- **Cannot witness:** logic correctness — it maps capabilities, not behaviour
- **Non-claims:** an effect footprint is a map, not a guarantee an effect is safe
- **Environment requirement:** the cargo-scan effect-analysis tool
- **Receipt:** `receipts/RECEIPT-2026-06-05.md` (RAN; confirmed non-functional stub).
- **Cross-reference:** `docs/effect-boundary-map.md` (zic-rs's own per-module effect map)
