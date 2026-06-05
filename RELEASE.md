# Release process & checklist

zic-rs is **pre-1.0**; this is the process for cutting a release once one is made, and the checklist of
**what is verified, recorded, and hashed at a cut** so a release is itself an auditable artifact. It pairs
with `CHANGELOG.md`, [`docs/schema-compatibility-policy.md`](docs/schema-compatibility-policy.md), and
[`docs/cli-compatibility-policy.md`](docs/cli-compatibility-policy.md).

## Gate (must be green on the release commit)

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                                  # default suite (474+ green)
cargo doc --no-deps
bash /tmp/t9sweep.sh                         # CORE.1 → 341/0/0 over 1900..2040
cargo build --locked --release               # reproducible, no-network, no build.rs
```

## Release checklist (recorded at the cut)

Record each of these in the release notes / a `reports/release-<version>.md` so the release is provable:

- **version + provenance:** the crate version, the git commit, `rustc`/toolchain, target(s) built.
- **tzdb provenance:** the admitted tzdb release (currently `2026b`), its archive hash + signature status
  (the T12.5a.2 admission receipt), and the `2026b-dirty`-vs-pristine distinction if relevant.
- **CORE.1 receipt:** the sweep result (341/0/0) over 1900..2040.
- **report-schema versions:** the `schema` id of every emitted/ingested surface (the
  [`schemas/`](schemas/) registry — confirm `tests/schemas.rs` green; no drift, no orphans). Note any
  schema bump in `CHANGELOG.md` with its meaning-change rationale.
- **sample artifacts (hashed):** a `support-report` + `structural-report` + a `doctor` sample + a
  `release-diff` sample against the prior tzdb (when available), each with its sha256.
- **vendor matrix pointer:** the vendor-oracle lab state (17 rows / 19 receipts) +
  `../zic-rs-vendor-oracle-lab/IMAGE-PROVENANCE.md`.
- **fuzz status:** the `fuzz/receipts/RUNS.md` state — a **bounded smoke ran** (T23.cargo-fuzz.1/.2: found+fixed
  F1–F3, re-ran 9/9 clean; `audits/cargo-fuzz/receipts/RECEIPT-2026-06-04*.md`); a coverage-saturating campaign
  is still pending. Link any further run receipts.
- **risk-register status:** the current per-risk status column ([`docs/risk-register.md`](docs/risk-register.md)).
- **packaging:** `Cargo.lock` committed; `cargo package --list` reviewed; `cargo vendor` policy noted;
  license/REUSE (`MIT OR Apache-2.0`) + `ACKNOWLEDGEMENTS.md` current.
- **CHANGELOG.md:** an entry for the version (append-only; the `[Unreleased]` section graduates).

## Doctrine

- A release **does not** widen any claim: it is a snapshot of the *measured* state. Behaviour claims stay
  release-scoped to the admitted tzdb release named in the reports.
- **Reproducible + no-network:** `cargo build --locked` with no `build.rs` and no network at build/test
  (the only network step in the project's history is the *admission* of a signed, hash-pinned tzdb
  archive, recorded separately — never at ordinary build time).
- A release is **evidence, not attestation**, unless and until releases are signed (a future item; reports
  remain `unsigned_local_report` today — see `RISK.REPORT.1`).
- Schema/CLI compatibility across releases follows the stability policies; an old report reader should fail
  closed on an unknown `schema` id rather than guess.

## Emergency tzdb-release handling (timezone law can change suddenly)

When IANA cuts an out-of-cycle release (a sudden DST-law change):
1. **Admit** the new release per the T12.5a.2 discipline (fetch · verify signature · hash-pin · delta
   review) — do **not** silently generalize a prior release's claims to it.
2. Re-run CORE.1 + the support/structural sweeps against the new release; record a new receipt.
3. Run `release-diff <prior> <new>` to surface the changed identifiers (behaviour past/future).
4. Cut a release noting the new admitted release and the diff; update `CHANGELOG.md`.

The admitted-release matrix (only `2026b` today) is the gate: support is **per-release**, recorded, never
inferred.
