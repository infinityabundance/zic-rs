# Reader-compatibility gauntlet — receipt (T23.reader-compat.1, first cut)

- **Date:** 2026-06-03
- **Host:** Linux 7.0.9-1-cachyos x86_64
- **Run:** `python3 reports/reader-compat/gauntlet.py` (exit 0 = no divergence)
- **Scope:** 12 RFC 9636 Appendix-A / reader-trap microcase zones × 2 real readers, at 14 UTC probe
  instants spanning **~−2³¹ (1901-12-13 20:45:52) … ~+2³¹ (2038-01-19 03:14:07/08) … 2200**.
- **Method (the tier-correct comparison):** each microcase zone is compiled by **both** zic-rs and
  reference `zic` from the same admitted source; each reader consumes **both** TZif files; a row is
  `matched` iff that reader observed identical behaviour from the two files (output-consumer equivalence),
  `diverged` if not, `unassessed` if the reader is absent/errored.

## Identities

| Artifact | Identity |
|---|---|
| zic-rs | `target/release/zic-rs` (this tree); `compile --all-supported` |
| reference `zic` (ref tree producer) | `zic (tzcode) 2026b-dirty` |
| source | `/usr/share/zoneinfo/tzdata.zi` sha256 `0078657fd0b76865…` |
| Reader A | `zdump (tzcode) 2026b-dirty` — `zdump -v -c 1900,2041 <file>` (path token stripped for comparison) |
| Reader B | CPython **3.14.5** `zoneinfo.ZoneInfo.from_file` — per-instant `(utcoffset, dst, tzname)` |
| Reader C | Go `time` — **NOT RUN** (Go absent on host) → `unassessed` |

## Input fixtures (sha256, zic-rs | reference `zic`)

| zone | zic-rs | reference `zic` | byte-id |
|---|---|---|---|
| `Etc/UTC` | `fddce1e648a1732a…` | `fddce1e648a1732a…` | yes |
| `Europe/Dublin` | `d88a38a08a280366…` | `11c00336e02f1318…` | no |
| `Asia/Gaza` | `85db6553f359e955…` | `f8f0bffe018e0da0…` | no |
| `Africa/Cairo` | `b5a301a2a6019f6e…` | `89cb9a36212fb82e…` | no |
| `Asia/Kathmandu` | `76b8f1bfe072231a…` | `76b8f1bfe072231a…` | yes |
| `Australia/Eucla` | `ec4ed99ed31c997e…` | `dcdaac15f33347af…` | no |
| `America/Argentina/Buenos_Aires` | `c911c2f508cbc597…` | `20454ea527c8ea88…` | no |
| `Antarctica/Troll` | `b38cf417fb8acf1d…` | `b38cf417fb8acf1d…` | yes |
| `Europe/London` | `7c27432f530167b1…` | `676541f0b8ad457c…` | no |
| `America/New_York` | `8d0d951800a68a0c…` | `d7f2206b3a45989f…` | no |
| `Pacific/Kiritimati` | `71454698c4418259…` | `71454698c4418259…` | yes |
| `Europe/Amsterdam` | `1d683bc35b40e2e3…` | `b10f9542a8509f0a…` | no |

## Observed result

```text
PAIR TOTALS (24 = 12 zones × 2 readers): matched=24  diverged=0  unassessed=0
byte-identical zic-rs vs ref-zic: 4/12
```

- **24/24 reader-pairs matched · 0 diverged** (zdump and Python zoneinfo each saw identical behaviour
  from the zic-rs and reference-`zic` TZif for all 12 microcases).
- **8/12 fixtures are NOT byte-identical** (default fat vs slim + footer/representation differences) yet
  still read identically → **reader compatibility holds where bytes legitimately differ** (the finding that
  justifies this rung existing separately from byte/structural parity).
- **Go reader: unassessed** (absent) — not a pass.

## Non-claims

- This is output-consumer evidence only: **does not** prove civil-time truth, reference-`zic` parity beyond
  these readers/instants, or acceptance by any untested reader/runtime.
- `unassessed` ≠ pass. A `matched` is scoped to **these readers, these zones, these probe instants, this host**.
- Not a fuzz/exhaustive sweep over all 598 zones — a curated trap set; broadening readers (Go/CCTZ/…) and
  zones is the next cut.
