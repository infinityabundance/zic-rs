# audits/dsfb-gray — zic-rs's own gray-box claim-boundary audit

> **Status: ✅ RAN (the real `dsfb-scan-crate` tool) — **60.3% mixed assurance posture** (review-readiness map, NOT a parity contradiction); SARIF+in-toto+DSSE artifacts in `output/`. Replaces the prior hand-written grep summary.** A real receipt lives in
> `receipts/`. `dsfb-gray` is zic-rs's **own** gray-box audit of the thing the project is most at risk
> from: **claim drift** (prose outrunning typed evidence, stale state, a non-claim quietly dropped, a
> report literal un-owned, a risk uncovered, a ledger inconsistent). It is the executable form of the
> same-batch-seal discipline.

- **Scope (the claim-boundary checks):**
  1. **schema/report-literal ownership** — every emitted `schema` id is registered + has a published
     `schemas/*.schema.json` (drift guard: `tests/schemas.rs`).
  2. **risk-register coverage** — the `RISK.*` rows exist and each carries a status + a non-claim.
  3. **non-claim presence** — `negative_capabilities` are emitted with an `enforced_by` guard.
  4. **ledger consistency** — every `source-ledger` source id has an `archive-ledger` row (no orphan/missing).
  5. **stale-state scan** — current-state banners / counts / schema-version comments match reality
     (the same class as the two T19-seal stale fixes: a doctor-`v1` comment, an ecology banner).
- **Command:** `cargo test --test schemas` + `grep`/`awk` consistency checks (see the receipt).
- **What it witnesses:** that the *claim surface* is internally consistent — schemas owned, risks covered,
  non-claims present, ledgers matched, no stale current-state text.
- **Cannot witness:** semantic correctness of the compiler (that is CORE.1 / the oracle), nor external-tool
  findings (those are the other `audits/` folders).
- **Non-claims:** a clean gray-box pass means *the claims are consistent with the evidence*, not that the
  claims are *true* — truth is the underlying receipts/tests/oracle.
- **Receipt:** `receipts/RECEIPT-2026-06-05.md`.
- **Cross-reference:** `tests/schemas.rs` · `docs/risk-register.md` · `docs/schema-compatibility-policy.md` ·
  the source/archive ledgers · `CONTRIBUTING.md` (the same-batch-seal rule).
