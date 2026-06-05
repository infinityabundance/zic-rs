# audits/cargo-audit — RustSec advisory scan

> **Status: ✅ RUN (advisory) — clean (0 vulnerabilities).** An audit folder existing is not an audit result — a result is admitted only by a
> receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit · findings ·
> fixed · residual · non-claims · next-owner). See `../README.md`.

- **Scope (what it witnesses for zic-rs):** whether any pinned dependency has a known RustSec advisory or is yanked
- **Command:** `cargo audit`
- **Tool / version:** cargo-audit 0.22.1 (RustSec DB: 1102 advisories), run 2026-06-02.
- **Input / corpus:** `Cargo.lock` — **67 crate dependencies**.
- **Result:** **0 vulnerabilities / 0 warnings** → `receipts/RECEIPT-2026-06-02.md` (+ `raw-2026-06-02.txt`).
- **Cannot witness:** logic bugs · wrong output · `unsafe` correctness — an advisory lookup, not an analysis
- **Non-claims:** a clean run proves *no known-advisory dep at scan time*, not *no vulnerability*
- **Environment requirement:** network access to the RustSec advisory DB
- **Receipt:** `receipts/RECEIPT-2026-06-02.md` (✅ RUN, clean).
- **Cross-reference:** `docs/security-rewrite-evaluation.md` §D/§E · `docs/maintenance-policy.md` §5
