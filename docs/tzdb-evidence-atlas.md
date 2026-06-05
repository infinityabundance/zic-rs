# TZDB Evidence Atlas (TZDB-ATLAS.4)

> **Claim wording (binding):** *TZDB-ATLAS does not create new compiler parity evidence. It joins existing
> upstream-release, vendor-oracle, reader-compatibility, and drop-in receipt evidence so that differences are
> attributable to their evidence axis rather than treated as undifferentiated zic-rs failures.*
>
> **What changed in .2:** the **one** zic-rs-side source-shape band that TZDB-ATLAS.1 identified (the 2008a–2012j
> Latin-1 era) was **closed by LEGACY-SOURCE.1** — `--legacy-latin1` admits Latin-1 **in comments only** → all
> **60** releases behaviour-match reference `zic` (420/420 fixtures, 0 divergent).
>
> **What changed in .3:** the last historical wall — the **≤2000e `yearistype`** band (66 releases the *current*
> reference `zic` cannot build, because tzcode 2020a removed `-y`) — was split by **YEARISTYPE.1**.
> `--legacy-yearistype` admits the historical Rule `TYPE` predicates (`even`/`odd`, anchored to `yearistype.sh`
> v7.4; zic-rs never executes the script) → **55/66 build and match an admitted historical `zic` oracle
> byte-identically** (354/354 fixtures, 0 divergent). This is **historical-source replay, not current-reference
> parity**. The remaining **11** (1993–94) hit a *separate, pre-existing* feature — their even/odd rules were
> **perpetual** (`1990 max even/odd`), so the recurring tail is **not POSIX-footer-expressible**; zic-rs accepts
> the TYPE then defers the footer (`ZIC001`). Honestly deferred (`zic_rs` axis), **not** a yearistype failure.
>
> **What changed in .4:** those 11 perpetual-footer releases were **closed by PERPETUAL-EXPANSION.1**.
> `--legacy-empty-footer` emits the explicit transitions zic-rs already expands through `RECUR_HI = 2037`
> plus an **empty footer** (frozen beyond) on the synthesis-failure path only → all **66/66** yearistype-band
> releases now build and behaviour-match the historical oracle over `[1980, 2037]` (371/371 fixtures, 0
> divergent). It is a **footer-emission policy** (no transition-generation change); the default still fails
> closed (`ZIC001`); **CORE.1 is byte-unchanged**. Under the explicit replay modes, **every stable tzdata
> release (276/276) now replays with 0 zic-rs behaviour divergence** (the historical bands vs an admitted
> oracle). Beyond the last explicit transition is intentionally not claimed.

The atlas joins three evidence planes so a difference is **explainable by axis** — *upstream data / source
shape · reference-`zic` ecology · vendor-`zic` ecology · reader/runtime · drop-in environment · zic-rs · its
safer-divergence / deferred / bounded-legacy-source features* — instead of a scalar "fail." Machine-readable:
`reports/tzdb-atlas/atlas.tsv` (rebuild: `build-atlas.py`); receipt: `reports/tzdb-atlas/RECEIPT-TZDB-ATLAS-2.md`. Machine-readable exports (NDJSON + JSON-Schema + HTML, derived from the TSV — ATLAS-FORMAT.1): `reports/tzdb-atlas/README.md` (rebuild: `emit-formats.py`); receipt: `reports/tzdb-atlas/RECEIPT-ATLAS-FORMAT-1.md`
(with the original `RECEIPT-TZDB-ATLAS-1.md`). No new compiler work — pure join.

## The three axes (all now carrying real outcomes)

1. **Upstream archive + data** — RELEASE-ALL.1 (785 entries indexed, 571 archives, provenance) + RELEASE-ALL.DATA.1
   (**all 276 stable tzdata compiled**, behaviour-classified). `docs/iana-release-archive-ledger.md`.
2. **Vendor-oracle** — 19 real platform receipts: what each vendor's `zic`/`zdump` actually is (lineage ×
   version × shipped tzdb release × verdict). `../zic-rs-vendor-oracle-lab/`.
3. **Drop-in** — zic-rs built in real OS/build environments emitting the installed TZif tree. `reports/drop-in/`.
   (+ reader-compat: `reports/reader-compat/`.)

## The joined atlas

| axis | subject | key fact | verdict | attribution |
|---|---|---|---|---|
| `upstream_archive` | IANA release directory | 785 entries · 571 archives (276 tzdata · 246 tzcode · 48 bundles) · 214 sigs | **indexed** | `upstream_archive` |
| `upstream_archive` | signature provenance | 214/571 signature-backed (357 unsigned pre-signing-era); 55/55 signed-bundle+pilot GOODSIG | **verified-subset** | `upstream_archive` |
| `upstream_archive` | release-identity provenance (RELEASE-METADATA.1) | 276 releases; the authoritative tags (filename + `version` file) **never disagree** → **0 major** contradictions; 2 minor (NEWS-changelog only: 2019a upstream typo `Release 20198`; 2026b NEWS lags at 2026a); 211 legacy-unstructured (filename-only era). The atlas `release_id` is provenance-grounded | **multi-surface-consistent** | `upstream_archive` |
| `upstream_archive` | complete-bundle phase (RELEASE-ALL.TZDB.1) | **48/48** signed `tzdb-*.tar.lz` bundles (2016g→2026b) **verified_combined_release**: all GOODSIG (Eggert), all combined data+code, all identity-consistent (filename==`version`), all data **byte-identical to the standalone pair** → behaviour inherits DATA.1's `match`; **0** identity/data contradictions | **verified-combined-release** | `upstream_archive` |
| `upstream_data` | all stable tzdata (1993→2026b) | 276 releases. **Default UTF-8:** 150 match · 66 ref-build-incompat · 60 source-shape-incompat · 0 divergent. **With `--legacy-latin1`:** 210 match · 66 ref-build-incompat · **0 source-shape** · 0 divergent | **match-where-both-build** | `zic_rs` |
| `upstream_data` | behaviour divergence | 1050/1050 fixtures behaviour-match where both compilers built; ZERO zic-rs behaviour divergence across 30 years (+420/420 in the legacy-mode band) | **0-divergent** | `zic_rs` |
| `reference_zic` | yearistype breakpoint (≤2000e) — **66/66 CLOSED** | 66 releases use Rule `TYPE` even/odd; current zic removed `-y` (tzcode 2020a). **YEARISTYPE.1** `--legacy-yearistype` + **PERPETUAL-EXPANSION.1** `--legacy-empty-footer` → **all 66 build & match an admitted historical `zic` oracle** over `[1980,2037]` (371/371 fixtures). Historical-source replay, **not** current-reference parity | **closed-by-historical-replay** | `bounded_legacy_source` |
| `zic_rs` | perpetual year-parity footer (93b–94f) — **CLOSED** | **11** releases had **perpetual** even/odd rules (`1990 max even/odd`) → recurring tail not POSIX-footer-expressible. **CLOSED by PERPETUAL-EXPANSION.1**: `--legacy-empty-footer` emits the already-expanded explicit transitions (through `RECUR_HI`=2037) + an **empty footer**, matching the oracle's behaviour over `[1980,2037]`. Footer-emission policy only; default still fails closed; **CORE.1 byte-unchanged**; beyond-horizon freeze not claimed | **bounded-empty-footer-replay** | `bounded_legacy_source` |
| `source_shape` | source-profile parity (SOURCE-VARIANT.1) | zic-rs behaviour-matches reference `zic` on **all** major source profiles (2026b): `DATAFORM` main/vanguard/rearguard + backzone excluded/included — **8/8 fixtures each** (incl. the backzone-sensitive `America/Montreal`), **0 divergent / deferred / unsupported**. zic-rs normalises the 3 encodings to **byte-identical** output (not main-only — vanguard's negative-SAVE/`%z` AND rearguard's nonnegative-SAVE expansion both match) | **all-profiles-behaviour-match** | `zic_rs` |
| `source_shape` | UTF-8/Latin-1 breakpoint (2008a–2012j) — **CLOSED** | 60 releases were ISO-8859/Latin-1 pre-2013; **CLOSED by LEGACY-SOURCE.1** — `--legacy-latin1` admits Latin-1 **in comments only** (fields still fail closed) → all 60 → **match** (420/420 fixtures, 0 divergent, verified vs reference zic); default stays UTF-8-required (`ZIC012`) | **closed-by-legacy-source** | `bounded_legacy_source` |
| `drop_in` | installed-tree determinism | zic-rs bundle_hash 453641ff2568d8b1 byte-IDENTICAL across 15+ OS/build environments (host·container·VM·BSD·illumos·source-builds) | **byte-identical-tree** | `drop_in_environment` |
| `reader` | glibc-localtime | reads zic-rs output vs reference | **match** | `reader` |
| `reader` | go-time | reads zic-rs output vs reference | **match** | `reader` |
| `reader` | cctz-absl | reads zic-rs output vs reference | **match** | `reader` |
| `reader` | java/php/icu | reads zic-rs output vs reference | **unsupported** | `unsupported_by_design` |
| `vendor_oracle` | 19 platforms | real vendor zic/zdump: lineage (glibc/tzcode/fork) × tzdb release (2022g→2026b) × verdict | **admitted (15+ rows; 2 old-fork)** | `vendor_zic` |

## The headline (acceptance #7)

> **TZDB-ATLAS.4:** under explicit historical replay modes (`--legacy-latin1 --legacy-yearistype
> --legacy-empty-footer`), **all 276/276 stable tzdata releases build and behaviour-match** (150 modern + 60
> Latin-1 + 66 yearistype, the last two bands vs an admitted historical oracle), with **0 zic-rs behaviour
> divergences** across 30 years. No stable-release historical replay band remains; the only residual is the
> intentionally-unclaimed beyond-horizon freeze of the empty-footer files.

> Across **all stable tzdata releases attempted, the atlas records 0 zic-rs behaviour divergences** (1050/1050
> default + 420/420 Latin-1-band + 371/371 yearistype+empty-footer-band fixtures). The historical breakpoints
> partition cleanly by axis, and **both are now fully replayable**:
>
> - **2008–2012 Latin-1 band → `bounded_legacy_source`:** closed by `--legacy-latin1` (comments-only admission,
>   fields fail closed) → all 60 move to match. The default UTF-8 contract is unchanged.
> - **≤2000e `yearistype` band → `bounded_legacy_source` (66/66):** `--legacy-yearistype` (YEARISTYPE.1) admits
>   the historical Rule `TYPE` predicates (internal, no script execution) → 55/66; `--legacy-empty-footer`
>   (PERPETUAL-EXPANSION.1) closes the remaining 11 perpetual-year-parity releases via an explicit-expansion +
>   empty-footer fallback → **all 66/66 match an admitted historical `zic` oracle over `[1980,2037]`**. *Current*
>   reference `zic` still cannot build them (it removed `-y` in tzcode 2020a) — historical-source replay, not
>   current-reference parity; the beyond-horizon freeze is intentionally not claimed.

## The cross-axis join (the thing no single matrix answers)

**Every tzdb data release that real vendors actually ship — 2023c (OpenBSD) · 2025b (SUSE) · 2026a
(Gentoo/Ubuntu/NixOS/Alma) · 2026b (Alpine/Arch/Debian/host) — falls inside the upstream `match` band**
(RELEASE-ALL.DATA.1). None is in a `reference-build-incompatible` or `source-shape-incompatible` era. So the
data that production systems compile is exactly the data zic-rs behaviour-matches with 0 divergence.

## Questions the atlas answers directly

| question | answer (with axis) |
|---|---|
| Which releases build with **both** current `zic` and zic-rs? | **150** by default (2000f–2007 + 2013→2026b); **210** with `--legacy-latin1` (the 60 Latin-1 releases close) — `upstream_data` |
| Which release-era failures are **removed reference-`zic` support**? | **66** (≤2000e, `yearistype`) — **all 66 now replay** via `--legacy-yearistype` + `--legacy-empty-footer` (vs a historical oracle over [1980,2037]); no stable-release replay band remains — `bounded_legacy_source` |
| Which failures were **zic-rs source-shape policy**? | **60** (2008–2012, non-UTF-8/Latin-1) — **CLOSED** by `--legacy-latin1` (LEGACY-SOURCE.1) → all 60 → match (420/420 fixtures) — `bounded_legacy_source` |
| Which vendor rows show **old-fork** behaviour? | OpenBSD 7.9 · DragonFly 6.4 (can't ingest modern zishrink source) — `vendor_zic` |
| Which drop-in rows prove **installed-tree parity**? | all 15+ environments — `bundle_hash 453641ff…` byte-identical — `drop_in_environment` |
| Which reader rows are **limitations**, not mismatches? | Java/PHP/ICU (consume a compiled DB, not raw TZif); CCTZ (ignores leap) — `unsupported_by_design`/`reader` |
| **Where does zic-rs diverge when both build?** | **Nowhere** — 0 divergent across 30 years — `zic_rs` |

## Attribution taxonomy (every joined finding carries one)

`upstream_archive` · `source_shape` · `reference_zic` · `vendor_zic` · `reader` · `drop_in_environment` ·
`zic_rs` · `unsupported_by_design` · `bounded_legacy_source` (LEGACY-SOURCE.1) · `deferred`.

## Non-claims (acceptance #7)

- **Not** all historical civil-time data is claimed true (IANA/CLDR own that).
- **Not** all old historical `zic` behaviour is reproduced — the reference is *current* `zic` on old source;
  pre-2000f releases the *reference itself* cannot build.
- **No new evidence** — the atlas only *joins + attributes* the existing RELEASE-ALL / vendor-oracle /
  reader-compat / drop-in receipts. Each cell links to its source receipt.
- Bounded fixture set; `zdump` over 1900..2037; vendor/drop-in axes are bounded ecologies, not exhaustive.
