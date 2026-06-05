# T17 Close Receipt — Rust Reliability Hardening

> **Historical receipt — accurate at the T17 close.** Some items here describe the state *then*; later
> receipts supersede or refine them:
> - **T17.FUZZ** sat `pending_capture` at this close. It was **later executed** — `T23.cargo-fuzz.1`
>   (bounded smoke found 3 panic bugs F1–F3) and `T23.cargo-fuzz.2` (fixed 3/3 + regression seeds, re-ran
>   9/9 clean). See `audits/cargo-fuzz/receipts/RECEIPT-2026-06-04*.md`.
> - The test count ("480 tests") is the T17-seal snapshot; the live count is in `STATUS.md`.
> - The Kani count grew from this era's first proofs to **10** (T23.kani.*); cargo-vet advanced to
>   **T23.cargo-vet.4** (29 fully · 1 partial · 36 exempted).
>
> The reliability hardening recorded below remains valid; only these status numbers moved.

## Verdict

**T17 is CLOSED.** Reliability and claim-surface hardening of the Rust implementation is sealed; the only
remaining work is genuinely **external/future** (24h fuzz execution needs nightly/libFuzzer; T18–T23 are
later layers). T17.7 closed the verify-here remainder, so this receipt has no local unfinished work hiding
inside it.

> **The T17 boundary (exact):** *T17 hardens reliability and claim surfaces. It does not claim universal
> replacement parity, signed attestation, whole-tree crash atomicity, or complete external audit coverage.*

## Gate (on the close commit)

- **480 default tests** green (`cargo test`).
- `cargo fmt --check` clean.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo doc --no-deps` clean.
- **CORE.1 = 341 match / 0 mismatch / 0 fail-closed** over 1900..2040 (`bash /tmp/t9sweep.sh`).
- `#![forbid(unsafe_code)]`, no `build.rs`, no new core dependencies, `overflow-checks` in all profiles.

## Sealed substeps

| Substep | What it sealed |
|---|---|
| **T17.1a** | TZif `parse` transition `type_index < typecnt` bounds-guard at the single decode choke point — a malformed/hostile TZif is a typed `Err`, never a latent OOB panic downstream. |
| **T17.1b** | `limits::ResourceLimits` input caps (source-bytes/zone/rule/link/leap counts + link-chain & continuation depth) as bucket-3 `Error::config`; the panic-policy contract (`docs/panic-policy.md`). |
| **T17.2** | CONTRACT.TYPING — the 6 remaining claim-bearing manifest/alias-map free strings → typed enums (JSON byte-identical, **no schema bump**). |
| **T17.3** | doctor/release-diff failure taxonomy — `ToolVersionStatus`/`HashReadStatus` (doctor **v1→v2**) · `OracleFailureScope` (global vs per-row) · the `--split` exclusive-seam fix (split-year change is `behavior_future`, never double-counted). |
| **T17.4** | install/materialization hardening — per-file crash-durable publish (content fsync + atomic publish + parent-dir fsync, Unix) · leaf TOCTOU races closed (exclusive-create + atomic symlink overwrite). |
| **T17.5** | `CountArithmeticVerdict` — checked `count×element-size` arithmetic + the pre-allocation bound (reject an implausible declared count before any `with_capacity`). |
| **T17.6** | schema/CLI stability policy — 10 schema surfaces + every command classified gate/diagnosis/witness/admission/convenience. |
| **T17.FUZZ** | receipt-bearing fuzz **scaffold** (`fuzz/` — 9 targets, seed corpora, receipt template + RUNS ledger). **Scaffold only; no run admitted (see below).** |
| **T17.7** | verify-here remainder — reliability tests (determinism · locale/TZ isolation · name-as-path) · `SECURITY.md`/`CONTRIBUTING.md`/`CHANGELOG.md`/`RELEASE.md` · `schemas/*.json` + the registry/drift validation test. |

## Evidence artifacts (produced/owned by T17)

- `docs/panic-policy.md` — the no-panic-on-untrusted-input contract + audited assertion sites.
- `docs/risk-register.md` — the 15 claim-boundary risks (the standing armor).
- `docs/install-materialization-contract.md` — path/symlink/hardlink/clobber/temp/rename/fsync policy +
  the exact crash-durability claim + the named TOCTOU residual.
- `docs/schema-compatibility-policy.md` · `docs/cli-compatibility-policy.md` — the public-contract rules.
- `reports/contract-typing-audit.md` — the CONTRACT.TYPING audit table.
- `reports/t17-count-arithmetic-verdict.md` — the 10 checked count/size/offset surfaces.
- `fuzz/README.md` · `fuzz/receipts/TEMPLATE.md` · `fuzz/receipts/RUNS.md` · `fuzz/fuzz_targets/*` (9) ·
  `fuzz/corpus/*` — the fuzzing scaffold.
- `SECURITY.md` · `CONTRIBUTING.md` · `CHANGELOG.md` · `RELEASE.md` — the maintenance/process surfaces.
- `schemas/*.schema.json` (10) + `schemas/README.md` — the published JSON Schemas.
- `tests/reliability.rs` · `tests/schemas.rs` — the determinism/locale/name + schema-registry gates.

## Risks advanced (see `docs/risk-register.md` for the status columns)

- **RISK.PATH.1** — leaf TOCTOU races **closed** (exclusive-create + atomic symlink overwrite); the
  concurrent **parent-component** symlink-swap residual is named precisely (`RequiresOpenatStyleHardening`).
- **RISK.INSTALL.1** — **per-file crash-durable publish** guarded (Unix); whole-tree atomicity is an
  explicit non-claim.
- **RISK.COUNT.1** — **guarded** (checked arithmetic + pre-allocation bound).
- **RISK.DIFF.1** — unknown is never "unchanged" (`behaviour_unassessed` + `OracleFailureScope`).
- **RISK.REPORT.1** — public-contract rules + the report-as-attestation non-claim.
- **RISK.RESOURCE.1** — input-dimension caps (`limits::ResourceLimits`).

## Fuzz status (exact boundary)

> *T17.FUZZ created a real fuzzing scaffold and target inventory, but **no fuzz-run result is admitted by
> T17**. All fuzz targets are `pending_capture` until a receipt records toolchain, target, corpus hash,
> duration, result, and artifacts.*

The scaffold is built and inert to the gate; `fuzz/receipts/RUNS.md` lists every target `pending_capture`
(`manifest_json` is `surface_absent` — manifests are write-only). The 24h campaigns are an operator/lab
task (nightly + libFuzzer + network).

## Explicit non-claims (carried forward)

- **No whole-tree crash-atomic install** claim (per-file durable publish only).
- **No closure of the parent-component symlink-swap race** without `openat`/`O_NOFOLLOW` (forbidden:
  needs `unsafe`/a dep).
- **No fuzz-run / fuzz-duration claim** (scaffold only; runs `pending_capture`).
- **No schema instance-validation claim in the core** (the core has no JSON-Schema validator dependency;
  instance-validation is an audit-suite task). The dep-free `tests/schemas.rs` guards registry/drift only.
- **No signed-attestation claim** (reports are `unsigned_local_report`).
- **No universal drop-in `zic` replacement** claim (behaviour parity is exactly CORE.1 over 1900..2040;
  parity/operational axes stay separate).
- **No civil-time-truth claim** (zic-rs compiles admitted IANA tzdb source; it does not define civil time).

## Future owners

| Layer | Owner |
|---|---|
| Source / knowledge provenance (source ledger, Wayback/archive.today, copyright policy) | **T18** |
| Trust front door (`TRUST.md`) · effect-boundary map · audit-readiness · maintenance policy · replacement-readiness ladder · not-yet-ready · drop-in-compat · misuse-resistance | **T19** |
| Security personas (per-persona may/may-not-conclude) | **T20** |
| Packaging gauntlet (Debian/Alpine/Arch/openSUSE-SLES/Nix/Gentoo) | **T21** |
| Performance / resource ledger | **T22** |
| Audit suite (`audits/` — the operator-authoritative tool list, receipt-bearing) | **T23** |
| 24h fuzz execution receipts | operator/lab (nightly + libFuzzer) → `fuzz/receipts/RUNS.md` |

**NEXT: T18** (source/knowledge provenance). T17 CLOSED.
