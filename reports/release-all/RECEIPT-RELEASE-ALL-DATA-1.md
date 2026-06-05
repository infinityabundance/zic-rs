# RELEASE-ALL.DATA.1 — all-stable-tzdata compile/behaviour gauntlet (2026-06-05)

> **Claim wording (binding):** *RELEASE-ALL.DATA.1 does not claim all historical civil-time data is true
> and does not claim all historical `zic` behavior is reproduced. It attempts every stable IANA tzdata
> archive and classifies each release by source compatibility, compile compatibility, and bounded fixture
> behaviour against the declared reference.*

## Method

Every **276 stable `tzdata` archives** (1993 → 2026b), each hashed + signature-classified, extracted
(`.tar.gz` or legacy `.tar.Z`), compiled with **both** reference `zic` (the host 2026b build) and `zic-rs`
(`--unsupported skip`) over the fixture-relevant region set (`africa antarctica asia australasia europe
northamerica southamerica etcetera backward`), with per-fixture `zdump` behaviour compared over 1900..2037.
Reference = current `zic` on old source (no old `zic` binaries held). Harness: `reports/release-all/data-gauntlet.sh`;
raw rows: `reports/release-all/data-gauntlet.tsv`. **No failed release is hidden** — each is classified.

Fixtures (stable across eras; unavailable zones recorded honestly): `Etc/UTC` · `Europe/Lisbon` ·
`Europe/London` · `America/New_York` · `Asia/Singapore` · `Pacific/Kiritimati` · `Australia/Lord_Howe`.

## Outcome (all 276 attempted, every release classified)

| outcome class | count | era | meaning |
|---|--:|---|---|
| **match** | **150** | 2000f–2007 + 2013→2026b | both build; all available fixtures `zdump`-behaviour-match |
| **reference-build-incompatible** | **66 → 0** | 1993 → 2000e | **current `zic` cannot build it** (the `yearistype` breakpoint — `-y` removed in tzcode 2020a). **ALL 66 CLOSED**: `--legacy-yearistype` (YEARISTYPE.1, 55) + `--legacy-empty-footer` (PERPETUAL-EXPANSION.1, the 11 perpetual-year-parity ones) → all build and match an admitted historical `zic` oracle over [1980,2037] (371/371 fixtures). Historical-source replay, not current-reference parity |
| **source-shape-incompatible** | **60 → 0** | 2008a → 2012j | **zic-rs rejects the source** (the non-UTF-8 encoding breakpoint) — **CLOSED by LEGACY-SOURCE.1**: under `--legacy-latin1` all 60 build and behaviour-match (see below) |
| **zic-rs-divergent** | **0** | — | no behaviour mismatch anywhere both compilers built the fixtures |

**Per-fixture: 1050 / 1050 behaviour-match where both compilers built the zone (150 match releases × 7),
0 mismatch.** Across 30 years of stable tzdata, zic-rs never diverged in behaviour from reference `zic` on
a release both could build.

> **Reconciled with `--legacy-latin1` (LEGACY-SOURCE.1, 2026-06-05).** The 60 `source-shape-incompatible`
> releases are the pre-2013 Latin-1 era; LEGACY-SOURCE.1 added a bounded, opt-in `--legacy-latin1` mode that
> admits Latin-1 **only inside `#` comments** (semantics-bearing fields still fail closed). Re-running the band
> under that mode moves **all 60 → match** (**420 / 420** fixture comparisons, 0 divergent), so the **legacy-mode
> tally is 210 match · 66 reference-build-incompatible · 0 source-shape-incompatible · 0 divergent**. The default
> run above stays UTF-8-required (`ZIC012`); see `reports/release-all/RECEIPT-LEGACY-SOURCE-1.md` and the
> refreshed `docs/tzdb-evidence-atlas.md` (TZDB-ATLAS.2). Reproduce: `LEGACY=1 bash reports/release-all/data-gauntlet.sh`.

## Named source-shape breakpoints (acceptance #5 — the atlas-relevant findings)

1. **The `yearistype` breakpoint (≤ 2000e → reference-build-incompatible).** Pre-2000f tzdata uses
   `year type "even"/"odd"/"uspres"` in its `Rule` lines (e.g. `australasia` line 58). Reference `zic`
   **removed `yearistype` support in tzcode 2020a**, so the *current* `zic` binary itself **cannot build**
   these releases (`year type "even" is unsupported`). This is a **reference-ecology** breakpoint, not a
   zic-rs issue: the oracle can't build old data without the old `-y yearistype` script. Clean boundary —
   `2000e` fails, `2000f` builds.
2. **The UTF-8 encoding breakpoint (2008a–2012j → source-shape-incompatible).** In this band several files
   (e.g. `southamerica`) are **ISO-8859 / Latin-1** (`file`: *ISO-8859 text*) — accented city/author names
   in non-UTF-8 bytes — before tzdb converted to **UTF-8** (`file` 2015g: *UTF-8 text*). zic-rs's input
   admissibility (**T14**) **requires UTF-8** and fail-closes (`ZIC012: source is not valid UTF-8`); current
   `zic` is byte-oriented and accepts it. This is a **documented bucket-3 safer divergence** (zic-rs refuses
   non-UTF-8 input by design), **not a behaviour bug** — the rejected bytes are in comments/names, not time
   logic. **(LIFTED by LEGACY-SOURCE.1 — 2026-06-05.)** The bounded, opt-in `--legacy-latin1` mode now admits
   the Latin-1 bytes *because* they are confined to `#` comments (any semantics-bearing non-UTF-8 byte still
   fails closed), so all 60 releases build and behaviour-match reference `zic` under that mode; the default
   remains UTF-8-required. See `reports/release-all/RECEIPT-LEGACY-SOURCE-1.md`.

## Honesty / non-claims

- **Not all historical data is true / not all `zic` behaviour reproduced.** 66 releases the reference can't
  build; 60 zic-rs declines (encoding); only the 150 both-built releases carry a behaviour verdict (and it
  is **match**, 0 divergent).
- The reference is *current `zic` on old source* — so "reference-build-incompatible" is a fact about the
  current `zic` binary + the old source, not about any historical `zic`.
- Fixture set (7 zones), not all zones of each release; `zdump` over 1900..2037; bounded.
- The `yearistype` and UTF-8 classifications are about the **fixture-relevant region set**; `solar*`/`systemv`/
  `pacificnew` (removed-feature noise) are deliberately excluded so the class reflects fixture-relevant source.

## Reproduce

```sh
bash reports/release-all/data-gauntlet.sh   # uses the cached archives from RELEASE-ALL.1; emits data-gauntlet.tsv
```

This is **axis 1 (behaviour)** of the TZDB Evidence Atlas now carrying real outcomes (not only
provenance) — ready to join with the vendor-oracle and drop-in axes (`TZDB-ATLAS.1`).
