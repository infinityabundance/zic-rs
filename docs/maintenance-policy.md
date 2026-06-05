# Maintenance policy (T19)

> What it takes to **inherit and operate** zic-rs over a 10–20-year horizon: the platform/toolchain floor,
> the release and tzdb-intake cadence, the **emergency tzdb-release** procedure, the security-handling
> contact path, and the deprecation/refresh discipline. This is the steward's contract — the answer to *"can
> a serious team depend on this without the original author?"* Pairs with `RELEASE.md` (the per-cut
> checklist), `CONTRIBUTING.md` (the gate + doctrine), `SECURITY.md` (defect policy), and
> `docs/{schema,cli}-compatibility-policy.md` (the stability contracts).

## 1. Toolchain & platform floor

- **MSRV:** Rust **1.74**, pinned in `Cargo.toml`; raised only in a minor release with a CHANGELOG note. `overflow-checks = true` in release (arithmetic overflow is a typed/aborting failure, never UB).
- **No `build.rs`, no `unsafe`** (`#![forbid(unsafe_code)]`), no network at build or test, lean deps (`clap`, `thiserror`, `tempfile`; `tz-rs` dev-only/optional). New core dependencies are a deliberate, justified decision, not a convenience.
- **Platforms:** the **compiler core is platform-neutral** (pure `bytes → TZif bytes`); install surfaces (symlink/`--mode`/localtime) are `cfg(unix)`-gated and **fail closed** off-Unix (`docs/platform-portability.md`, `docs/effect-boundary-map.md`). Cross-target `cargo check` (musl/Windows/macOS/wasm-library) is the portability guard (CI-wired in T17/T23).

## 2. Release cadence & versioning

- **Pre-1.0:** claims are **release-scoped** — a report's claim is bound to the tzdb release it names; historical claims are not auto-forward-ported. 1.0 is defined in `TRUST.md`/roadmap (canonical-zone parity for the current release · documented structural policy · supported CLI subset · schema stability · warning taxonomy · fuzzing · security policy · no known panic on untrusted input).
- **Schema/CLI stability:** governed by `docs/schema-compatibility-policy.md` (bump `vN` on a *meaning* change; additive fields don't bump) and `docs/cli-compatibility-policy.md` (the 0/1/2 exit taxonomy; each command classified gate/diagnosis/witness/admission/convenience). Diagnostic codes (`ZIC001`–`ZIC026`) are **append-only, never reused**; severity only tightens across a major version.
- **Every cut** runs the gate (`fmt`/`clippy -D`/`test`/`doc` + CORE.1 sweep) and records what is hashed (`RELEASE.md`).

## 3. tzdb-intake cadence (the recurring core work)

A new tzdb release is admitted by a **delta review, not just a hash** (the `release_delta_review_hash`
discipline, `reports/t12_5a2-reference-admission.md` is the worked example for 2026b):

1. Fetch the official archive; **verify the OpenPGP signature** (fingerprint-anchored) and hash-pin every required file.
2. Diff vs the prior admitted release: NEWS · `Makefile` · `ziguard.awk` · `backward`/`backzone` · new/removed policy axes · `zic` diagnostic/format changes (`NewsDeltaKind`).
3. Re-run CORE.1 against the new release's reference `zic`/`zdump`; record the behaviour delta with `release-diff`.
4. Add the release as a **new row** in the release-admission matrix (`docs/build-profile-parity.md`) — support is per-release, never globally inferred. 2026b is currently the only admitted release.

## 4. Emergency tzdb-release procedure (short-notice law changes)

IANA ships emergency releases on short notice (RFC 6557; Morocco/Gaza-style late changes). The fast path:

1. Treat freshness as **operational**, not a correctness claim — zic-rs never claims legislative-update timeliness (`RISK.TIME.1`; `does_not_claim_emergency_legislative_update_freshness`).
2. Run the §3 intake on the emergency archive; if a full delta review can't complete in time, admit it **data-only** with that status recorded — never silently promote it to a full feature-profile admission.
3. `release-diff --old <prev> --new <emergency> --zone-list <affected>` to surface exactly which zones/transitions changed (the rebuild signal for downstreams/bundles, T21).
4. Publish the new admitted row + a CHANGELOG entry; the claim stays scoped to that release.

## 5. Security handling

- Report path and what counts as a defect: `SECURITY.md` (claim-boundary bugs — panic on hostile input, path/materialization escape, resource exhaustion, wrong-TZif-with-valid-report, unknown-as-unchanged, oracle confusion, report-as-attestation, provenance overclaim — are reportable, alongside memory/`unsafe` issues).
- Advisories are issued per the stability contracts; a fix that changes a claim surface bumps the relevant schema/CLI version and is recorded in `docs/risk-register.md`.
- Dependency posture is audited via the `audits/` suite (T23: cargo-audit/cargo-vet/cargo-deny/cargo-geiger/…), receipt-bearing when run.

## 6. Deprecation & evidence-refresh discipline

- **Deprecation:** a command/flag is deprecated for one minor cycle (warned, still working) before removal in a major; schema ids are never silently re-meant (`docs/schema-compatibility-policy.md`).
- **Vendor-lab refresh** (`docs/vendor-lab-refresh-policy.md`, T23-owned): refresh receipts on a major tzdb release · annually · on a distro-major / glibc↔tzcode boundary crossing / package-ownership change. **Historical rows are append-only — never overwritten** (a receipt is dated evidence, not mutable state).
- **Documentation is additive** (the standing evidence-density doctrine): docs grow with more evidence, receipts, and explicit non-claims — reorganization adds navigation, never deletes or narrows evidence.

## 7. Non-claims

- This policy states the **intended** cadence and floor; CI-wiring of every gate (grep gate, cross-target matrix, SBOM) is tracked in T17/T23 and is **not** claimed as fully wired here.
- A maintenance policy existing is **not** a guarantee of an active maintainer team — it is the contract a steward would operate under, written so the project is inheritable.
- tzdb-intake cadence is a **procedure**, not a freshness SLA; zic-rs claims no update-timeliness guarantee.
