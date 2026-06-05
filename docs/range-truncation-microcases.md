# `-r` range-truncation microcases — reference fixtures (T10.4c)

> **These are NOT implementation tests yet.** They are **reference fixtures** captured from real
> `zic -r`, pinning the exact output that the T10.4d implementation must reproduce. Oracle-first, same
> discipline that made CORE.1 strong: pin the ground truth *before* writing the clamp + `-00` logic.
> Raw reference TZif bytes live in [`fixtures/range/reference/`](../fixtures/range/reference/) (hashed
> below); this doc is the human-readable decode.

## Provenance

| Item | Value |
|------|-------|
| `zic` | tzcode **2026b** (`zic (tzcode) 2026b-dirty`) |
| source | `/usr/share/zoneinfo/tzdata.zi`, version `2026b`, sha256 `0078657f…6334c371` |
| decoder | `/tmp/tzif_decode.py` (minimal RFC 9636 v2/v3 block decoder) |

These fixtures are **release-scoped** to that `zic`/tzdata pair (like every behaviour claim). A
different tzdata release will produce different bytes; regenerate + re-pin then.

| Fixture | Zone | `-r` argument | sha256 |
|---------|------|---------------|--------|
| `ny.lo-2000.tzif` | `America/New_York` | `@946684800` (lo = 2000-01-01) | `2f5f1a33…e565dde6` |
| `utc.win-2000-2020.tzif` | `Etc/UTC` | `@946684800/@1577836800` (2000..2020 window) | `62a73dc4…450b5e0a` |
| `london.lo-1975.tzif` | `Europe/London` | `@157766400` (lo = 1975-01-01) | `5e2ad91b…aa7b8acd` |
| `gaza.lo-2020.tzif` | `Asia/Gaza` | `@1577836800` (lo = 2020-01-01) | `8f303afc…125f3cea` |
| `gmtplus5.win-2000-2020.tzif` | `Etc/GMT+5` | `@946684800/@1577836800` (window) | `440959c0…5227a62e` |

## Invariants observed across all four (the rules T10.4d must obey)

1. **Leading `-00` on start-truncation.** When `lo` is cut, the file gains a synthetic **type 0 =
   `(utoff 0, isdst 0, abbr "-00")`** meaning *local time unspecified* before `lo` — **not** UTC. A
   transition lands **at exactly `lo`** switching to the first real in-range type.
2. **Trailing `-00` + empty footer on end-truncation.** When `hi` is bounded (the UTC window), a
   transition lands **at exactly the parsed `@hi`** switching **back** to `-00`, and the **footer is
   emptied** (`""`). (`zic` computes `hi -= 1` then emits the trailing transition at `hi + 1`, so it
   nets to the parsed `@hi`.) When `hi` is unbounded, the recurring **footer is preserved**.
3. **TZif version is content-driven, unaffected by `-r`.** Gaza stays **v3** (its law-10 re-anchored
   footer); the others stay v2.
4. **`-r` is not a transition filter.** It reshapes the representable interval and *adds* the `-00`
   boundary type(s); it never just drops transitions.

## America/New_York — `-r @946684800` (lo only; rich DST + footer kept)

```
version=v2  isutcnt=0 isstdcnt=0 leapcnt=0 timecnt=16 typecnt=3 charcnt=12
types:  [0] utoff=+0     isdst=0 abbr="-00"   <-- leading unspecified
        [1] utoff=-14400 isdst=1 abbr="EDT"
        [2] utoff=-18000 isdst=0 abbr="EST"
first transitions:  946684800 -> [2] EST   (== lo; first real in-range type)
                    954658800 -> [1] EDT
                    972799200 -> [2] EST  …
footer="EST5EDT,M3.2.0,M11.1.0"           (hi unbounded → recurring tail preserved)
```
zdump (1999..2001): `… 1999 -00 gmtoff=0` then at `Sat Jan 1 00:00:00 2000 UT` → `EST gmtoff=-18000`.
Everything before `lo` is `-00` (unspecified), **not** EST.

## Etc/UTC — `-r @946684800/@1577836800` (window; fixed-offset, both boundaries)

```
version=v2  isutcnt=0 isstdcnt=0 leapcnt=0 timecnt=2 typecnt=2 charcnt=8
types:  [0] utoff=+0 isdst=0 abbr="-00"
        [1] utoff=+0 isdst=0 abbr="UTC"
transitions:  946684800  -> [1] UTC    (== lo)
              1577836800 -> [0] -00    (== parsed @hi; back to unspecified)
footer=""                               (bounded hi → NO recurring tail)
```
Shows that even a **fixed-offset** zone is not a no-op under a *windowed* `-r`: it gains the leading
**and** trailing `-00`, and loses its footer.

## Europe/London — `-r @157766400` (lo 1975; multi-era historical)

```
version=v2  isutcnt=0 isstdcnt=0 leapcnt=0 timecnt=44 typecnt=3 charcnt=12
types:  [0] utoff=+0    isdst=0 abbr="-00"
        [1] utoff=+3600 isdst=1 abbr="BST"
        [2] utoff=+0    isdst=0 abbr="GMT"
first transitions:  157766400 -> [2] GMT   (== lo)
                    164167200 -> [1] BST  …
footer="GMT0BST,M3.5.0/1,M10.5.0"
```

## Asia/Gaza — `-r @1577836800` (lo 2020; far-future finite Ramadan + v3 footer)

```
version=v3  isutcnt=0 isstdcnt=0 leapcnt=0 timecnt=195 typecnt=3 charcnt=13
types:  [0] utoff=+0     isdst=0 abbr="-00"
        [1] utoff=+10800 isdst=1 abbr="EEST"
        [2] utoff=+7200  isdst=0 abbr="EET"
first transitions:  1577836800 -> [2] EET   (== lo)
                    1585346400 -> [1] EEST …
footer="EET-2EEST,M3.4.4/50,M10.4.4/50"    (law-10 v3 footer preserved)
```
Confirms `-r` start-truncation composes with the CORE.1 law-10 re-anchored **v3** footer: version and
footer unchanged, leading `-00`, finite Ramadan rows retained after `lo`.

## Etc/GMT+5 — `-r @946684800/@1577836800` (nonzero fixed offset, no transition in range)

The hardening case: a zone whose real type is a **nonzero fixed offset** (`-05`, utoff −18000) with
**no transition inside the window**. It proves the `-00` placeholder (utoff **+0**) is a *distinct*
type and never masquerades as the real offset.

```
version=v2  isutcnt=0 isstdcnt=0 leapcnt=0 timecnt=2 typecnt=2 charcnt=8
types:  [0] utoff=+0     isdst=0 abbr="-00"   <-- placeholder: offset 0, NOT the real offset
        [1] utoff=-18000 isdst=0 abbr="-05"   <-- real fixed offset, preserved
transitions:  946684800  -> [1] -05    (== lo)
              1577836800 -> [0] -00    (== parsed @hi)
footer=""
```
zdump: `… 1999 -00 gmtoff=0`, then `Sat Jan 1 00:00:00 2000 UT = … -05 gmtoff=-18000`, then back to
`-00 gmtoff=0` at 2020. **`-00` (offset 0, local time unspecified) ≠ the zone's real `-05` offset** —
a T10.4d that collapsed or reused type 0 for the in-range data would corrupt this; the fixture catches
it.

## What T10.4d must reproduce (acceptance, deferred)

For each fixture: byte-identical TZif (or, at minimum, identical decoded `counts` + `types` incl. the
`-00` + transition list + footer + version), and `zdump`-identical behaviour over the truncated
horizon. Only after these microcases pass should T10.4e run the full 341-zone `-r` gate. The clamp,
the `-00` boundary types, and the footer-emptying-on-`hicut` are all to be implemented **without**
merging with `-R` (a separate clamp input); `EmitOptions` will gain a **distinct `range` field**, not
overload `redundant_until`.
