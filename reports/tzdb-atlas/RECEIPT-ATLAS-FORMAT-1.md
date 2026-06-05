# RECEIPT — ATLAS-FORMAT.1 (machine-readable atlas exports) — 2026-06-05

> **Claim wording (binding):** *ATLAS-FORMAT.1 does not create new evidence. It makes the existing TZDB
> Evidence Atlas easier for external reviewers and tools to parse, validate, and inspect.*

## Method

A new derived-emitter `reports/tzdb-atlas/emit-formats.py` reads the **canonical** `atlas.tsv` (produced by
`build-atlas.py`) and re-serialises the *same* rows three ways. Nothing is hand-transcribed — the emitter
parses the TSV, asserts its 6-column header, and writes the exports from those parsed rows. Reproduce:
`python3 reports/tzdb-atlas/build-atlas.py && python3 reports/tzdb-atlas/emit-formats.py`.

## Acceptance — all met

| # | gate | result |
|---|---|---|
| 1 | Generated from the existing TSV, not hand-transcribed | ✓ `emit-formats.py` reads `atlas.tsv`; header asserted `== [axis,subject,key_fact,verdict,attribution,receipt]` |
| 2 | Row count matches TSV: 40 | ✓ `atlas.ndjson` = **40** lines = 40 TSV data rows |
| 3 | Schema validates every NDJSON row | ✓ **40/40** validate under `python jsonschema` (draft 2020-12); plus an inline no-dep check in the emitter |
| 4 | HTML clearly displays axis · subject · verdict · attribution · receipt | ✓ `atlas.html` — 6-column table, verdict bold, receipts are clickable repo-relative links, axis-count chips |
| 5 | README explains TSV = canonical, NDJSON/HTML = derived | ✓ `reports/tzdb-atlas/README.md` canonical-vs-derived table + "the TSV wins" |
| 6 | doc-staleness green | ✓ (see Gate) |
| 7 | CORE.1 unchanged | ✓ docs/report only — no `src/` change; CORE.1 341/0/0 + 520 tests unaffected |

## Artifacts

- `atlas.ndjson` — 40 objects, one per line, keys sorted (`axis · subject · key_fact · verdict · attribution · receipt`).
- `atlas.schema.json` — `zic-rs-atlas-row-v1`, draft 2020-12, `additionalProperties:false`, all 6 fields
  `required` + `minLength:1`; `axis`/`attribution` carry enums **derived from the current TSV** (so they
  cannot drift from the rows they validate); `verdict` intentionally un-enumed (axis-local vocabulary).
- `atlas.html` — generated table; axis tally chips (vendor_oracle 19 · drop_in 6 · upstream_archive 4 ·
  reader 4 · upstream_data 2 · zic_rs 2 · source_shape 2 · reference_zic 1).
- `README.md` — canonical-vs-derived contract, regenerate commands, `jq` usage, non-claims.

## Non-claims

- Format convenience, **not** authority — each `receipt` field still points at the actual backing artifact.
- The schema enums reflect the *current* atlas; a new axis in `build-atlas.py` regenerates the TSV and (via
  `emit-formats.py`) the enum together — generated-from-the-same-rows, so no silent drift.
- Not a new evidence axis; the 40 rows are exactly the existing atlas, re-encoded.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 520 tests unaffected; doc-staleness green. Cross-linked
from STATUS · `reports/tzdb-atlas/README.md` · `docs/tzdb-evidence-atlas.md`.
