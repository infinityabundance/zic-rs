# RECEIPT — TZDB-ATLAS.2 (atlas refresh: Latin-1 source-shape band closed) — 2026-06-05

> *TZDB-ATLAS.2 does not create new compiler parity evidence. It refreshes the TZDB-ATLAS.1 join so the public
> evidence atlas reflects the one zic-rs-side source-shape band that LEGACY-SOURCE.1 closed: the 2008a–2012j
> Latin-1 era now behaviour-matches reference `zic` under the explicit `--legacy-latin1` replay mode.*

## What .2 changes (refresh, not new evidence)

TZDB-ATLAS.1 named exactly one zic-rs-side compatibility band: **2008a–2012j → `source-shape-incompatible`**
(non-UTF-8/Latin-1 source, `deferred`). LEGACY-SOURCE.1 closed it. .2 reconciles the public join to that fact —
**docs + atlas.tsv + receipt only; no `src/` change**.

- **`reports/tzdb-atlas/build-atlas.py`** — the `upstream_data` row now records **both** tallies (computed from
  `data-gauntlet.tsv` + the LEGACY-SOURCE.1 receipt); the `source_shape` row flips to **CLOSED** with the
  `bounded_legacy_source` attribution pointing at `RECEIPT-LEGACY-SOURCE-1.md`.
- **`reports/tzdb-atlas/atlas.tsv`** — regenerated (`python3 build-atlas.py`); 30 rows, attributions now include
  `bounded_legacy_source`.
- **`docs/tzdb-evidence-atlas.md`** — title → ATLAS.2; "what changed" note; the joined table, headline, and
  questions-answered all carry the dual tally + the closed band.

## The reconciled tally (acceptance: DATA.1 reconciled with LEGACY mode)

| mode | match | reference-build-incompat (`yearistype`) | source-shape-incompat (Latin-1) | divergent |
|---|---|---|---|---|
| **default (UTF-8-required)** | 150 | 66 | 60 | 0 |
| **`--legacy-latin1`** | **210** | 66 | **0** | 0 |

Behaviour divergence is **0 in both modes**; legacy mode adds **420/420** fixture comparisons, all match. The
default UTF-8 source contract is byte-unchanged (the modern path never takes the legacy branch).

## The clean atlas story now

```text
2008–2012 Latin-1 band   →  CLOSED by bounded legacy-source replay (LEGACY-SOURCE.1)  →  bounded_legacy_source
≤2000e yearistype band   →  reference-zic ecology limit (current zic can't build it)  →  reference_zic
```

The only remaining historical wall is the `yearistype` band, and it is **attributable to the reference**, not a
zic-rs divergence. Every tzdb release real vendors ship (2023c · 2025b · 2026a · 2026b) remains in the upstream
`match` band — the join the single matrices can't answer is unchanged and still clean.

## Non-claims

- **No new evidence** — .2 only re-attributes the existing RELEASE-ALL.DATA.1 + LEGACY-SOURCE.1 receipts; each
  atlas cell still links to its source receipt.
- The legacy tally is the LEGACY-SOURCE.1-verified band result; the committed `data-gauntlet.tsv` remains the
  **default UTF-8** run (re-runnable in legacy mode via `LEGACY=1 bash reports/release-all/data-gauntlet.sh`).
- Bounded fixtures; `zdump` over 1900..2037; vendor/drop-in axes are bounded ecologies, not exhaustive; not all
  historical civil-time data is claimed true (IANA/CLDR own that); the `yearistype` band is **not** reopened
  here (a separate opt-in `-y` replay mode is deliberately *not* added).

## Gate

Docs/report only — no `src/` change; **CORE.1 341/0/0** unchanged; doc-staleness green. Cross-linked STATUS ·
TRUST · README. Rebuild: `python3 reports/tzdb-atlas/build-atlas.py`.
