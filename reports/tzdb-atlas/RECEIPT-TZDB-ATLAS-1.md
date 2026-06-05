# RECEIPT — TZDB-ATLAS.1 (upstream × vendor × drop-in evidence join) — 2026-06-05

> *TZDB-ATLAS.1 does not create new compiler parity evidence. It joins existing upstream-release,
> vendor-oracle, reader-compatibility, and drop-in receipt evidence so that differences are attributable to
> their evidence axis rather than treated as undifferentiated zic-rs failures.*

## What was joined

- **Upstream archive** — `reports/release-all/index.tsv` (785 entries) + `verified.tsv` (55/55 GOODSIG).
- **Upstream behaviour** — `reports/release-all/data-gauntlet.tsv` (276 releases: 150 match · 66 ref-build-incompat
  · 60 source-shape-incompat · **0 divergent**).
- **Vendor-oracle** — 19 `../zic-rs-vendor-oracle-lab/receipts/*.json` (lineage × version × shipped tzdb).
- **Drop-in** — `reports/drop-in/` (bundle_hash `453641ff…` byte-identical across 15+ environments).
- **Reader-compat** — `reports/reader-compat/` (glibc/Go/CCTZ match; Java/PHP/ICU unsupported-by-reader).

Output: `reports/tzdb-atlas/atlas.tsv` (30 joined rows, each with an **axis attribution**) + the queryable
`docs/tzdb-evidence-atlas.md`. Rebuild: `python3 reports/tzdb-atlas/build-atlas.py`.

## The headline

**Across all stable tzdata releases where both current reference `zic` and zic-rs build the bounded fixture
set, 0 zic-rs behaviour divergences** (1050/1050; 150 both-built releases). The two historical breakpoints are
attributable, not zic-rs behaviour bugs:
- **`yearistype` removal** (≤2000f) → `reference_zic` axis (the *current reference* can't build old data).
- **non-UTF-8/Latin-1 source** (2008–2012) → `deferred`/`source_shape` axis (zic-rs T14 UTF-8 policy).

## The cross-axis result (the novel join)

**Every tzdb data release real vendors ship — 2023c · 2025b · 2026a · 2026b — falls in the upstream `match`
band; none in a `reference-build-incompatible` or `source-shape-incompatible` era.** The data production
systems compile is exactly what zic-rs behaviour-matches with 0 divergence.

## Axis attributions present (acceptance #5)

`upstream_archive` · `source_shape` · `reference_zic` · `vendor_zic` · `reader` · `drop_in_environment` ·
`zic_rs` · `unsupported_by_design` · `deferred` — every joined row carries one.

## Non-claims

Not all historical data true; not all old `zic` behaviour reproduced (reference = current `zic` on old
source); **no new evidence** — only join + attribution over existing receipts; bounded fixtures/ecologies.

## Gate

Docs/harness only — no `src/` change; CORE.1 341/0/0; doc-staleness green. Cross-linked STATUS · TRUST ·
README · REVIEW-IN-10-MINUTES · iana-release-archive-ledger · provenance-ledger · zic-operational-parity-matrix.
