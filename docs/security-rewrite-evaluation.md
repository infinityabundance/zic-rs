# Security rewrite-evaluation packet (T20)

> For the reviewer who is **not** impressed by *"it's written in Rust."* They want **risk reduction
> provable from artifacts** (SLSA: source review ≠ artifact integrity; OpenSSF Scorecard: repeatable
> security *habits*). This packet does not restate the risk register — it **frames the rewrite-safety
> argument** and points each row at its proof. "Rust" is **one row of the threat table, not the argument.**
> Consumes: `docs/risk-register.md` (the 15 claim-boundary risks), `SECURITY.md` (what counts as a defect),
> `docs/panic-policy.md`, `reports/t17-count-arithmetic-verdict.md`, `docs/install-materialization-contract.md`,
> `docs/effect-boundary-map.md`. The per-audience reading is `docs/security-personas.md`.

## (A) Rewrite threat-model table — threat · mitigation · evidence · risk-row

| Threat | Mitigation | Evidence | Risk row |
|---|---|---|---|
| buffer/memory overflow | safe Rust · `#![forbid(unsafe_code)]` | the crate compiles under forbid; grep gate (CI-wire) | — (structural) |
| integer / count overflow | `overflow-checks` (all profiles) + **checked** count×size before any alloc | `reports/t17-count-arithmetic-verdict.md`; `tzif::validate` tests | `RISK.COUNT.1` (guarded) |
| OOB type-index panic on hostile TZif | bounds-guard at the single decode choke point | `src/tzif/validate.rs` tests (OOB index → typed `Err`) | `RISK.COUNT.1` |
| panic / abort on malformed input | *no panic on untrusted input → typed `Err`* | `docs/panic-policy.md`; parser/`pathology_ledger`/`reliability` tests | (policy) |
| path traversal / clobber | `ZIC008` hard-reject · no-clobber-without-`--force` | `docs/install-materialization-contract.md`; `tests/hostile_output_tree.rs` | `RISK.PATH.1` (partial — leaf closed) |
| TOCTOU write-through (leaf) | exclusive-create · atomic temp-symlink+rename | `fs/atomic_write.rs`, `fs/output_tree.rs` tests | `RISK.PATH.1` |
| partial / non-durable install | compile-all-to-memory → publish · per-file fsync+dir-fsync (Unix) | `install-materialization-contract.md` | `RISK.INSTALL.1` (partial) |
| resource exhaustion (giant input) | `limits::ResourceLimits` caps → `Error::config` | `src/limits.rs` tests | `RISK.RESOURCE.1` (guarded) |
| wrong TZif with a valid-looking report | `zic`/`zdump` oracle (CORE.1) + 5 separate TZif verdicts | CORE.1 sweep · `tzif-validate` · `semantic-report` | `RISK.TZIF.1` (guarded) |
| `unknown` rendered as `unchanged` | typed `behaviour_unassessed` / `oracle_mode` visible | `tests/release_diff.rs` | `RISK.DIFF.1` (guarded) |
| oracle-identity confusion | oracle binary sha256 + explicit TZif-path argument | `semantic-report` `oracle_identity` | `RISK.ORACLE.1` (guarded) |
| report-as-attestation | `ReportProvenance = unsigned_local_report` | every report's provenance block | `RISK.REPORT.1` (guarded) |
| supply-chain tampering | lean deps · no `build.rs` · `Cargo.lock` · (SBOM/signing → §D, planned) | `Cargo.toml`; §D below | (see §D/§E) |
| dependency compromise | minimal dependency set; cargo-audit/deny/vet (→ T23 `audits/`) | `audits/` suite (receipt-bearing when run) | `RISK.ADOPT.1` context |

## (B) Panic discipline

Already shipped — see `docs/panic-policy.md` (untrusted-reachable → must `Result`; internal-invariant →
may assert with a named reason; audited allowed-assertion sites; CI grep-gate intent recorded). Not
restated here.

## (C) Resource-exhaustion model

Every input-driven dimension reference `zic` leaves unbounded is capped by `limits::ResourceLimits`
(bucket-3 *intentional safer divergence*; a breach is an operational `Error::config`, **not** a `ZIC###`
grammar diagnostic — the code space stays reserved for source grammar):

| Dimension | Disposition |
|---|---|
| source bytes / file (512 MiB) · zone / rule / link counts (1M) · leap entries (100k) · link-chain depth (256) · zone-eras (100k) | **hard-reject** (`Error::config`) |
| line length (2048, `_POSIX2_LINE_MAX`) | hard-reject (`ZIC017`) |
| transitions / zone (`MAX_TRANSITIONS`) · abbrev-table bytes (255, `u8` index) | hard-reject (`ZIC009` / structural) |
| count × element-size before `with_capacity` | checked → `Err` before alloc (`RISK.COUNT.1`) |

Defaults sit far above any real tzdb (2026b: ~350 zones, ~600 links, 27 leaps, chains 1–3 deep), so **no
legitimate input is ever rejected** — the caps bound only the pathological tail. **Non-claim:** not total
DoS resistance; the caps bound the *input-size* tail, not all adversarial CPU/time (`RISK.RESOURCE.1`).

## (D) Supply-chain trust packet (what exists vs what is planned)

| Element | Status |
|---|---|
| `#![forbid(unsafe_code)]`, no `build.rs`, no network at build/test | ✅ enforced |
| minimal dependency set (`clap`, `thiserror`, `tempfile`; `tz-rs` dev/optional) + `Cargo.lock` | ✅ present |
| in-house SHA-256 (`src/hash.rs`) for every emitted artifact + source hash in manifest | ✅ present |
| reference `zic`/`zdump` identity (binary sha256) recorded in reports | ✅ present (`oracle_identity`) |
| tzdb release + build profile recorded | ✅ present (manifest `tzdb` block; admitted 2026b) |
| **CycloneDX SBOM · SLSA provenance · signed tags · `cargo package --list`** | ▷ **planned** (CI-wire / not-a-git-repo here — stated honestly, not faked) |

**Non-claim:** zic-rs does not today ship an SBOM, SLSA provenance, or signed releases — those are
named, owned (T23 `audits/` + a release pipeline), and **not asserted as present**.

## (E) OpenSSF Scorecard-readiness checklist (honest)

| Practice | State |
|---|---|
| `SECURITY.md` · LICENSE · `CONTRIBUTING.md` | ✅ present |
| panic/`unsafe` discipline + docs | ✅ present (`panic-policy.md`) |
| fuzz target | ✅ bounded smoke RAN (T23.cargo-fuzz.1/.2): found 3 panic bugs F1–F3, fixed 3/3 + regression seeds, re-ran 9/9 clean; **bounded ≠ exhaustive** (no coverage-saturating campaign yet) |
| CI · SAST (CodeQL) · pinned-by-SHA actions · read-only tokens · signed releases · branch protection | ▷ planned (CI-wire; **no badge claimed until actually met**) |

A Scorecard badge is added **only after** the repo genuinely meets the criteria — claiming readiness is
not claiming compliance.

## (F) Rewrite-safety delta — the honest framing

**Removed from the C `zic` risk surface** (provably, by construction): manual TZif buffer allocation ·
unchecked string/path handling · unchecked count arithmetic before allocation · unsafe filesystem writes.

**NOT auto-solved by Rust** (and never claimed to be): wrong *semantics* (mitigated by the oracle, not
the language) · wrong *output* with a clean report (mitigated by CORE.1 + the 5 TZif verdicts) ·
supply-chain / dependency compromise (§D/§E) · DoS via resource exhaustion (§C bounds the input tail, not
all adversarial cost) · install *policy* mistakes (the materialization contract, not the borrow checker).

> **The one-sentence security claim:** *Rust removes an entire class of memory/arithmetic/buffer defects
> from the `zic` surface by construction; everything else zic-rs is trusted for is earned by an oracle
> check, a typed contract, a bounded cap, or a named non-claim — never by the rewrite alone.*
