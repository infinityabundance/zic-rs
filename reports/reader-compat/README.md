# `reports/reader-compat/` — reader-compatibility gauntlet (T23.reader-compat.1 / .2 / .3)

Output-consumer evidence: *do real TZif **readers** consume zic-rs's output the same as reference-`zic`'s
output across RFC 9636 Appendix-A / known-reader traps?* The microcase ledger is
`docs/reader-compatibility-microcases.md`; this folder holds the harness + dated receipts.

- **`.1`** (`RECEIPT-2026-06-03-first-cut.md`) — 12 Appendix-A microcases × {`zdump`, Python `zoneinfo`}.
- **`.2`** (`RECEIPT-2026-06-03-appendix-a.md`) — 15 edge fixtures incl. v4/`-r`/`right`; **found + fixed**
  the right/-leap transition-shift bug.
- **`.3`** (`RECEIPT-T23-reader-compat-3.md`) — **cross-reader ecology**: 6 T18-ledger fixtures × {glibc 2.43 ·
  Go 1.26 · CCTZ/abseil} all see zic-rs ≡ reference (16 match · 1 match-with-known-limit · **0 mismatch**);
  Java/PHP/ICU classified `unsupported_by_reader` (consume a compiled DB, not raw TZif). Harness in `r3/`.

**Tier (loud):** `memory-safe ≠ format-valid ≠ reader-compatible ≠ semantic parity ≠ civil-time truth`. A
`matched` row = these named readers, at these named instants, observed identical behaviour from the zic-rs
and reference-`zic` TZif. Nothing more.

## Reproduce

```sh
# 1. Build both TZif trees from the same admitted source (uses the release binary + reference zic):
target/release/zic-rs compile --all-supported --input /usr/share/zoneinfo/tzdata.zi --out /tmp/rc_rs
zic -d /tmp/rc_ref /usr/share/zoneinfo/tzdata.zi
# 2. Run the gauntlet (exit 0 = no divergence):
python3 reports/reader-compat/gauntlet.py
```

## Receipts (append-only)

| Date | Readers run | Result |
|---|---|---|
| 2026-06-03 | `zdump` (tzcode 2026b) · Python `zoneinfo` 3.14.5 · Go=unassessed | **24/24 reader-pairs matched · 0 diverged** (12 microcases × 2 readers); 4/12 byte-identical — `RECEIPT-2026-06-03-first-cut.md` |
| 2026-06-03 | same readers · **Appendix-A expansion** (right/leap · v4 · `-r` truncation · footer/boundary/abbrev/offset) | **30/30 reader-pairs matched** (15 microcases × 2 readers) — but **found AND fixed a real `right/` leap transition-encoding defect** (first run 1 diverged); harness `gauntlet-appendix-a.py` — `RECEIPT-2026-06-03-appendix-a.md` |

## Next cuts (on demand)

- Add **Go** (`time.LoadLocationFromTZData`), then **CCTZ**, **Timelib/PHP**, **Java**, **ICU** as
  available — each lands as `matched`/`diverged`/`unassessed`, never assumed.
- Broaden microcases (permanent-DST, leap-`right/` profile, v4) and toward the full 598-zone set.
- A divergence, if ever found, is captured with the reader, instant, and both observations — no fake green.
