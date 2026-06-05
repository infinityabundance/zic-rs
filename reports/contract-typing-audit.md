# CONTRACT.TYPING audit (T17.2)

> **Doctrine.** *Prose is the weakest guarantee; an exhaustive `match` that won't compile if a variant is
> unclassified is the strongest.* Every **claim-bearing** public surface with a *finite vocabulary* — a
> value that could let the project overclaim, silently drift, or misclassify evidence — must be a typed
> enum/newtype that owns its JSON/text literal via an `as_str()`/`label()` rendered **only at the
> boundary**, with a totality test (and a golden roundtrip where it appears in public JSON). The lens is
> **not** "can this be an enum?" — it is *"could a wrong value here overclaim, drift, or misclassify?"*
> Internal glue that cannot affect a claim stays simple (over-typing trivia is its own failure).
>
> This is the standing audit table the T17.CONTRACT.TYPING campaign promised. It is **append-only**: each
> row is statused `done` / `owned-by-Tn` / `intentionally-untyped (reason)`. Rows are not deleted when
> done — the history of *why each string was a claim-bearing seam* is itself the evidence (see the
> **doc-evidentiary-density**
> doctrine: nothing compressed away).

## Status legend

- **✅ done** — typed enum/newtype, renders its literal at the boundary, totality-tested.
- **◷ owned-by-Tn** — a claim-bearing surface deferred to its owning milestone (born typed there).
- **⊘ intentionally-untyped** — deliberately left as-is, with the reason; re-typing would be ceremony, not
  safety (the "where NOT to overdo it" rows — over-typing is a real failure mode).

## The six T17.2 surfaces (the campaign's core — all ✅ done)

Each was a free `String` / `&'static str` whose value came from a **closed** set but was emitted as prose,
so a future code path or careless edit could leak an unintended value into the public
`zic-rs-compile-manifest-v8` / `zic-rs-alias-map-v1` JSON. The emitted strings are **byte-identical** to
the pre-T17.2 output (the manifest tests + the `support-report` conformance golden pin them), so **no
schema bumps**. This was claim hygiene, *not* a style refactor — each is a place a reviewer could have
mistaken prose for a finite contract.

| # | Surface (file) | Was | Now (T17.2) | Boundary renderer | Totality test |
|---|---|---|---|---|---|
| 1 | `OracleResult.result` (`manifest.rs`) | `String` (`"not-run"`) | **`OracleVerdict`** enum | `OracleVerdict::as_str` (hyphen `"not-run"` preserved) | `output_tree_leap_mode_oracle_verdict_literals` |
| 2 | `BuildProfile.output_tree` (`manifest.rs`) | `&'static str` (`"posix"`/`"right"`) | **`OutputTree`** enum | `OutputTree::as_str` | `output_tree_leap_mode_oracle_verdict_literals` |
| 3 | `LeapSourceInfo.mode` (`manifest.rs`) | `&'static str` (`"none"`/`"file"`) | **`LeapSourceMode`** enum | `LeapSourceMode::as_str` | `output_tree_leap_mode_oracle_verdict_literals` |
| 4 | `SourceInputs.kind` (`manifest.rs`) | `String` (4 values) | **`SourceInputKind`** enum (+ `ALL`) | `SourceInputKind::as_str` | `source_input_kind_totality_and_literals` |
| 5 | alias-map entry `"kind"` (`manifest.rs`) | hand-emitted `"zone"`/`"link"` literal in the JSON format string | **`AliasEntry::kind_str()`** (the enum already existed; the *literal* is now owned) | `AliasEntry::kind_str` | `alias_entry_kind_str` |
| 6 | `BuildProfile.{emit_style, link_mode}` (`manifest.rs`) | `EmitStyle`/`LinkMode` **re-stringified to `String`** at manifest-build (enum ownership lost across the boundary) | store the typed `crate::EmitStyle` / `crate::LinkMode` directly | `emit_style_str` (module-private) / `LinkMode::as_str` | `emit_style_boundary_literals_unchanged` |

**Why #6 mattered most:** the enums *existed* but the manifest downgraded
them to `String` at construction, so the JSON literal was a hand-maintained copy that could silently
diverge from the actual `config` value. Storing the enum and rendering at the boundary closes that seam.

## Already-typed before T17.2 (recorded for completeness — ✅ done in earlier milestones)

- `OracleResult.mode` / `OracleResult.result.mode` → **`OracleMode`** (T15.2a; boundary shim
  `manifest_str()` preserves the legacy `"not-run"` value, drift-tested).
- alias-map `materialised` → **`LinkMode`** rendered via `LinkMode::as_str()` (T15.5-remainder).
- `ParityClass` (structural report), `ReportKind`, `ConformanceLevel`, the eight source-variant
  `detected`/`claimed` evidence enums, `ArtifactCategory`, `ReferenceLocatorKind`/`SignatureTrustModel`,
  `ZoneTableKind`/`ZoneTableStructuralVerdict`, the `vendor-oracle-receipt-v1` enums, the six CLI arg
  enums, `ReleaseChangeKind` (T16.6a), `ToolStatus`/`TzdataStatus` (T16.6b) — all already typed +
  totality-/shape-tested where public.

## Intentionally-untyped (⊘ — the "do not over-type" rows, with reasons)

- **`*Evidence::status()` derived methods** (`version_status`, the four `*Evidence::status()`): already
  `match` exhaustively over typed enums and are totality-tested; re-typing the *returned string* would be
  ceremony over the same state. The inputs are typed; the output is a derived render.
- **`Verdict` (`tests/diagnostic_parity.rs`) and `Admit` (`tests/input_admissibility.rs`)**: test-only
  classifications of *our comparison verdicts*, not a public contract — they stay in tests by design.
- **Free-form human message strings** in diagnostics/errors (`Error::message`, diagnostic `message`):
  these are *wording*, explicitly the non-contract axis (the diagnostic contract is
  class/layer/severity/span, compared *last* on wording). Typing wording would freeze what is meant to
  evolve.

## Owned-by-later-milestone (◷ — born typed there, not retrofitted)

- `receipt_production_mode` (vendor-oracle lab) → a future receipt-schema / matrix-renderer axis (the
  vocabulary is recorded in prose in `../zic-rs-vendor-oracle-lab/RECEIPT-MATRIX.md` meanwhile) →
  **T16.6.x**.
- `OracleFailureScope`, `ToolVersionStatus`, typed hash-read status (doctor/release-diff) → **T17.3**.
- The `EvidenceDisposition` / `InadmissibilityReason` umbrella + `InputAdmissibility` /
  `ReferencePlatformStatus` typed dispositions → tracked in the plan's `T17.CONTRACT.TYPING` block (some
  already realised as the `DiagnosticLayer::Lexical` mapping; the umbrella enum remains a tracked item).

## Acceptance (met)

All six remaining claim-bearing free-string surfaces are finite enums with `as_str()` renderers, totality
tests where appropriate, JSON/text rendering **unchanged except for the typed source of truth**, and **no
new open-ended claim vocabulary can enter the manifest / alias-map through these fields**. Gate:
`fmt` ✅ · `clippy --all-targets -D warnings` ✅ · **465 tests** ✅ · `doc` ✅ · CORE.1 341/0/0 ✅. No
report/manifest schema bumped (literals byte-identical; golden unchanged).
