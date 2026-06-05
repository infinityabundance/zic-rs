# Changelog

All notable changes to **zic-rs** are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/); the project is **pre-1.0** (see
[`docs/schema-compatibility-policy.md`](docs/schema-compatibility-policy.md) for how report/schema
versions evolve, and `docs/cli-compatibility-policy.md` for the CLI/exit contract). Behaviour claims are
**release-scoped** to the admitted tzdb release named in the report.

This file is **append-only**: superseded entries are not rewritten. The exhaustive per-substep history
lives in the project plan and the close receipts under `reports/`.

## [Unreleased]

Pre-1.0; no tagged release yet. Current sealed state:

### Core (done / closed)
- **CORE.1 (T7):** all **341 / 341** canonical zones in `tzdata.zi` 2026b behaviour-match reference
  `zic`/`zdump` over `1900..2040` — 341 match · 0 mismatch · 0 fail-closed (the standing gate).
- **T8** structural TZif parity (version byte · abbreviation suffix-sharing · slim/fat); the default
  emission is behaviour-matched fat-style, `--emit-style zic-slim` reproduces slim `zic`.
- **T9–T11** CLI/filesystem/install-mode parity · range/emission (`-b`/`-R`/`-r`) · leap/`right/`/TZif v4
  (opt-in `-L`).
- **T12** build-profile provenance — manifest `zic-rs-compile-manifest-v8`, the source-variant evidence
  arc, the all-IANA release-admission matrix (only 2026b admitted) — **closed** (`reports/t12-close-receipt.md`).
- **T13** warning/diagnostic parity — the executable diagnostic contract `ZIC001`–`ZIC020` — **closed**.
- **T14** hostile-input/parser-edge parity — `ZIC021`–`ZIC025`, incl. a removed panic on untrusted input
  — **closed**.
- **T15** the public conformance engine — `support-report-v4` / `semantic-report-v1` / `tzif-validation-v1`,
  `ZIC026`, 15 guard-enforced non-claims, a bounded `ConformanceStatus` (no global verdict) — **closed**.

### T16 — release ecology (in progress)
- Inventory · `ReferenceBuildProfile` · locator+trust (`ReferenceLocatorKind`/`SignatureTrustModel`) ·
  auxiliary-table validator (`aux-table-validation-v1`).
- **Vendor-oracle receipt admission + no-dep ingestion** + the external lab: **17 ecology rows / 19
  receipts (18 admitted + 1 typed non-admission)** across BSD · illumos · Linux (musl/glibc/source/
  content-addressed/RPM/apk/pacman) · embedded build-system. Findings: two `zic` lineages (the glibc one
  version-stratified, inflection bracketed between glibc 2.39 and 2.40), lineage = a packaging choice
  independent of libc *and* package format, bloat-default its own axis, build-host ≠ runtime-consumer.
- **T16.6** operator tooling: `release-diff` (`release-diff-v1`) + `doctor` (`doctor-v2`).

### T17 — reliability hardening (in progress)
- **T17.1** TZif-read bounds-guard (transition `type_index < typecnt`) + `docs/panic-policy.md` + the
  `limits::ResourceLimits` input caps (bucket-3 `Error::config`).
- **T17.2** CONTRACT.TYPING — the 6 remaining claim-bearing manifest/alias-map free strings → typed enums
  (JSON byte-identical, no schema bump); `reports/contract-typing-audit.md`.
- **T17.3** doctor/release-diff failure taxonomy — `ToolVersionStatus`/`HashReadStatus` (**doctor v1 → v2**)
  · `OracleFailureScope` · the `--split` exclusive-seam fix (split-year change is `behavior_future`, never
  double-counted).
- **T17.4** install/materialization hardening — per-file crash-durable publish (content fsync + atomic
  publish + parent-dir fsync, Unix) + leaf TOCTOU races closed; `docs/install-materialization-contract.md`.
  Whole-tree crash-atomicity + the parent-component symlink-swap race are explicit non-claims.
- **T17.5** `CountArithmeticVerdict` — checked count×element-size arithmetic + the pre-allocation bound;
  `reports/t17-count-arithmetic-verdict.md`.
- **T17.6** schema/CLI stability policy — `docs/schema-compatibility-policy.md` + `docs/cli-compatibility-policy.md`.
- **T17.FUZZ** — `fuzz/` scaffold (9 receipt-bearing targets, seed corpora, receipt template). Run receipts
  were **`pending_capture`** at the T17 seal. *(Later: **T23.cargo-fuzz.1/.2** ran a bounded smoke — found +
  fixed 3 panic bugs F1–F3 with regression seeds, re-ran 9/9 clean; see `audits/cargo-fuzz/receipts/`.)*
- **T17.7** — verify-here remainder: determinism/locale/symlink tests · `SECURITY.md` · `CONTRIBUTING.md` ·
  `CHANGELOG.md` · `RELEASE.md` · `schemas/*.json` + the registry/drift validation test.

### Standing properties
- `#![forbid(unsafe_code)]`, no `build.rs`, minimal dependencies, `overflow-checks` in all profiles;
  deterministic output independent of host `TZ`/`LC_ALL`; **474 default tests** green.

### Not yet claimed (explicit)
- Not a full `zic` replacement; not a runtime `localtime`/CLDR library; not a civil-time authority; not a
  signed attestation; vendor parity only per admitted receipt. See `docs/risk-register.md` +
  `docs/differences-from-reference-zic.md`.
