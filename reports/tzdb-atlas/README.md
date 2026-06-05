# TZDB Evidence Atlas — formats

The atlas joins zic-rs's evidence axes (upstream-archive · upstream-data · vendor-oracle · drop-in ·
reader · source-shape · zic_rs/reference_zic findings) into one queryable table with per-finding axis
attribution and a backing receipt.

## Canonical vs derived

| file | role | produced by |
|---|---|---|
| **`atlas.tsv`** | **CANONICAL** — the single source of truth (6 columns: `axis · subject · key_fact · verdict · attribution · receipt`) | `build-atlas.py` (the join over the evidence axes) |
| `atlas.ndjson` | **derived** — one JSON object per TSV row, newline-delimited (for `jq` / pandas / external tools) | `emit-formats.py` ← `atlas.tsv` |
| `atlas.schema.json` | **derived** — JSON Schema (draft 2020-12) every NDJSON row validates against (`zic-rs-atlas-row-v1`) | `emit-formats.py` ← `atlas.tsv` |
| `atlas.html` | **derived** — generated human-readable table (clickable receipts) | `emit-formats.py` ← `atlas.tsv` |

**The TSV wins.** The NDJSON / schema / HTML are re-serialisations of the *same* rows — ATLAS-FORMAT.1
makes the existing atlas easier for external reviewers and tools to parse, validate, and inspect; it
**does not create new evidence**. If they ever disagree with `atlas.tsv`, the TSV is correct and the
derived files are stale — regenerate.

## Regenerate

```sh
python3 reports/tzdb-atlas/build-atlas.py      # rebuild the canonical atlas.tsv from the axes
python3 reports/tzdb-atlas/emit-formats.py     # derive atlas.ndjson + atlas.schema.json + atlas.html
```

`emit-formats.py` asserts the TSV header matches the expected 6 columns, validates every row against the
emitted schema's row constraints inline (no external dependency required), and prints the row count.

## Use the exports

```sh
# every vendor-oracle row, just subject + verdict:
jq -r 'select(.axis=="vendor_oracle") | "\(.subject)\t\(.verdict)"' reports/tzdb-atlas/atlas.ndjson

# validate the NDJSON against the schema (if python jsonschema is installed):
python3 -c 'import json,jsonschema; s=json.load(open("reports/tzdb-atlas/atlas.schema.json")); \
[jsonschema.validate(json.loads(l),s) for l in open("reports/tzdb-atlas/atlas.ndjson")]; print("ok")'
```

## Non-claims

- The exports are a **format convenience**, not an authority — every `receipt` field points at the actual
  backing artifact, which remains the evidence.
- The schema's `axis`/`attribution` enums are derived from the *current* TSV contents; adding a new axis
  to `build-atlas.py` regenerates both the TSV and (via `emit-formats.py`) the enum — they cannot drift
  silently because the schema is generated from the same rows it validates.
- A row's `verdict` vocabulary is **axis-local** (kept distinct per axis, never collapsed) — the schema
  intentionally does not enum `verdict` across axes.

Receipt: [`RECEIPT-ATLAS-FORMAT-1.md`](RECEIPT-ATLAS-FORMAT-1.md).
