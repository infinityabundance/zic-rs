# T15 — `support-report` as the public conformance engine: closure receipt

> **T15 CLOSED for its declared scope.** T15 turned zic-rs from "well-tested internally" into
> **externally reviewable**: an outsider can run a handful of commands and read typed, machine-checkable
> reports that state — and *bound* — every compatibility claim. The recurring shape held throughout:
> **doctrine → owner type → report field → executable witness → receipt.** Each public field is born
> typed (an exhaustive enum / newtype rendered only at the JSON boundary, with a totality test); oracle
> absence is always visible; every non-claim names the guard that enforces it; and the report itself is
> treated as a claim-bearing artifact, not an unexamined trust root.
>
> **Gate at close: 394 tests · `fmt`/`clippy -D warnings`/`doc` clean · CORE.1 341/0/0** (no valid
> `tzdata.zi` behaviour changed anywhere in T15). Diagnostic contract grew **ZIC001–ZIC025 → ZIC001–ZIC026**
> (`ZIC026` via the machine-checked totality path). **No global compatibility claim, no
> `conformant: true`, no report-as-attestation claim** was introduced — by construction.

## The arc

| Substep | What | Headline | Surface / schema |
|---------|------|----------|------------------|
| **T15.1** | conformance-report **schema inventory** (inventory-first) | pinned the public report contract + the born-typed enum plan; shape-witness asserts not-yet-present fields are genuinely absent (flips deliberately) | `tests/conformance_report_shape.rs` |
| **T15.2** | typed **`OracleMode`** + **`negative_capabilities`** | oracle absence visible (`not_run`/`reference_zic`/`unavailable`+reason); each non-claim → an enforced guard | reports **v2→v3** |
| **T15.2a** | **`OracleResult.mode`** unification | no free-string oracle path remains; v8 manifest value preserved via a drift-tested boundary shim | manifest stays v8 |
| **T15.3** | **`zdump`-backed semantic witnesses** + **`ArtifactCategory`** | behaviour parity beyond bytes, oracle absence visible, every claim-bearing row categorised | `semantic-report` / `zic-rs-semantic-report-v1` |
| **T15.4** | **RFC 9636 TZif structural validator** | five *separate* typed verdicts (never one `valid:true`); bounds-safe; validates reference `zic` output too | `tzif-validate` / `zic-rs-tzif-validation-v1` |
| **T15.5 (core)** | **`ConformanceStatus`** rollup + report-as-artifact provenance | bounded `ConformanceLevel` (never `compatible`) · `declared_scope_hash` · `CompilerIdentity`/`WorkspaceProvenance`/`ReportProvenance` (honest nulls) | support-report **v3→v4** |
| **T15.5-remainder** | typed **claim-shape** axes + richer `OracleIdentity` + `ZIC026` + golden/failure-mode fixtures | `ReferencePinGate`·`ClaimPortability`·`EvidenceAuthorityKind`·`ClaimBoundary`·`VALID_DISAMBIGUATION`; oracle binary-sha256/cmdline/env/zoneinfo-resolution; the T14.4 "values over 24h" tail | `tests/conformance_golden.rs` · `fixtures/conformance/` |

## The public surfaces (what each proves — and does NOT)

| Surface | Report / artifact | Owner types | Claim proved | Does **not** prove | Status |
|---------|-------------------|-------------|--------------|--------------------|--------|
| `support-report` | `zic-rs-support-report-v4` | `OracleMode`·`NegativeCapability`·`ConformanceStatus`·`ReferencePinGate`·`ClaimPortability`·`EvidenceAuthorityKind`·`ClaimBoundary` | the admitted release's zones **compile** (compile-coverage, every zone in exactly one bucket) | behaviour / structural / reader parity (separate surfaces) | ✅ sealed |
| `semantic-report` | `zic-rs-semantic-report-v1` | `SemanticWitnessVerdict`·`OracleIdentity`·`ArtifactCategory` | selected `offset/is_dst/abbr` **match `zdump`** for the declared witness set | universal behaviour parity (scope = `small_seed`); RFC 9636 structural validity | ✅ sealed |
| `tzif-validate` | `zic-rs-tzif-validation-v1` | `TzifStructuralVerdict`·`PosixFooterVerdict`·`ReaderCompatibilityVerdict`·`LeapExpiryVerdict`·`TzifVersionVerdict` | RFC 9636 **byte-format integrity** (counts·bounds·index·indicator·ascending·version·footer-shape); reference output too | civil-time behaviour; a security sandbox; arbitrary round-trip; full dual-block/projection equivalence (first-pass v1) | ✅ sealed |
| `compile --manifest` | `zic-rs-compile-manifest-v8` | `SourceInputs`·`LinkProfile`·`SourceProfile`·`OracleResult`·`LinkMode` | build/source/profile **provenance** of a run | TZif semantics (a manifest is a sidecar, not required to read the bytes) | ✅ (T12) |
| diagnostic contract | `ZIC001`–`ZIC026` | `DiagnosticCode`·`DiagnosticLayer`·`DiagnosticVerbosity`·`DiagnosticSpanPrecision` | what the tool **noticed** + classified (layer/severity/verbosity/span) | output semantics from diagnostics alone | ✅ (T13/T14 + `ZIC026`) |
| `declared_scope_hash` | (in `conformance_status`) | SHA-256 over the claim envelope | a **compact pin** for "which claim envelope?" (gate·schemas·sorted non-claims·CORE.1) | anything by itself — it is an *identifier*, not a verdict | ✅ sealed |
| `VALID_DISAMBIGUATION` | (in `conformance_status`) | `const &[&str]` (7 senses) | that "valid" is **seven distinct claims**, impossible to blur | — (it exists to *prevent* a global reading) | ✅ sealed |

### Public command set

```sh
zic-rs support-report    --input <src> --format json
zic-rs semantic-report   --input <src> --format json
zic-rs structural-report --input <src> --reference-zic zic --format json
zic-rs tzif-validate     --input <src> --format json
zic-rs compile --manifest --input <src> --out <dir> --all-supported
```

## The seven senses of "valid" (the disambiguation, frozen)

`structurally_valid` ≠ `semantically_witness_matching` ≠ `modern_reader_compatible` ≠
`future_projection_matching` ≠ `release_admitted` ≠ `compile_covered` ≠ **`behaviour_matched`**. The live
behaviour claim (CORE.1) is the **last** and is deliberately separate from all the others — no surface
collapses them, and the schema makes the collapse impossible.

## Negative capabilities (15 — each JSON-visible and guard-enforced)

Recorded as `NegativeCapability` enum variants (sorted, totality-tested, each `enforced_by` a real
guard/test/receipt — never decorative). T15.5-remainder added the last three guard-backed entries
(`leap_smear_semantics` · `range_truncation_leap_expiry_interaction_parity_without_witness` ·
`depend_on_host_endianness`), joining the T15.2/T15.5-core set
(`all_iana_releases_without_admission` · `arbitrary_tzif_roundtrip` · `full_toctou_resistance` ·
`future_civil_time_authority` · `report_authenticity_without_signature_or_reproducible_context` ·
`tzif_validator_as_security_sandbox` · `unadmitted_vendor_parity` · `curate_time_or_define_display_names` ·
`infer_dataform_from_content` · `infer_source_variant_from_output_shape` · `require_manifest_to_read_tzif` ·
`treat_manifest_as_tzif_semantics`).

## Recorded-or-assigned (never vaguely mentioned)

Per the close discipline, every remaining candidate non-claim / typed surface is **either** an enum
variant **or** assigned to an owning milestone with a pointer:

| Item | Disposition |
|------|-------------|
| `does_not_claim_crash_durable_install_without_fsync_contract` | **→ T17.** Honest nuance: `fs/atomic_write.rs` *does* `sync_all()` the temp file (file contents durable), but does **not** fsync the parent directory after rename, so rename *visibility* is not crash-guaranteed. A blanket "no fsync" non-claim would misread the actual behaviour → the full crash-durability/directory-fsync contract is a T17 reliability item, not a T15 enum. |
| `does_not_claim_downstream_update_timeliness` | **→ T16** (release ecology) — about consumer update cadence, not a zic-rs guard. |
| `does_not_claim_CLDR_or_ICU_or_runtime_api_parity` | **→ T19** (language-runtime-contract persona) — broader than, and partly covered by, `does_not_curate_time_or_define_display_names`. |
| `ReferenceLocatorKind` (versioned-archive / live-dir / cached) | **→ T16/T16.5** — part of the richer reference-identity enrichment (`ReferenceBuildProfile`/`TimeTModel`/…), not built in a close receipt. |
| `SignatureTrustModel` (fingerprint-anchored / WoT / keyring / hash-only) | **→ T16** — the T12.5a.2 admission already used fingerprint-anchored OpenPGP; typing the model lands with the release-intake/signing-provenance block. |

## Intentional divergences / non-claims of the engine

* **No global verdict.** `ConformanceLevel` is bounded (`release_admitted_compile_coverage`, …); there is
  deliberately no `compatible` / `conformant: true`. A wrong value cannot overclaim — the test asserts
  the *absence* of an unbounded verdict.
* **A report is not an attestation.** `report_provenance` defaults to `unsigned_local_report`;
  `workspace_provenance` is honestly `unknown` (no git tree / no `build.rs`); `rustc`/`git_commit` are
  `null`, disclosed not faked.
* **Layers never blur.** semantic witness ≠ structural validity ≠ diagnostic ≠ manifest provenance —
  each has its own surface; a witness does not prove bytes, a validator does not prove behaviour.
* **The structural validator is a first-pass v1** (counts/bounds/index/indicator/ascending/version/
  footer-shape); full dual-block 32/64 equivalence · footer future-*projection* match · exhaustive v4
  leap-expiry subcases are tracked (T15.4-enrichment / T16), not assumed.

## Freeze statement

> **T15 — the public conformance engine — is complete for the declared scope.** The reports are typed,
> witnessed, report-backed, and bounded. No global compatibility claim, no `conformant: true`, no
> report-as-attestation claim exists. The next battlefield is **T16 (release ecology & downstream
> contract parity)** — `ReferenceBuildProfile`, the QEMU vendor diagnostic oracle lab, the auxiliary-table
> validator, install ecology, release-intake / patch-stack / signing provenance, and source/diagnostic
> provenance through generated transforms. T15 made the reports trustworthy; T16 makes the wider
> release/platform ecology *admissible*.
