# Schema compatibility policy (T17.6)

> **Doctrine.** *Public report fields are **compatibility surfaces**. A schema version names **meaning**,
> not formatting preference. If meaning changes, **bump**; if evidence is absent, render a **typed
> unknown**; if a field is claim-bearing, its literal must be **owned by an enum** (or an explicit,
> documented compatibility shim).* This locks the rules **before** T17.FUZZ / the T23 audit suite start
> emitting more artifacts, so those receipts emit into a stable contract. It is policy only — **no schema
> or behaviour changes here**; it classifies what already exists.

zic-rs emits **machine-readable JSON** from several commands, plus two interchange artifacts. Each carries
a `schema` id of the form `zic-rs-<kind>-vN` (the vendor receipt is the one deliberate exception — an
external-lab↔core interchange contract in its own namespace). Once a consumer reads these, the field set
and field *meanings* are commitments.

## The rules

1. **Bump the major `vN` when *meaning* changes** — a field is removed, renamed, re-typed, or its
   semantics change; an enum variant's meaning shifts; the shape of a sub-object changes. A bump is
   **intentional and recorded** (a changelog line in the owning module + a note in the seal). Example
   precedent: **`zic-rs-doctor-v1 → v2`** (T17.3) — `Present` replaced `version: string|null` + `sha256`
   with the typed `version_status`/`hash_status` objects; recorded in `src/doctor.rs`'s `SCHEMA` comment.
2. **Additive fields do *not* require a bump** *iff* they are optional and have a documented default
   meaning when absent (old readers ignore them). Example: `release-diff`'s `behaviour_error` (T17.3) —
   emitted only when present, no bump. **Adding a field that changes how existing fields are interpreted
   is a meaning change → bump.**
3. **Public-literal ownership (CONTRACT.TYPING).** Every finite-vocabulary claim-bearing JSON value is
   **owned by a Rust enum** rendered to its literal at the boundary (`as_str()`/`label()`), never a
   hand-emitted string. The T17.2 audit (`reports/contract-typing-audit.md`) enumerates these; the rule is
   standing for all future fields. A new literal entering a report as prose is a policy violation.
4. **Append-only enum policy.** Enum variants behind public literals are **append-only**; a variant's
   literal is never reused for a different meaning, never silently removed. Removing/repurposing one is a
   meaning change → bump. (Pre-1.0, a variant explicitly marked *experimental* in its doc may change, but
   it is labelled as such.)
5. **Typed-unknown over silence.** Absent evidence is rendered as an explicit typed value
   (`oracle_mode: unavailable` + reason · `BuildAxisEvidence::UnknownUnmeasured` · `behaviour_unassessed`
   · `HashReadStatus::Unreadable` · `not_probed`), **never** omitted or defaulted to a confident value.
   *Unknown is a value, not a gap.*
6. **Golden regeneration policy.** The pinned golden(s) (`fixtures/conformance/support-report.golden.json`)
   own the exact public literals + `declared_scope_hash`. A golden may change **only** when a bump or an
   intentional additive change is sealed; the diff is **reviewed and bounded to the intended change**
   (e.g. the T16.5-core regen was limited to the one new `NegativeCapability` + the expected scope-hash
   update). Host-variant fields (`target`/`profile`/rustc/git) are normalised, never pinned.
7. **History/appendix is append-only.** Changelogs, the manifest schema-changelog, close receipts, the
   plan's milestone history, the vendor-lab `IMAGE-PROVENANCE.md`, and SESSION-CONTEXT's ledger are
   **never rewritten** — superseded entries are added, not edited away. (Current-state *headers/banners*
   are refreshed; the dated/milestone-pinned record is not.)
8. **Old-reader / fail-closed expectation.** A consumer that does not recognise the `schema` id, or finds
   an **unknown critical field**, should **fail closed** (refuse to interpret), not guess. zic-rs states
   this expectation; it cannot enforce it in third-party readers, but its own readers (e.g. the
   `vendor-oracle-admit` ingester) follow it: `from_json` rejects an unknown top-level field
   (`UnknownField`) and a wrong `schema` (`WrongSchema`).

## The surfaces (classified)

| Surface | `schema` id | Emitted by | Stable? | Bump rule | Golden |
|---|---|---|---|---|---|
| Compile manifest | `zic-rs-compile-manifest-v8` | `compile --manifest` | **stable** (8 intentional bumps recorded in `manifest.rs` changelog) | meaning change → bump; additive evidence axes have lifted it before | `tests/manifest.rs` shape tests |
| Alias map | `zic-rs-alias-map-v1` | `compile --alias-map` | **stable** | meaning change → bump | `tests/manifest.rs` |
| Support report | `zic-rs-support-report-v4` | `support-report` | **stable** (v3→v4 = `conformance_status` rollup) | meaning change → bump | **`fixtures/conformance/support-report.golden.json`** (pinned) |
| Structural report | `zic-rs-structural-report-v3` | `structural-report` | **stable** (v2→v3 = provenance block) | meaning change → bump | shape test |
| Semantic report | `zic-rs-semantic-report-v1` | `semantic-report` | **stable** | meaning change → bump | shape test |
| TZif validation | `zic-rs-tzif-validation-v1` | `tzif-validate` | **stable** (T15.4 "first-pass v1" scope labelled) | meaning change → bump | `tests/` |
| Aux-table validation | `zic-rs-aux-table-validation-v1` | `aux-table-validate` | **stable** | meaning change → bump | `tests/aux_table_validation.rs` |
| Release diff | `zic-rs-release-diff-v1` | `release-diff` | **stable** (T16.6; `behaviour_error` added additively, no bump) | meaning change → bump | `tests/release_diff.rs` shape test |
| Doctor | `zic-rs-doctor-v2` | `doctor` | **stable since T17.3** (v1 shipped + bumped same cycle, no external consumers) | meaning change → bump (the v1→v2 worked example) | `tests/doctor.rs` shape test |
| Size report | `zic-rs-size-report-v1` | `size-report` | **stable** (T21.2; bundle footprint + deterministic `bundle_hash`) | meaning change → bump; additive optional fields don't | `tests/size_report.rs` |
| Vendor-oracle receipt | `vendor-oracle-receipt-v1` (interchange) | external lab → `vendor-oracle-admit` | **stable** (a lab↔core *interchange* contract, not an emitted report) | meaning change → bump; **`admit()` recomputes admission — the producer's self-assessment is never trusted** | `tests/vendor_oracle_receipt.rs` + round-trip |

> **Sealed-this-cycle note:** `doctor-v2` and `release-diff-v1` were introduced this development cycle
> (T16.6/T17.3) with no external consumers yet; the v1→v2 doctor bump was therefore an honest in-cycle
> shape correction, not a break of a frozen contract. Once an external consumer exists, every surface is
> governed by rule 1 (bump on meaning change) with no in-cycle exceptions.

## Non-claims

- This policy governs **zic-rs's own emitted/ingested schemas**; it does not constrain RFC 9636 TZif
  (the *output* format, governed by the standard) — that is a separate, external contract.
- A `schema` version is **not** an attestation of correctness; it names the *shape + meaning* of the
  evidence. Correctness lives in the evidence itself (CORE.1, the verdicts, the receipts).
- Pre-1.0, *experimental*-labelled surfaces may change without a major bump; they are explicitly marked.
  Nothing is currently labelled experimental — all ten surfaces above are stable.

See also: [`cli-compatibility-policy.md`](cli-compatibility-policy.md) (exit codes + command status),
`reports/contract-typing-audit.md` (literal ownership), `risk-register.md` (`RISK.REPORT.1`).
