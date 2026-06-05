# `support-report` as the public conformance engine (T15) — schema inventory (T15.1)

> **T15.1 is reference/inventory-first — NO behaviour change.** It makes the **conformance-report
> contract** explicit before building the validator: what each public report field is, which finite
> vocabularies must become enums (CONTRACT.TYPING, born typed in T15.2+), the source of truth for each,
> the claim it protects, and its test/golden witness. It changes no code beyond an executable
> **shape-witness** (`tests/conformance_report_shape.rs`) that pins the *current* report shape so T15.2+
> additions are deliberate. (T15.1 added only the shape-witness — no behaviour change; T15.2–T15.5 then
> added the born-typed fields, each flipping the witness deliberately. T15.5-remainder added the typed
> claim-shape axes (`ReferencePinGate` · `ClaimPortability` · `EvidenceAuthorityKind` · `ClaimBoundary` ·
> `valid_disambiguation`), richer `OracleIdentity`, `ZIC026`, and golden + failure-mode fixtures. **Live
> gate: 394 tests · CORE.1 341/0/0**; support-report now **v4**, structural-report v3, + `semantic-report-v1`
> / `tzif-validation-v1`. Diagnostic contract `ZIC001`–`ZIC026`.)
>
> **The design principle (the T15 north star):** *if a human can read it as a claim, a machine should be
> able to locate the field that proves or bounds it.* The engine keeps four layers **distinct** and never
> collapses them: **manifest provenance** (what was built) · **diagnostic evidence** (what was noticed) ·
> **semantic behaviour** (what the output *does*, via `zdump`) · **TZif structural correctness** (what the
> bytes *are*, via RFC 9636). A report row belongs to exactly one layer.

## Current public report surfaces (pinned)

* **`support-report` — `zic-rs-support-report-v4`** (`src/report.rs::to_json`): `schema` · `provenance{…}`
  · **`oracle_mode{mode, skipped_with_reason}`** (T15.2) · `tzdb_version` · `zones_parsed` · `links_parsed`
  · `supported_identifiers` · `fully_accounted` · `supported_zones[]` ·
  `unsupported{label:{count, deep_semantic, example_message, zones[]}}` ·
  `links{to_supported, to_unsupported, cycles, missing}`.
* **`structural-report` — `zic-rs-structural-report-v3`** (`src/structural.rs::to_json`): `schema` ·
  **`oracle_mode`** (T15.2; `reference_zic`) ·
  `provenance{…}` · `tzdb_version` · `reference_zic` · `zones_compared` · `class_counts` ·
  `version_footer_match` · `ours/ref_{timecnt,charcnt,version}` · `timecnt_delta_total` ·
  `differences[{zone, class, dims}]` · `errors[]`.
* **`provenance{…}` block** (shared, `manifest::provenance_block_json`): `manifest_schema` ·
  `per_run_profile` · `source_variant_reference_pin_gate` · `blocked_substeps` ·
  `unpinned_required_files` · `source_variant_behavior_implemented` · `note`.
* **`compile --manifest` — `zic-rs-compile-manifest-v8`**: `build_profile` · `source_inputs` ·
  `link_profile` · `source_profile.{backward,backzone,packratlist,dataform}_evidence` · `oracle{mode,
  horizon, result}`.

All JSON is hand-rolled + deterministic (shared `crate::json::escape`, no serde).

## The contract table (field · finite-vocab owner · source of truth · claim · witness)

| Field | Finite vocab? | Current type | Born-typed owner (T15+) | Claim protected | Witness |
|-------|---------------|--------------|-------------------------|-----------------|---------|
| structural `class` | yes (10) | **`ParityClass` enum** ✓ | — (done) | structural parity axis | `tests/structural_report.rs` |
| `source_variant_behavior_implemented` | bool ✓ | `bool` | — | "no variant behaviour claimed" | report-shape tests |
| `source_variant_reference_pin_gate` | yes | `&str` const `lifted_for_2026b` | **`ReferencePinGate`** (T15.5) | release-admission gate state | shape-witness |
| `oracle.mode` / `.result` | yes (`not-run`/`zdump`/`structural`) | `String` | **`OracleMode`** (T15.2) | *oracle availability* — a test never silently weakens | T15.2 |
| evidence-category of each input/artifact | yes (8) | **prose only** | **`ArtifactCategory`** (T15.3) | the `zone.tab`-is-policy-not-compile catch, *as a type* | T15.3 |
| `negative_capabilities` | yes (list) | **prose only** | **`NegativeCapability`** (T15.2) | advertised restraint = enforced guard | T15.2 |
| semantic witness verdict | yes | **not present** | **`SemanticWitnessVerdict`** (T15.3) | behaviour parity beyond bytes (`zdump`) | T15.3 |
| TZif structural verdict | yes | **not present** | **`TzifStructuralVerdict`** (T15.4) | RFC 9636 bytes-are-well-formed | T15.4 |
| one-line machine status | yes | **not present** | **`ConformanceStatus`** (T15.5) | the machine-readable summary | T15.5 |
| `unsupported` bucket label | derived from `DiagnosticCode` | `String` | (stays derived — totality-tested upstream) | which construct is unsupported | report tests |
| `deep_semantic` | finite (laws) | `&'static str`/null | (function-backed; leave) | which deep-semantics law applies | report tests |

## Born-typed enums (the T15 CONTRACT.TYPING deliverables — planned, built in the named substep)

```rust
enum OracleMode { NotRun, ReferenceZic, ReferenceZdump, StructuralDecode, Unavailable(reason) }   // T15.2 ✅ shipped
enum NegativeCapability { /* one per advertised restraint, each → an enforced guard + test */ }   // T15.2 ✅ shipped
enum ArtifactCategory { CompileInput, PolicyInput, ReferenceInput, GeneratedArtifact, OutputArtifact,
    DiagnosticArtifact, SemanticWitnessArtifact, StructuralValidationArtifact, PolicyProse,
    ReleaseNoteEvidence }                                                                          // T15.3 ✅ shipped (full T12 spine + 2)
enum SemanticWitnessVerdict { Match, Mismatch, SkippedOracleUnavailable, NotApplicable,
    OutOfHorizon, KnownDivergence }                                                                // T15.3 ✅ shipped
// also shipped T15.3: OracleIdentity {zic, zdump, versions, reference_platform}; fixture_set; witness_horizon
enum TzifStructuralVerdict { Conformant, Violation }                                               // T15.4 ✅ shipped
// also shipped T15.4: PosixFooterVerdict · ReaderCompatibilityVerdict · LeapExpiryVerdict · TzifVersionVerdict
struct ConformanceStatus { ConformanceLevel · declared_scope_hash · ReportKind · CompilerIdentity ·
    WorkspaceProvenance · ReportProvenance }                                                       // T15.5 ✅ shipped (core)
enum ReferencePinGate { Open, LiftedFor2026b }                                                     // T15.5-rem ✅ shipped (drift-pinned to SOURCE_VARIANT_GATE_STATUS)
enum ClaimPortability { ReleaseSpecific, OracleSpecific, PlatformSpecific, ProfileSpecific,
    FixtureSpecific, GeneralProjectPolicy }                                                         // T15.5-rem ✅ shipped
enum EvidenceAuthorityKind { NormativeSpec, ImplementationObservation, ManpageDocumentation,
    PolicyGuidance, ReleaseNote, EmpiricalFixture, ProjectDoctrine }                                // T15.5-rem ✅ shipped
struct ClaimBoundary { proves · does_not_prove · depends_on }                                      // T15.5-rem ✅ shipped (on conformance_status)
const VALID_DISAMBIGUATION                                                                          // T15.5-rem ✅ shipped (the 7 distinct senses of "valid")
// also shipped T15.5-rem: richer OracleIdentity (binary sha256 · command-line · env TZ/LC_ALL ·
//   zoneinfo_resolution=explicit_tzif_path_argument) · ZIC026 "values over 24 hours" (the T14.4 residual)
//   · golden + failure-mode report fixtures (tests/conformance_golden.rs + fixtures/conformance/).
// T15.close ✅ SEALED (reports/t15-close-receipt.md): the "valid"-disambiguation + 3 more guard-backed
//   non-claims (→ 15) shipped. Assigned out (recorded-or-assigned rule): ReferenceLocatorKind ·
//   SignatureTrustModel · the richer reference-identity axes [ReferenceBuildProfile/TimeTModel/
//   RuntimeLeapSupport/…] → T16/T16.5; crash_durable_without_fsync → T17; CLDR_ICU_runtime_api → T19.
```

Each lands the way `DiagnosticCode`/`ParityClass` did: an exhaustive enum, a boundary renderer
(`label()`/`as_str()`), a **totality test**, and a **golden roundtrip** test for the public JSON. Strings
are rendered only at the JSON boundary; the report schema bumps **only** when a field is genuinely added.

## `negative_capabilities` — the seed set (T15.2 builds the enum; each entry → an enforced guard + test)

Advertised restraint is engineering, not decoration: each entry is **JSON-visible and test-pinned** —
mapped to a guard that already exists (or a test that proves the inference is refused). The seed set
(each becomes a `NegativeCapability` variant in T15.2):

```text
does_not_claim_unadmitted_vendor_parity          → T13 reference-platform matrix (only upstream_iana_2026b admitted)
does_not_infer_source_variant_from_output_shape   → T12.5 source_variants_not_inferred_* tests
does_not_infer_DATAFORM_from_content              → T12.5d (negative-SAVE ≠ vanguard) test
does_not_treat_manifest_as_TZif_semantics         → manifest is provenance, not a required TZif sidecar (T12 close §5)
does_not_claim_full_TOCTOU_resistance             → T14.6 RequiresOpenatStyleHardening row
does_not_claim_all_IANA_releases_without_admission → T12.5a.3 release-admission matrix (1 admitted)
does_not_require_manifest_to_read_TZif            → RFC 9636 reader needs only the emitted bytes
does_not_curate_time_or_define_display_names      → IANA/CLDR boundary (not zic-rs's job)
```

## The mantra (in force for T15 and every later claim-bearing surface)

```text
Every claim needs an owner type.
Every finite status needs an enum.
Every public report field needs a test.
Every non-claim needs a machine-visible row.
Every oracle absence needs skipped_with_reason, not silence.
```

Two distinctions T15 must **never** blur: **semantic witness** (offset / abbreviation /
is_dst at selected timestamps, via `zdump`) vs **structural validator** (the TZif bytes are well-formed
and internally consistent, via RFC 9636) — both matter; neither substitutes for the other. And reports
must be **machine-checkable**, never prose that reads well but cannot be tested.

## Substep roadmap (named; only T15.1 built now)

- **T15.1 — schema inventory ✅** (this doc + the shape-witness; no behaviour change).
- **T15.2 — `OracleMode` + `negative_capabilities` ✅ DONE** (CONTRACT.TYPING priorities 5 & 1; additive
  schema bump **`support-report`/`structural-report` v2 → v3**; CORE.1 341/0/0; **366 tests**). `OracleMode`
  enum (`src/manifest.rs`): `NotRun`/`ReferenceZic`/`ReferenceZdump`/`StructuralDecode`/`Unavailable(reason)`,
  rendered as the report field `"oracle_mode": { "mode": …, "skipped_with_reason": …|null }` — **absence is
  visible** (support-report → `not_run`; structural-report → `reference_zic`; `Unavailable` carries the
  reason, never silence). `NegativeCapability` enum + `NEGATIVE_CAPABILITIES` (sorted, unique): emitted in
  the shared provenance block as `{ "capability": …, "enforced_by": … }` — **each non-claim names the
  guard/test/receipt that enforces it** (never decorative). The T15.1 shape-witness **flipped** (both
  fields now asserted present; `conformance_status` still asserted absent → T15.5). +3 witness tests
  (`oracle_mode` shape · negative-caps sorted+guarded · `skipped_with_reason` visible).
- **T15.2a — `OracleResult.mode` unification ✅ DONE** (no schema change; CORE.1 341/0/0; **369 tests**).
  The CONTRACT.TYPING rule made strict: *once a finite vocabulary gets its owner enum, no current
  claim-bearing path may keep emitting it as a free string.* So `OracleResult.mode` (the manifest's oracle
  field — only ever `not_run` in practice, single construction site) changed `String` → **`OracleMode`**.
  The `zic-rs-compile-manifest-v8` `oracle.mode` value is preserved (`"not-run"`) via a documented
  **boundary shim** `OracleMode::manifest_str()` (the one legacy value; everything else == `mode_str()`),
  with a removal plan (canonicalize at the next manifest major bump) and a **drift test** pinning that the
  shim diverges for that one value only — so **all** oracle-mode rendering is single-sourced through the
  owner enum, manifest stays v8 (no churn), and no free-string oracle-mode path remains. +3 tests.
- **T15.3 — semantic witnesses + `ArtifactCategory` ✅ DONE** (new `semantic-report` command + schema
  `zic-rs-semantic-report-v1`; CORE.1 341/0/0; **376 tests**). A typed, **`zdump`-backed** behaviour
  surface — `src/semantic_witness.rs`: `SemanticWitnessVerdict` (`match`/`mismatch`/`skipped_oracle_unavailable`/
  `not_applicable`/`out_of_horizon`/`known_divergence`), `SemanticObservation {offset_seconds, is_dst,
  abbreviation}`, witness rows (zone · timestamp · reference · zic_rs · verdict · `artifact_category`).
  **Footer-aware** (the verdicts read reference `zic`'s bytes through `zdump -v -c LO,HI`, which evaluates
  the POSIX footer — sound across slim/fat, unlike raw decode). **Oracle absence visible**: no reference
  `zic`/`zdump` → `OracleMode::Unavailable(reason)` + every row `skipped_oracle_unavailable` (never silent).
  **`ArtifactCategory`** (`manifest`) shipped as the full **T12 spine + 2** (`compile_input`/`policy_input`/
  `reference_input`/`generated_artifact`/`output_artifact`/`diagnostic_artifact`/`semantic_witness_artifact`/
  `structural_validation_artifact`/`policy_prose`/`release_note_evidence`) — the typed guardrail the
  `zone.tab` error earned; **every claim-bearing row carries it**. Born with room for the
  *add-now* shapes: **`oracle_identity`** (tool names + captured versions + `reference_platform`; binary
  hash/command-line → T15.5), **`fixture_set`** (`semantic-witness-seed-v1`), **`witness_horizon`** +
  **`witness_scope: small_seed`** (so the seed set is never overread as "semantic parity" — a match is
  *matched for the declared witness set*, no more). **Explicit non-claim in the report:** a semantic
  witness is **NOT** RFC 9636 structural validity (that is T15.4). The T14 multi-era same-instant residual
  is representable as the typed `KnownDivergence` verdict (not prose). **Design note:** chose a *dedicated*
  `semantic-report-v1` (degrades gracefully when the oracle is absent) over bloating `support`/`structural`
  reports — cleaner layer separation, so those stay **v3** (no forced v3→v4). +7 tests.
- **T15.4 — RFC 9636 TZif structural validator ✅ DONE** (new `tzif-validate` / `zic-rs-tzif-validation-v1`;
  CORE.1 341/0/0; **385 tests**). `src/tzif/rfc9636.rs` — **five separate typed verdicts, never one
  `valid:true`**: `TzifStructuralVerdict` (conformant/violation) · `PosixFooterVerdict` (parseability;
  *projection*-match is semantic = T15.3) · `ReaderCompatibilityVerdict` (no-hazard / legacy-transition-
  count[`>1200`, ZIC020] / v4-reader / not-exercised / unknown — a *separate* axis from structural) ·
  `LeapExpiryVerdict` (no-table / no-expiration / expiration-present[v4 no-op marker]) · `TzifVersionVerdict`
  (v1–v4). Checks the counted-array bounds RFC flags + the invariants `parse` leaves latent (typecnt≥1 ·
  transition type-index < typecnt · indicator counts ∈{0,typecnt} · strictly-ascending transitions).
  **Bounds-safe** — reuses the bounds-checked `parse` (Err→Violation, never panics; hand-built type-index-
  OOB fixture caught as a verdict). **Validates reference `zic` output too** (the producer-profile guard —
  reference output must pass, else the validator is too strict); `reference_validated` recorded, never
  silent. **Non-claims (in the report):** NOT semantic behaviour (T15.3) · NOT a hardened security sandbox ·
  no arbitrary-TZif round-trip. +9 tests.
- **T15.5 — one-line conformance rollup + report-as-artifact provenance ✅ DONE (core)** (support-report
  **v3 → v4**; CORE.1 341/0/0; **387 tests**). `ConformanceStatus` (in `manifest.rs`) emits a `conformance_status`
  block in support-report: **`ConformanceLevel`** — a *bounded* level (no `compatible`/`conformant:true`);
  support-report establishes `release_admitted_compile_coverage`, nothing more · **`declared_scope_hash`**
  (SHA-256 over the claim envelope: admitted-release gate · manifest+report schema versions · sorted
  negative-capability ids · CORE.1 claim — a compact id a reviewer pins a claim to; deterministic, tested) ·
  **`ReportKind`** · **`CompilerIdentity`** (zic_rs_version + arch-os target + profile; rustc/git **honestly
  `null`** — no `build.rs`, disclosed not faked) · **`WorkspaceProvenance`** (`unknown` — not a git tree) ·
  **`ReportProvenance`** (`unsigned_local_report` — *a report is not an attestation*) · `available_surfaces`.
  *A public report is itself a claim surface, not an unexamined trust root.* +4 enforced **non-claims**
  (`arbitrary_tzif_roundtrip` · `tzif_validator_as_security_sandbox` [T15.4] · `future_civil_time_authority`
  · `report_authenticity_without_signature_or_reproducible_context`). +6 tests; shape-witness flipped
  (conformance_status now present).
- **T15.5-remainder ✅** — shipped the above tracked items: typed `ReferencePinGate`/`ClaimPortability`/
  `EvidenceAuthorityKind`/`ClaimBoundary` + `VALID_DISAMBIGUATION` on the rollup; richer `OracleIdentity`
  (binary sha256 · command-line · env · `zoneinfo_resolution`); **`ZIC026`** "values over 24 hours"
  (the T14.4 tail); golden + failure-mode report fixtures (`tests/conformance_golden.rs`).
- **T15.close ✅ — T15 CLOSED** (`reports/t15-close-receipt.md`): the table-driven public-surface seal
  (each surface's owner types / proves / does-not-prove), the public command set, the frozen 7-sense
  **"valid"-disambiguation**, and the diagnostic contract `ZIC001`–`ZIC026`. Added the last 3 guard-backed
  `NegativeCapability` variants (→ **15** total): `leap_smear_semantics` ·
  `range_truncation_leap_expiry_interaction_parity_without_witness` · `depend_on_host_endianness`. Every
  remaining item is **recorded-or-assigned, never vaguely mentioned**: `crash_durable_without_fsync`→T17
  (honest nuance — `atomic_write` *does* `sync_all()` the file but not the parent dir) ·
  `downstream_update_timeliness`→T16 · `CLDR_ICU_runtime_api`→T19 · `ReferenceLocatorKind`/
  `SignatureTrustModel`→T16. **Freeze: no global compatibility claim · no `conformant:true` · no
  report-as-attestation.**

## Acceptance (T15 — overall)

> T15 is accepted when zic-rs's support/structural reports are a public conformance engine: they expose
> oracle availability, admitted evidence categories, negative capabilities, semantic-witness verdicts,
> TZif structural validation, and a one-line machine-readable status — each backed by a born-typed enum
> with a totality + golden test — while preserving the distinction between manifest provenance, diagnostic
> evidence, semantic behaviour, and TZif structural correctness. *(T15.1 met: the contract is inventoried,
> the born-typed owners named, and the current shape pinned by an executable witness; no behaviour change.)*

## Non-claims (T15.1)

* No new report field or behaviour yet — T15.1 is the contract, not the engine.
* The born-typed enums are *named + planned*, not built (each is its own substep, born typed there).
* No report-schema bump in T15.1 (the shape is pinned, not changed).
