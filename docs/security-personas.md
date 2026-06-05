# Security-persona interpretation layer (T20)

> For every serious audience, this answers three questions and refuses the fourth: **what zic-rs proves
> for you · what it explicitly does *not* prove for you · where to verify it** — and never lets you
> conclude more. This is the project's anti-overclaim surface: each persona's *may-NOT-conclude* column is
> deliberately the loudest. It **consumes** `docs/risk-register.md`, `SECURITY.md`,
> `docs/security-rewrite-evaluation.md`, `TRUST.md`, `docs/effect-boundary-map.md`,
> `docs/drop-in-compatibility-contract.md`, `docs/not-yet-ready.md`, `docs/misuse-resistance-ledger.md`,
> and `docs/audit-readiness.md` — it does not restate them. Each packet: **fears · evidence to read ·
> commands to run · may conclude · may NOT conclude · remaining risks · future owner.**

Every packet runs against **explicit, named inputs** — the How-to-verify posture never reads system
zoneinfo implicitly (`VerifierCommandPolicy`).

---

## 1. Distro maintainer

- **Fears:** non-reproducible builds · network at build · install escaping DESTDIR · shipping a wrong zoneinfo to users.
- **Read:** `docs/drop-in-compatibility-contract.md` (§"What a packager should actually do") · `docs/install-materialization-contract.md` · `docs/replacement-readiness-ladder.md` (a dedicated `docs/distro-packager-guide.md` is **T16.7-planned**, not yet written).
- **Run:** `cargo build --locked` (no network, no `build.rs`); `zic-rs compile --input <tzdata.zi> --out <staged>`; `structural-report`/`release-diff` vs system `zic`; the CORE.1 sweep.
- **May conclude:** it builds offline + reproducibly, installs only under `--out`, behaviour-matches reference `zic` for 341 canonical 2026b zones over 1900..2040, and is evaluable **side-by-side without installing** (RRL-2).
- **May NOT conclude:** that it is a default-`zic` drop-in (it is **not argv-compatible by design** — subcommands + required `--out`); that other tzdb releases are covered (only 2026b admitted); that a clean side-by-side equals distro-wide approval (that is RRL-4+, not reached).
- **Remaining risks:** `RISK.ADOPT.1` (premature replacement), `RISK.INSTALL.1` (no whole-tree crash-atomicity).
- **Future owner:** T21 (packaging gauntlet receipts).

## 2. Security reviewer

- **Fears:** panic/OOM on hostile input · path escape · a plausible-but-wrong artifact that loads fine.
- **Read:** `docs/security-rewrite-evaluation.md` (threat table A · delta F) · `docs/panic-policy.md` · `docs/risk-register.md` · `reports/t17-count-arithmetic-verdict.md`.
- **Run:** `cargo test` (incl. `pathology_ledger`, `reliability`, `hostile_output_tree`, `tzif::validate` OOB); `zic-rs tzif-validate --input <hostile.tzif>`.
- **May conclude:** safe Rust removes the memory/buffer/count-overflow class by construction; hostile input becomes a typed `Err`, not a panic/OOM; leaf TOCTOU is closed; counts are range-checked before allocation.
- **May NOT conclude:** that zic-rs is DoS-proof (caps bound the *input-size* tail, not all adversarial CPU); that the parent-component symlink-swap race is closed (`RequiresOpenatStyleHardening`); that "no panic found" equals "no panic exists" — a *bounded* fuzz smoke ran (T23.cargo-fuzz.1/.2: found + fixed 3 panic bugs F1–F3, re-ran clean), but it is not a coverage-saturating campaign.
- **Remaining risks:** `RISK.PATH.1` (parent-component race), `RISK.RESOURCE.1` (CPU tail).
- **Future owner:** T23 `audits/` (miri/kani/cargo-fuzz receipts), T20-hardening (openat).

## 3. Supply-chain reviewer

- **Fears:** transitive `unsafe`/yanked/advisory deps · unverifiable build provenance · tampered artifacts.
- **Read:** `docs/security-rewrite-evaluation.md` §D/§E · `Cargo.toml`/`Cargo.lock` · `docs/maintenance-policy.md`.
- **Run:** inspect the lean dependency set; (when CI-wired) cargo-audit/deny/vet via the `audits/` suite; verify per-artifact SHA-256 + recorded oracle/tzdb identity.
- **May conclude:** the dependency surface is small, `#![forbid(unsafe_code)]`, no `build.rs`, no build/test network; every emitted artifact + the reference oracle + the tzdb release are hash-recorded.
- **May NOT conclude:** that an SBOM, SLSA provenance, or signed releases exist **today** (they are named + planned, **not** present — §D); that the transitive graph has been formally audited until the `audits/` receipts exist.
- **Remaining risks:** dependency compromise (mitigated by minimality, not yet by cargo-vet receipts).
- **Future owner:** T23 `audits/` (cargo-audit/auditable/vet/deny/geiger) + a release-signing pipeline.

## 4. SRE / operator

- **Fears:** a silent wrong answer in production · "no change" hiding a real change · misreading a report's exit code.
- **Read:** `docs/cli-compatibility-policy.md` · `docs/misuse-resistance-ledger.md` · `SECURITY.md`.
- **Run:** `zic-rs doctor --format json` (host fitness); `zic-rs release-diff --old A --new B` (change witness); `support-report`.
- **May conclude:** exit codes are a stable taxonomy (0/1/2); a report's exit 0 means *it ran*, never *all clear*; an absent oracle is visible (`behaviour_unassessed` / `oracle_mode`), never silent "no change".
- **May NOT conclude:** that `doctor` exit 0 means host tools are *correct* (only present/typed); that `release-diff` "no rows" means *safe* (it is a witness, not a gate); that any report is a signed certification (`unsigned_local_report`).
- **Remaining risks:** `RISK.DIFF.1`, `RISK.ORACLE.1`, `RISK.REPORT.1` (all guarded — the danger is *misreading*, which the ledger pre-empts).
- **Future owner:** T19 (sealed) — this persona is steady-state operational.

## 5. Embedded / appliance builder

- **Fears:** dragging a toolchain/host tzdata into the image · non-deterministic bundles · no on-device compiler when needed.
- **Read:** `docs/platform-portability.md` · `docs/effect-boundary-map.md` · `docs/drop-in-compatibility-contract.md`.
- **Run:** the library path (`compile_zone_to_bytes`) with **explicit** source bytes, no host read; cross-target `cargo check` (musl/wasm-library).
- **May conclude:** the **compile core is pure** (no fs/process/env; no reference `zic` needed for production compile — only the conformance path needs it); output is deterministic and independent of host `TZ`/`LC_ALL`; copy-mode is the portable baseline.
- **May NOT conclude:** that install features (symlink/`-m`/`-u`) exist off-Unix (they `cfg`-gate + fail closed); that a produced image carries a compiler (the Yocto/Poky target row proved images can be TZif *consumers* with no on-device `zic`).
- **Remaining risks:** platform install-surface gaps (named, fail-closed, not silent).
- **Future owner:** T21 (bundle profiles + `size-report` + bundle hash).

## 6. TZif / standards reviewer

- **Fears:** non-conformant TZif · counted-array bounds violations · footer/version misuse read as conformance.
- **Read:** `docs/structural-parity.md` · the `tzif-validate` report (`zic-rs-tzif-validation-v1`) · `reports/t13/t14-close-receipt.md`.
- **Run:** `zic-rs tzif-validate --input <file.tzif>` (5 separate typed verdicts); `structural-report --reference-zic zic`.
- **May conclude:** emitted TZif is RFC 9636 structurally valid (counted arrays, ordering, index bounds, footer, version semantics), validated as **five separate verdicts** — never one `valid:true` — and the validator passes reference `zic` output too (producer-profile guard).
- **May NOT conclude:** that structural validity equals **semantic** correctness (separate axis — CORE.1/`semantic-report`); that the validator is a security sandbox or does arbitrary round-trip; that byte-parity holds outside slim mode / pinned fixtures.
- **Remaining risks:** `RISK.TZIF.1` (structural ≠ semantic — guarded by keeping them separate).
- **Future owner:** T15.4 enrichment (dual-block + indicator-relation verdicts), tracked.

## 7. Rust reliability reviewer

- **Fears:** hidden panics · non-determinism · effect leakage (silent host reads) · schema drift.
- **Read:** `docs/effect-boundary-map.md` · `docs/panic-policy.md` · `docs/schema-compatibility-policy.md` · `tests/reliability.rs`/`tests/schemas.rs`.
- **Run:** `cargo test` (determinism: same source → byte-identical; locale/host-time isolation); `cargo clippy --all-targets -- -D warnings`.
- **May conclude:** the compile path is pure + deterministic + locale/TZ-independent; effects are classified per module and host reads are caller-explicit; schema ids are typed contracts with a drift guard.
- **May NOT conclude:** that schema **instance**-validation runs in the core (registry/drift only — no validator dep); that the fuzz harness has run *exhaustively* (only a bounded smoke ran — T23.cargo-fuzz.1/.2 — not a coverage-saturating campaign); that the CI grep/cross-target gates are wired here (intent recorded, not run).
- **Remaining risks:** schema instance-validation deferred (audit-suite); CI-wire items.
- **Future owner:** T23 `audits/` + CI pipeline.

## 8. Legal / provenance reviewer

- **Fears:** republished copyrighted text · sources cited without provenance · public-domain treated as no-provenance.
- **Read:** `docs/copyright-non-republication-policy.md` · `docs/zic-knowledge-index.md` + `claim-source-map.md` · `ACKNOWLEDGEMENTS.md`.
- **Run:** inspect the source/archive ledgers (typed `SourceLedgerEntry`; `canonical_url`=authority, `archive_url`=witness).
- **May conclude:** provenance is recorded per source (URL · authority tier · rights posture · supported claim); copyrighted sources are link-only-no-mirror; the project preserves *claim provenance*, not copyrighted bodies of text.
- **May NOT conclude:** that the index is archive-complete (T18.1 is a **first cut**; archival columns are `pending_capture` — no URL fabricated); that public-domain tzdb removes the need for authenticity/version-scoping records.
- **Remaining risks:** `RISK.SOURCE.1` (doctrine — T18 breadth + capture pass pending).
- **Future owner:** T18.2/T18.3 (breadth→40 + lawful Wayback/archive.today capture).

## 9. Packager / release engineer

- **Fears:** unstable CLI/JSON contracts · silent schema-meaning drift · an unrepeatable release cut.
- **Read:** `RELEASE.md` · `docs/schema-compatibility-policy.md` · `docs/cli-compatibility-policy.md` · `CHANGELOG.md`.
- **Run:** the gate (`fmt`/`clippy -D`/`test`/`doc` + CORE.1 sweep); `tests/schemas.rs` (registry/drift).
- **May conclude:** JSON is deterministic + schema-versioned (bump = meaning change, additive ≠ bump); the 0/1/2 exit taxonomy is stable; a release cut records exactly what is hashed.
- **May NOT conclude:** that text output is a contract (it is not — only JSON schemas are); that diagnostic *wording* is stable (only class/location/severity are); that pre-1.0 claims forward-port across tzdb releases (they are release-scoped).
- **Remaining risks:** stability is policy + a drift test, not yet a full instance-validation gate.
- **Future owner:** T23 (schema instance-validation in the audit suite).

## 10. Adversarial reviewer (the hostile reader)

- **Fears (theirs to exploit):** any place the project says more than it proved — the overclaim.
- **Read:** `docs/not-yet-ready.md` · the `negative_capabilities` in `support-report` (each with `enforced_by`) · `docs/misuse-resistance-ledger.md` (a dedicated `docs/reviewer-objections.md` hostile-checklist is **T19.1-planned**, not yet written).
- **Run:** try to make a report read as more than its evidence; try to alias `zic`→`zic-rs`; feed a structurally-valid-but-wrong TZif; absent the oracle and look for a silent "no change".
- **May conclude:** every claim is bounded by a typed non-claim with a named guard; the refusal surface is maintained as hard as the success surface; *claim overstatement is itself a defect class* (`SECURITY.md`).
- **May NOT conclude:** that a missing/weak non-claim is acceptable — **report it** (it is in scope as a defect); that any narrowing of the evidence docs is acceptable (the additive-only doctrine).
- **Remaining risks:** the standing one — **claim drift** (stale state, prose outrunning typed evidence). This persona exists to keep attacking it.
- **Future owner:** every future seal (the lens never retires); `dsfb-gray` self-audit in T23.

---

> **Doctrine:** *a persona packet that cannot state what the reader may **not** conclude is incomplete.*
> The value of this layer is not the reassurance — it is the refusal. If an audience could read zic-rs as
> proving something it does not, that gap is a T20 defect, not a documentation nicety.
