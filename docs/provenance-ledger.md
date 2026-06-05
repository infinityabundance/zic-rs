# Provenance ledger — breadth-to-40 (T18.3)

> Joined in the **[TZDB Evidence Atlas](tzdb-evidence-atlas.md)** (TZDB-ATLAS.1).

> **T18.3 turns breadth-to-40 from a raw provenance archive into a readable trust ledger: 40
> receipt-bearing cases, 15 trap categories, source-file coverage, named findings, and explicit claim
> boundaries.**

This is the **reader-facing** view of the zone provenance archive built in **T18.breadth-to-40**. The raw
evidence (per-zone `zdump` witnesses, full hashes, the harness) lives in `reports/provenance/`
(`RECEIPT-2026-06-04-breadth-to-40.md`, `gauntlet.sh`, `prov.tsv`, `zdump/`); this doc is the legible
ledger over it.

## How to read this archive (what it claims, and what it does not)

The archive compiles **40 diversity-selected zones** from the **signature-verified, hash-pinned pristine
IANA tzdb 2026b** (`tzdb-2026b.tar.lz` sha256 `ffad46a0…`; OpenPGP VALIDSIG by the tz key
`7E37 92A9 D8AC F7D6 33BC 1588 ED97 E90E 62AA 7E34`, Paul Eggert) with **both** reference `zic` and
`zic-rs`, from the same region source files, and witnesses each with `zdump`.

- **It claims `compiler-equivalence`** — for every case, zic-rs reproduces reference `zic` *behaviour*
  (`zdump` agreement over 1900–2040; the slim byte-diffs were further checked zdump-identical over **all**
  time). **40 / 40 zdump-MATCH.**
- **It claims `source-provenance`** — each case is traceable to a named, hash-pinned region file in a
  signature-verified release.
- **It does NOT claim historical correctness.** Whether the tzdb *data* is historically/legally true is
  IANA's authority, never zic-rs's. zic-rs faithfully compiles whatever the admitted source says.
- **Reader-compatibility is cross-referenced, not re-proven here.** That a real TZif *reader* consumes the
  output equivalently is the separate `T23.reader-compat` gauntlet. **T23.reader-compat.3** widened it to a
  **cross-reader ecology**: 6 of these ledger zones × {glibc 2.43 · Go 1.26 · CCTZ/abseil} all interpret
  zic-rs output **identically to reference-`zic`** (16 match · 1 match-with-known-limitation · **0
  mismatch**), with Java/PHP/ICU classified `unsupported_by_reader` (they consume a pre-compiled DB, not raw
  TZif). See `reports/reader-compat/RECEIPT-T23-reader-compat-3.md`. This ledger does not re-establish reader
  compat for all 40.

So a row that reads "✅ zdump-MATCH" means *"zic-rs compiled this admitted-2026b zone to the same observable
behaviour as reference `zic`"* — not *"this is the historically correct timezone."*

### Compact ledger — 40 cases

| # | zone | cats | src file | zic-rs fat sha | behaviour vs ref `zic` | slim vs ref (structural) |
|--|--|--|--|--|--|--|
| 1 | `America/New_York` | 1,6 | northamerica | `8d0d951800a68a0c` | ✅ zdump-MATCH | null-diff |
| 2 | `Europe/London` | 1,3,10 | europe | `7c27432f530167b1` | ✅ zdump-MATCH | null-diff |
| 3 | `Europe/Dublin` | 2 | europe | `d88a38a08a280366` | ✅ zdump-MATCH | null-diff |
| 4 | `Europe/Prague` | 2 | europe | `03acf7dbcb02cd36` | ✅ zdump-MATCH | null-diff |
| 5 | `Africa/Windhoek` | 2,8 | africa | `c7217e38a376f33f` | ✅ zdump-MATCH | null-diff |
| 6 | `Europe/Paris` | 3,1 | europe | `a8224f7ecb0230a0` | ✅ zdump-MATCH | null-diff |
| 7 | `Asia/Kolkata` | 4,9 | asia | `3a00bdbe1bc4959e` | ✅ zdump-MATCH | byte-parity |
| 8 | `Asia/Kathmandu` | 4 | asia | `76b8f1bfe072231a` | ✅ zdump-MATCH | byte-parity |
| 9 | `Australia/Eucla` | 4 | australasia | `ec4ed99ed31c997e` | ✅ zdump-MATCH | null-diff |
| 10 | `Asia/Tehran` | 4,8 | asia | `0367914d7dc5f18d` | ✅ zdump-MATCH | null-diff |
| 11 | `Asia/Yangon` | 4 | asia | `e89d835c811d4da4` | ✅ zdump-MATCH | byte-parity |
| 12 | `Asia/Singapore` | 5,4 | asia | `0954b2d9a301d94f` | ✅ zdump-MATCH | byte-parity |
| 13 | `Africa/Monrovia` | 5,8 | africa | `58cf8955faf9d365` | ✅ zdump-MATCH | byte-parity |
| 14 | `Asia/Jerusalem` | 6,8 | asia | `80bdfb8106ca9b3c` | ✅ zdump-MATCH | null-diff |
| 15 | `America/Santiago` | 6,10 | southamerica | `32eb11df7c318333` | ✅ zdump-MATCH | byte-parity |
| 16 | `Pacific/Honolulu` | 7 | northamerica | `1daa5729aa1e0f32` | ✅ zdump-MATCH | byte-parity |
| 17 | `America/Phoenix` | 7 | northamerica | `f7393bf53a951939` | ✅ zdump-MATCH | null-diff |
| 18 | `Asia/Gaza` | 8,10,12 | asia | `85db6553f359e955` | ✅ zdump-MATCH | null-diff |
| 19 | `Africa/Casablanca` | 8,10,12 | africa | `9abed260f26526c1` | ✅ zdump-MATCH | null-diff |
| 20 | `Pacific/Fiji` | 8 | australasia | `c278d82d6bad06b4` | ✅ zdump-MATCH | null-diff |
| 21 | `Asia/Damascus` | 8 | asia | `9a40e29a780c8c3a` | ✅ zdump-MATCH | null-diff |
| 22 | `Europe/Volgograd` | 8 | europe | `bf73fa88527ead38` | ✅ zdump-MATCH | byte-parity |
| 23 | `Europe/Kyiv` | 9 | europe | `258b85c6c897ae70` | ✅ zdump-MATCH | null-diff |
| 24 | `America/Argentina/Buenos_Aires` | 9 | southamerica | `c911c2f508cbc597` | ✅ zdump-MATCH | null-diff |
| 25 | `Asia/Ho_Chi_Minh` | 9 | asia | `0d6e76fb941d0540` | ✅ zdump-MATCH | byte-parity |
| 26 | `Antarctica/Troll` | 10,12 | antarctica | `b38cf417fb8acf1d` | ✅ zdump-MATCH | byte-parity |
| 27 | `Pacific/Kiritimati` | 12 | australasia | `71454698c4418259` | ✅ zdump-MATCH | byte-parity |
| 28 | `Asia/Tokyo` | 7 | asia | `8dff15a1da88f850` | ✅ zdump-MATCH | null-diff |
| 29 | `Africa/Juba` | 14,8 | africa | `ad136c5e3a4d13e6` | ✅ zdump-MATCH | null-diff |
| 30 | `Pacific/Bougainville` | 14,15 | australasia | `aea767d58e0749aa` | ✅ zdump-MATCH | byte-parity |
| 31 | `America/Punta_Arenas` | 14 | southamerica | `d80aa1edbaa8fa64` | ✅ zdump-MATCH | byte-parity |
| 32 | `Asia/Atyrau` | 14,8 | asia | `d581b84332f13d16` | ✅ zdump-MATCH | byte-parity |
| 33 | `America/Vancouver` | 15,8 | northamerica | `76487b663cb1d144` | ✅ zdump-MATCH | null-diff |
| 34 | `Antarctica/Casey` | 8,14 | antarctica | `d6373e1408ef90a9` | ✅ zdump-MATCH | byte-parity |
| 35 | `Pacific/Apia` | 8 | australasia | `4ff469339fb20ac0` | ✅ zdump-MATCH | null-diff |
| 36 | `Asia/Pyongyang` | 8 | asia | `3710b975af284d9e` | ✅ zdump-MATCH | byte-parity |
| 37 | `Europe/Lisbon` | 10 | europe | `36bcae5e21df5f7f` | ✅ zdump-MATCH | null-diff **(finding: timecnt 141 vs 142)** |
| 38 | `America/Adak` | 1,10 | northamerica | `124b05ba38bea317` | ✅ zdump-MATCH | null-diff |
| 39 | `US/Eastern` | 9,11 | backward | `8d0d951800a68a0c` | ✅ zdump-MATCH | null-diff |
| 40 | `Etc/UTC` | 7,13 | etcetera | `fddce1e648a1732a` | ✅ zdump-MATCH | byte-parity |

### Category coverage matrix (15 trap categories → proving zones)

| # | trap category | zones proving it |
|--|--|--|
| 1 | pre-1970 weirdness | `America/New_York` · `Europe/London` · `Europe/Paris` · `America/Adak` |
| 2 | negative DST / winter time | `Europe/Dublin` · `Europe/Prague` · `Africa/Windhoek` |
| 3 | double summer time | `Europe/London` · `Europe/Paris` |
| 4 | non-hour offsets | `Asia/Kolkata` · `Asia/Kathmandu` · `Australia/Eucla` · `Asia/Tehran` · `Asia/Yangon` · `Asia/Singapore` |
| 5 | sub-minute / odd LMT | `Asia/Singapore` · `Africa/Monrovia` |
| 6 | many transitions | `America/New_York` · `Asia/Jerusalem` · `America/Santiago` |
| 7 | sparse transitions | `Pacific/Honolulu` · `America/Phoenix` · `Asia/Tokyo` · `Etc/UTC` |
| 8 | late political churn | `Africa/Windhoek` · `Asia/Tehran` · `Africa/Monrovia` · `Asia/Jerusalem` · `Asia/Gaza` · `Africa/Casablanca` · `Pacific/Fiji` · `Asia/Damascus` · `Europe/Volgograd` · `Africa/Juba` · `Asia/Atyrau` · `America/Vancouver` · `Antarctica/Casey` · `Pacific/Apia` · `Asia/Pyongyang` |
| 9 | renamed / linked | `Asia/Kolkata` · `Europe/Kyiv` · `America/Argentina/Buenos_Aires` · `Asia/Ho_Chi_Minh` · `US/Eastern` |
| 10 | unusual footer | `Europe/London` · `America/Santiago` · `Asia/Gaza` · `Africa/Casablanca` · `Antarctica/Troll` · `Europe/Lisbon` · `America/Adak` |
| 11 | backward / backzone | `US/Eastern` |
| 12 | reader-compat stress | `Asia/Gaza` · `Africa/Casablanca` · `Antarctica/Troll` · `Pacific/Kiritimati` |
| 13 | leap / right | `Etc/UTC` |
| 14 | underrepresented region | `Africa/Juba` · `Pacific/Bougainville` · `America/Punta_Arenas` · `Asia/Atyrau` · `Antarctica/Casey` |
| 15 | reference-vs-vendor divergence history | `Pacific/Bougainville` · `America/Vancouver` |

### Source-file coverage matrix

| source file | # zones | zones |
|--|--|--|
| africa | 4 | `Africa/Windhoek` · `Africa/Monrovia` · `Africa/Casablanca` · `Africa/Juba` |
| antarctica | 2 | `Antarctica/Troll` · `Antarctica/Casey` |
| asia | 12 | `Asia/Kolkata` · `Asia/Kathmandu` · `Asia/Tehran` · `Asia/Yangon` · `Asia/Singapore` · `Asia/Jerusalem` · `Asia/Gaza` · `Asia/Damascus` · `Asia/Ho_Chi_Minh` · `Asia/Tokyo` · `Asia/Atyrau` · `Asia/Pyongyang` |
| australasia | 5 | `Australia/Eucla` · `Pacific/Fiji` · `Pacific/Kiritimati` · `Pacific/Bougainville` · `Pacific/Apia` |
| etcetera | 1 | `Etc/UTC` |
| europe | 7 | `Europe/London` · `Europe/Dublin` · `Europe/Prague` · `Europe/Paris` · `Europe/Volgograd` · `Europe/Kyiv` · `Europe/Lisbon` |
| northamerica | 5 | `America/New_York` · `Pacific/Honolulu` · `America/Phoenix` · `America/Vancouver` · `America/Adak` |
| southamerica | 3 | `America/Santiago` · `America/Argentina/Buenos_Aires` · `America/Punta_Arenas` |
| backward | 1 | `US/Eastern` |

*(backzone: off by default — not in the build; `backward` is represented by `US/Eastern` (Link → America/New_York). factory: source admitted/pinned but no selected case.)*

_tally: behaviour 40/40 MATCH · slim byte-parity 17/40 · slim null-diff 23/40 (all zdump-identical over all time)._

## Named findings (acceptance: nothing hidden)

1. **Europe/Lisbon — the one structural count-difference (behaviourally null).** In `zic-slim` mode zic-rs
   emits **timecnt 141 vs reference 142** (version/typecnt/charcnt identical), yet the two are
   **zdump-identical over all time**. This is exactly the documented **T8 "Europe/Lisbon residual: a
   ref-fatter-by-1 no-op"** — reference keeps one extra footer-redundant transition at the slim boundary.
   Surfaced again on the region-file input, recorded openly; not a behaviour bug.
2. **The other 22 slim byte-diffs are behaviourally null with *identical* counts** — equivalent internal
   type/abbreviation ordering produced when compiling region files directly (zic-rs reaches near-total slim
   byte-parity on the canonical zishrunk `tzdata.zi`, T8). Behaviour (`zdump`) is the contract; byte-layout
   in slim mode is a separate, weaker axis.
3. **`Europe/Amsterdam` was rejected as a pick — and why.** It was first chosen for category 5 (sub-minute
   LMT +0:19:32) but is a **`Link → Europe/Brussels`** in modern tzdb (carries Brussels's +0:17:30 LMT,
   not Amsterdam's). Admitting it under a "sub-minute LMT" rationale would have been mislabelled, so it was
   dropped — a case is *not* admitted just because it exists.
4. **`Asia/Singapore` is the replacement** — a genuine `Zone` (not a link) with famously odd historical
   offsets (LMT **+6:55:25** → **+7:20** → **+7:30** → **+8:00**), a stronger and honest category-5/4 case.

No behaviour mismatch was found across the 40, and nothing was excluded to reach the count.

## Claim boundaries (per the four-claim discipline)

| claim type | status in this ledger |
|---|---|
| source-provenance | **claimed** — pinned, signature-verified pristine 2026b region files |
| compiler-equivalence | **claimed** — 40/40 `zdump`-MATCH vs reference `zic` |
| reader-compatibility | **cross-referenced** — `T23.reader-compat.1/.2/.3` (glibc · Go · CCTZ all see zic-rs ≡ ref; Java/PHP/ICU consume a compiled DB → classified), not re-proven for all 40 here |
| historical-correctness | **NOT claimed** — IANA/tzdb authority |

Further bounds: byte parity is claimed only in `zic-slim` mode where it holds (17/40 on region-file input;
the rest behaviourally null); 40 stress-selected cases over `1900,2040` are **not** all-598 coverage
(CORE.1 covers all 341 canonical zones separately); `backzone` is off by default (case 39 `US/Eastern`
exercises a `backward` Link identity, not backzone supersession).

## Provenance-set bundle hash

sha256 over the 40 zic-rs *fat* outputs (sorted `relpath\0content-hash`):
`c489cbb68c1a92c9c94eb241b197fd910505fd896d2a8f2a788626e1660a517f`. **80 per-zone `zdump` witnesses**
(`ref` + `zrs` for each) are stored under `reports/provenance/zdump/`.

## Reproduce

`bash reports/provenance/gauntlet.sh` (needs reference `zic`/`zdump` 2026b + the pristine 2026b source at
`/tmp/prov/src/tzdb-2026b`, re-fetchable + signature-verifiable from the pins in
`reports/provenance/source-files.sha256` and the receipt). CORE.1 is unaffected by this archive (no `src/`
change): sweep stays **341/0/0**.
