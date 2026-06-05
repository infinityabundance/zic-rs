#!/usr/bin/env python3
# ATLAS-FORMAT.1 — derive machine-readable + human-readable exports from the CANONICAL atlas.tsv.
#
# Doctrine: atlas.tsv (produced by build-atlas.py) is the single source of truth. This script does NOT
# create evidence — it reads the TSV and re-serialises the SAME rows as:
#   atlas.ndjson       one JSON object per TSV row (newline-delimited; external tools / jq / pandas)
#   atlas.schema.json  JSON Schema (draft 2020-12) every NDJSON row validates against
#   atlas.html         a generated human-readable table (axis · subject · verdict · attribution · receipt)
# Reproduce: python3 reports/tzdb-atlas/build-atlas.py && python3 reports/tzdb-atlas/emit-formats.py
import json, html, collections

SRC = 'reports/tzdb-atlas/atlas.tsv'
COLS = ['axis', 'subject', 'key_fact', 'verdict', 'attribution', 'receipt']

lines = open(SRC).read().splitlines()
header = lines[0].split('\t')
assert header == COLS, f"atlas.tsv header drift: {header} != {COLS}"
rows = [dict(zip(COLS, ln.split('\t'))) for ln in lines[1:] if ln]
N = len(rows)

# --- atlas.ndjson : one JSON object per row, in TSV order (newline-delimited) ---
with open('reports/tzdb-atlas/atlas.ndjson', 'w') as f:
    for r in rows:
        f.write(json.dumps(r, ensure_ascii=False, sort_keys=True) + '\n')

# --- atlas.schema.json : draft 2020-12; every NDJSON row must validate ---
schema = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://github.com/infinityabundance/zic-rs/reports/tzdb-atlas/atlas.schema.json",
    "title": "zic-rs TZDB Evidence Atlas row (zic-rs-atlas-row-v1)",
    "description": "One evidence-atlas finding. DERIVED from the canonical reports/tzdb-atlas/atlas.tsv "
                   "(ATLAS-FORMAT.1) — this schema validates the NDJSON projection, it does not define new "
                   "evidence. Each row joins one of four evidence axes to a receipt with explicit attribution.",
    "type": "object",
    "additionalProperties": False,
    "required": COLS,
    "properties": {
        "axis": {"type": "string", "minLength": 1,
                 "description": "evidence axis the finding belongs to",
                 "enum": sorted({r['axis'] for r in rows})},
        "subject": {"type": "string", "minLength": 1,
                    "description": "what the finding is about"},
        "key_fact": {"type": "string", "minLength": 1,
                     "description": "the receipt-backed fact, stated honestly with its bounds"},
        "verdict": {"type": "string", "minLength": 1,
                    "description": "the row's classification (axis-local vocabulary; not collapsed across axes)"},
        "attribution": {"type": "string", "minLength": 1,
                        "description": "which surface the evidence is attributed to",
                        "enum": sorted({r['attribution'] for r in rows})},
        "receipt": {"type": "string", "minLength": 1,
                    "description": "path to the backing receipt/doc/tsv (repo-relative)"},
    },
}
with open('reports/tzdb-atlas/atlas.schema.json', 'w') as f:
    json.dump(schema, f, indent=2, ensure_ascii=False)
    f.write('\n')

# --- minimal inline validation: every row satisfies the schema's row constraints (no external dep) ---
ax_enum = set(schema['properties']['axis']['enum'])
at_enum = set(schema['properties']['attribution']['enum'])
for i, r in enumerate(rows):
    assert set(r) == set(COLS), f"row {i}: key drift"
    for c in COLS:
        assert isinstance(r[c], str) and len(r[c]) >= 1, f"row {i}: empty {c}"
    assert r['axis'] in ax_enum and r['attribution'] in at_enum, f"row {i}: enum miss"

# --- atlas.html : generated human-readable table ---
axes = collections.Counter(r['axis'] for r in rows)
verds = collections.Counter(r['verdict'] for r in rows)
def td(s): return f"<td>{html.escape(s)}</td>"
def rcpt(s):
    return f'<td><a href="../../{html.escape(s)}"><code>{html.escape(s)}</code></a></td>'
body = []
for r in rows:
    body.append("<tr>" + td(r['axis']) + td(r['subject']) + f"<td>{html.escape(r['key_fact'])}</td>"
                + f'<td class="v">{html.escape(r["verdict"])}</td>' + td(r['attribution']) + rcpt(r['receipt']) + "</tr>")
axis_chips = " ".join(f'<span class="chip">{html.escape(a)}: {n}</span>' for a, n in sorted(axes.items()))
htmldoc = f"""<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8">
<title>zic-rs — TZDB Evidence Atlas ({N} rows)</title>
<style>
 body{{font:14px/1.5 system-ui,sans-serif;margin:2rem;color:#1a1a1a;max-width:1400px}}
 h1{{font-size:1.4rem}} p.note{{color:#555}}
 .chip{{display:inline-block;background:#eef;border:1px solid #ccd;border-radius:4px;padding:1px 7px;margin:2px;font-size:12px}}
 table{{border-collapse:collapse;width:100%;margin-top:1rem;font-size:13px}}
 th,td{{border:1px solid #ddd;padding:6px 8px;text-align:left;vertical-align:top}}
 th{{background:#f4f4f8;position:sticky;top:0}}
 tr:nth-child(even){{background:#fafafc}}
 td.v{{font-weight:600;white-space:nowrap}} code{{font-size:12px}}
 tr:hover{{background:#fffbe6}}
</style></head><body>
<h1>zic-rs — TZDB Evidence Atlas</h1>
<p class="note"><strong>{N} rows.</strong> Generated by <code>reports/tzdb-atlas/emit-formats.py</code> from the
canonical <code>reports/tzdb-atlas/atlas.tsv</code> (ATLAS-FORMAT.1) — this HTML is a <em>derived</em> view, not a
source of evidence. The TSV wins; regenerate with
<code>python3 reports/tzdb-atlas/build-atlas.py &amp;&amp; python3 reports/tzdb-atlas/emit-formats.py</code>.</p>
<p>{axis_chips}</p>
<table><thead><tr><th>axis</th><th>subject</th><th>key_fact</th><th>verdict</th><th>attribution</th><th>receipt</th></tr></thead>
<tbody>
{chr(10).join(body)}
</tbody></table>
</body></html>
"""
with open('reports/tzdb-atlas/atlas.html', 'w') as f:
    f.write(htmldoc)

print(f"ATLAS-FORMAT.1: {N} rows → atlas.ndjson · atlas.schema.json · atlas.html (all validated against schema)")
print("axes:", dict(axes))
