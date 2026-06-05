# `reports/release-diff/` — real tzdb release-diff receipts (T23.release-diff-real.1)

Operator-path evidence: *can `zic-rs release-diff` compare a **real** prior tzdb release to the current
admitted release, on actual upstream change structure — not synthetic/self-diff?*

## Receipts (append-only)

| Date | Releases (official, OpenPGP-verified, hash-pinned) | Oracle | Result |
|---|---|---|---|
| 2026-06-03 | tzdata **2026a** → **2026b** (sig key `7E37…7E34` Paul Eggert; GOODSIG/VALIDSIG) | `reference_zdump` | **597 unchanged · 1 `behavior_past_and_future` = `America/Vancouver`** (independently confirmed real: source diff + reference-`zic` + NEWS) — `RECEIPT-2026a-2026b.md`, `diff-2026a-2026b.json` |

## Reproduce

```sh
B=https://data.iana.org/time-zones/releases
for r in 2026a 2026b; do curl -O $B/tzdata$r.tar.gz; curl -O $B/tzdata$r.tar.gz.asc; \
  gpg --verify tzdata$r.tar.gz.asc tzdata$r.tar.gz; sha256sum tzdata$r.tar.gz; \
  mkdir src$r && tar -xzf tzdata$r.tar.gz -C src$r; \
  mkdir clean$r && for f in africa antarctica asia australasia etcetera europe factory northamerica southamerica backward; do cp src$r/$f clean$r/; done; done
zic-rs release-diff --old clean2026a --new clean2026b --reference-zdump zdump --format json
```

**Non-claims:** one real delta does not prove all future diffs · not civil-time truth · not distro/package
behaviour · not untested readers · not exact diagnostic-wording parity. Scoped to the primary source set
over `1900,2040`.
