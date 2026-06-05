# Real release-diff receipt — tzdb 2026a → 2026b (T23.release-diff-real.1)

The first **real** old→new tzdb release-diff (not synthetic/self-diff): two **official, OpenPGP-verified,
hash-pinned** IANA source releases compared by `zic-rs release-diff` with the `zdump` behaviour oracle.

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64 · **zic-rs:** 0.1.0 (this tree).

## Source admission (the T12.5a.2 discipline: fetch → verify-sig → hash-pin)

| release | canonical URL | sha256 | bytes | signature |
|---|---|---|---|---|
| **2026a** | `https://data.iana.org/time-zones/releases/tzdata2026a.tar.gz` | `77b541725937bb53bd92bd484c0b43bec8545e2d3431ee01f04ef8f2203ba2b7` | 471812 | **GOODSIG + VALIDSIG** |
| **2026b** | `https://data.iana.org/time-zones/releases/tzdata2026b.tar.gz` | `114543d9f19a6bfeb5bca43686aea173d38755a3db1f2eec112647ae92c6f544` | 473703 | **GOODSIG + VALIDSIG** |

- **Signing key (admitted, fingerprint-anchored):** RSA `7E3792A9D8ACF7D633BC1588ED97E90E62AA7E34` — *Paul
  Eggert* (the published tz key; same key admitted in `reports/t12_5a2-reference-admission.md`). Both
  detached `.asc` signatures verified `GOODSIG`/`VALIDSIG`.
- **Provenance posture:** official IANA release tarballs; bytes live in `/tmp` (ephemeral, **not vendored**),
  pinned **by hash** + signature → re-fetchable, reproducible. `version` files confirm `2026a` / `2026b`.

## Run

```sh
# clean primary-source dirs (region files + backward; no .tab/.awk/Makefile):
#   africa antarctica asia australasia etcetera europe factory northamerica southamerica backward
zic-rs release-diff --old <clean-2026a> --new <clean-2026b> --reference-zdump zdump --format json
```

Artifact: `reports/release-diff/diff-2026a-2026b.json` (`zic-rs-release-diff-v1`).

## Result

- **`oracle_mode`: `reference_zdump`** (behaviour axis active; `/usr/bin/zdump`).
- **598 identifiers — 597 unchanged · 1 `behavior_past_and_future`.** (A small, realistic release delta.)

```json
{"added":0,"behavior_future":0,"behavior_past":0,"behavior_past_and_future":1,
 "behaviour_unassessed":0,"leap_only":0,"link_changed":0,"metadata_only":0,
 "removed":0,"unchanged":597}
```

- **The one changed zone: `America/Vancouver`** — structural `differing: [timecnt, typecnt, charcnt, footer]`,
  behaviour classified **past *and* future**.

## Independent confirmation (it is a genuine tzdb delta, not a zic-rs artifact)

1. **Source diff** (`northamerica`, 2026a→2026b) shows 2026b modified `America/Vancouver` — a "Temporary
   hack" era (`-8:00 Canada P%sT` until `2026 Mar 9`, then a one-off `PDT` to `2026 Nov 1`, then `-7:00 MST`).
2. **2026b NEWS** records pre-1996 northern-Canada timestamp fixes.
3. **Reference `zic` independently** compiles a *different* `America/Vancouver` for 2026a vs 2026b (`cmp`
   differs) — so the single detected change is real and externally witnessed.

The behaviour oracle (`zdump`) therefore had a genuine difference to classify, and zic-rs's `release-diff`
classified exactly the one real changed zone with both a past and a future component — and **collapsed nothing
into "unchanged/no-diff"** (the typed-failure acceptance: when the oracle was omitted, the same zone was
`behaviour_unassessed`, never silently "unchanged").

## Non-claims (loud)

- A real 2026a→2026b diff **does not** prove all future release diffs · **not** civil-time truth · **not**
  distro/package behaviour · **not** untested readers · **not** exact diagnostic-wording parity.
- Scoped to the **primary source set** (region files + `backward`) over the default `1900,2040` horizon;
  `backzone`/variant profiles not included here.
