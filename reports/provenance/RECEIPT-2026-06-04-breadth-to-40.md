# T18.breadth-to-40 — provenance archive receipt (2026-06-04)

> **Headline: provenance breadth expanded to 40 receipt-bearing cases, selected for historical,
> structural, regional, and reader-compatibility stress rather than convenience.**
>
> **Receipt wording (binding):** *T18 breadth-to-40 expands the provenance archive from a narrow
> correctness surface into a broader, less curated historical and ecological witness set. The receipt
> does not claim timezone truth; it claims that zic-rs preserves the admitted source semantics against
> the declared reference surfaces for the selected cases.*

## What was admitted (source provenance)

- **Release:** IANA **tzdb 2026b**, the **pristine** complete distribution `tzdb-2026b.tar.lz`,
  sha256 `ffad46a04c8d1624197056630af475a35f3556d0887f028ac1bd33b7d47dc653` — **identical to the
  T12.5a.2 admission**.
- **Signature:** **GOODSIG + VALIDSIG** by the published tz key `7E37 92A9 D8AC F7D6 33BC 1588 ED97
  E90E 62AA 7E34` ("Paul Eggert") — fingerprint-anchored trust (not web-of-trust; recorded honestly).
- **Source-file pins** (the actual region files each zone is compiled from — `reports/provenance/source-files.sha256`):

  | file | sha256 |
  |---|---|
  | africa | `c19940072a9e79d57ad844fc9f676f2067e5fada6708f3bf9a1cd4de34c8eeb7` |
  | antarctica | `e410ad71c9450828c592d21419301d41ac79ce50159fd0ac2d6c5031cb6bdfe6` |
  | asia | `cd12fe2bd64a02d808fd34abb92f08f19e5da20133a1c6c347d11171c00d9e1c` |
  | australasia | `e60bee81387d105dc2a31eef7defd0d0061322abd65c7a21c3bbcbb96e7c2d6b` |
  | europe | `b9c98254bed0773de5b523837cf996f3e88c93258d9c458ce51e69f77929a6c8` |
  | northamerica | `30bdcadf734a87b7bfc8a70fa9a76effe149d002da563b945a754f88a2791c57` |
  | southamerica | `c6e17ee367c6d7c1ab613444641538bca6d11818d46feb96e565e98c45238a2b` |
  | etcetera | `7281f095b42c13c4ae36b8bcba884e81dbb38127221fc1d9805c4dbf852487db` |
  | backward | `d2f4c8953f204982ddf4dc0c2debf41b2464de376dad7d546d0fc70f889fa706` |
  | factory | `ae2ec1d36dabf79a69cb7dd4fb6fd9168d05fc8cfd31aee2dd19e4f18beb9885` |

- **Build:** both compilers fed the **same region files** (the authentic tzdb build path), 598 outputs each:
  reference `zic (tzcode) 2026b` (slim default) and `zic-rs 0.1.0` (fat default + a `--emit-style zic-slim`
  pass). Harness + raw zdump witnesses: `reports/provenance/gauntlet.sh`, `prov.tsv`, `zdump/`.

## The four claim types — kept distinct (acceptance #4)

| claim | what this archive asserts | evidence column |
|---|---|---|
| **source-provenance** | each case comes from the signature-verified, hash-pinned pristine 2026b region file named | `src file` + the pins above |
| **compiler-equivalence** | zic-rs reproduces reference `zic` **behaviour** for the case | `behaviour (zdump 1900–2040)` → **40/40 MATCH** |
| **reader-compatibility** | real TZif **readers** consume the output equivalently | **cross-referenced**, not re-claimed here — see `T23.reader-compat.1/.2` (27 microcases × `zdump`/Python `zoneinfo`) |
| **historical-correctness** | the civil-time data is historically true | **NOT claimed** — that is IANA/tzdb authority, never zic-rs's |

The archive's load-bearing claim is **compiler-equivalence + source-provenance**. Historical-correctness is
explicitly out of scope; reader-compatibility is a separate, narrower gauntlet, cross-linked not duplicated.

## The 40 cases (each receipt-bearing — acceptance #1, #2)

Categories: 1 pre-1970 weirdness · 2 negative DST/winter time · 3 double summer time · 4 non-hour offsets ·
5 sub-minute/odd LMT · 6 many transitions · 7 sparse transitions · 8 late political churn · 9 renamed/linked ·
10 unusual footer · 11 backward/backzone · 12 reader-compat stress · 13 leap/right · 14 underrepresented
region · 15 reference-vs-vendor divergence history. (sha columns are the first 16 hex of the per-file sha256;
full per-zone hashes regenerate from `gauntlet.sh`.)

| # | zone | src file | ref `zic` sha | zic-rs fat sha | zic-rs slim sha | behaviour (zdump 1900–2040) | slim vs ref | cats | selected because |
|--|--|--|--|--|--|--|--|--|--|
| 1 | `America/New_York` | northamerica | `d7f2206b3a45989f` | `8d0d951800a68a0c` | `173e3f4292611578` | ✅ MATCH | ≠ (null) | 1,6 | pre-1883 LMT + many transitions + ~2^31 boundary |
| 2 | `Europe/London` | europe | `676541f0b8ad457c` | `7c27432f530167b1` | `7c27432f530167b1` | ✅ MATCH | ≠ (null) | 1,3,10 | pre-1847 LMT + WW2 double summer time + far-future footer |
| 3 | `Europe/Dublin` | europe | `11c00336e02f1318` | `d88a38a08a280366` | `59953cc9b6d2b3aa` | ✅ MATCH | ≠ (null) | 2 | negative DST (winter-time IST/GMT inversion) |
| 4 | `Europe/Prague` | europe | `a6e930e3375cdcb5` | `03acf7dbcb02cd36` | `734c7a64a212fcc0` | ✅ MATCH | ≠ (null) | 2 | inline negative SAVE (1 -1 GMT, law 7) |
| 5 | `Africa/Windhoek` | africa | `8358cb464a3fda97` | `c7217e38a376f33f` | `c7217e38a376f33f` | ✅ MATCH | ≠ (null) | 2,8 | Namibia winter-time / DST-sign churn |
| 6 | `Europe/Paris` | europe | `cd588e779c5737d7` | `a8224f7ecb0230a0` | `e14ef3778a68b081` | ✅ MATCH | ≠ (null) | 3,1 | 1940s double summer time under occupation |
| 7 | `Asia/Kolkata` | asia | `3a00bdbe1bc4959e` | `3a00bdbe1bc4959e` | `3a00bdbe1bc4959e` | ✅ MATCH | byte-match | 4,9 | +5:30 non-hour offset + renamed (Calcutta) |
| 8 | `Asia/Kathmandu` | asia | `76b8f1bfe072231a` | `76b8f1bfe072231a` | `76b8f1bfe072231a` | ✅ MATCH | byte-match | 4 | +5:45 sub-hour offset |
| 9 | `Australia/Eucla` | australasia | `dcdaac15f33347af` | `ec4ed99ed31c997e` | `ec4ed99ed31c997e` | ✅ MATCH | ≠ (null) | 4 | +8:45 non-hour offset + DST |
| 10 | `Asia/Tehran` | asia | `65ac5ec01f3721d6` | `0367914d7dc5f18d` | `0367914d7dc5f18d` | ✅ MATCH | ≠ (null) | 4,8 | +3:30 offset + Iran DST (now abolished) |
| 11 | `Asia/Yangon` | asia | `e89d835c811d4da4` | `e89d835c811d4da4` | `e89d835c811d4da4` | ✅ MATCH | byte-match | 4 | +6:30 offset |
| 12 | `Asia/Singapore` | asia | `0954b2d9a301d94f` | `0954b2d9a301d94f` | `0954b2d9a301d94f` | ✅ MATCH | byte-match | 5,4 | odd historical offsets: LMT +6:55:25 -> +7:20 -> +7:30 -> +8:00 |
| 13 | `Africa/Monrovia` | africa | `58cf8955faf9d365` | `58cf8955faf9d365` | `58cf8955faf9d365` | ✅ MATCH | byte-match | 5,8 | Liberia -0:44:30 until 1972 (odd sub-minute, late switch) |
| 14 | `Asia/Jerusalem` | asia | `9fcde8d584dea058` | `80bdfb8106ca9b3c` | `6a1c9387d4147ef7` | ✅ MATCH | ≠ (null) | 6,8 | many transitions, complex modern DST |
| 15 | `America/Santiago` | southamerica | `fd006953c2b442a2` | `32eb11df7c318333` | `fd006953c2b442a2` | ✅ MATCH | byte-match | 6,10 | southern-hemisphere DST + v3 footer (dayshift) |
| 16 | `Pacific/Honolulu` | northamerica | `1daa5729aa1e0f32` | `1daa5729aa1e0f32` | `1daa5729aa1e0f32` | ✅ MATCH | byte-match | 7 | sparse transitions, no modern DST |
| 17 | `America/Phoenix` | northamerica | `ae11453c21d08984` | `f7393bf53a951939` | `f7393bf53a951939` | ✅ MATCH | ≠ (null) | 7 | Arizona no-DST, sparse |
| 18 | `Asia/Gaza` | asia | `f8f0bffe018e0da0` | `85db6553f359e955` | `c9bf852b239f5e52` | ✅ MATCH | ≠ (null) | 8,10,12 | Ramadan rules, v3 footer, late churn (law 10) |
| 19 | `Africa/Casablanca` | africa | `30ca6cf13e00c2a6` | `9abed260f26526c1` | `9abed260f26526c1` | ✅ MATCH | ≠ (null) | 8,10,12 | Morocco Ramadan DST, unusual footer, late churn |
| 20 | `Pacific/Fiji` | australasia | `ba608d86d4ee0738` | `c278d82d6bad06b4` | `c278d82d6bad06b4` | ✅ MATCH | ≠ (null) | 8 | late political DST on/off churn |
| 21 | `Asia/Damascus` | asia | `02d6530d1cc7101e` | `9a40e29a780c8c3a` | `9a40e29a780c8c3a` | ✅ MATCH | ≠ (null) | 8 | Syria DST churn (DST dropped 2022) |
| 22 | `Europe/Volgograd` | europe | `bf73fa88527ead38` | `bf73fa88527ead38` | `bf73fa88527ead38` | ✅ MATCH | byte-match | 8 | Russia MSK offset churn (recent) |
| 23 | `Europe/Kyiv` | europe | `0589e80ddecebf9d` | `258b85c6c897ae70` | `258b85c6c897ae70` | ✅ MATCH | ≠ (null) | 9 | renamed (Kiev), political |
| 24 | `America/Argentina/Buenos_Aires` | southamerica | `20454ea527c8ea88` | `c911c2f508cbc597` | `c911c2f508cbc597` | ✅ MATCH | ≠ (null) | 9 | sub-zoned/renamed, provincial churn |
| 25 | `Asia/Ho_Chi_Minh` | asia | `0d6e76fb941d0540` | `0d6e76fb941d0540` | `0d6e76fb941d0540` | ✅ MATCH | byte-match | 9 | renamed (Saigon) |
| 26 | `Antarctica/Troll` | antarctica | `b38cf417fb8acf1d` | `b38cf417fb8acf1d` | `b38cf417fb8acf1d` | ✅ MATCH | byte-match | 10,12 | UTC<->+02 (CEST) — unusual footer / reader stress |
| 27 | `Pacific/Kiritimati` | australasia | `71454698c4418259` | `71454698c4418259` | `71454698c4418259` | ✅ MATCH | byte-match | 12 | +14 maximum offset (reader stress) |
| 28 | `Asia/Tokyo` | asia | `59a3871430f0d3b9` | `8dff15a1da88f850` | `8dff15a1da88f850` | ✅ MATCH | ≠ (null) | 7 | stable control zone (well-behaved) |
| 29 | `Africa/Juba` | africa | `553a683003fe8c9e` | `ad136c5e3a4d13e6` | `ad136c5e3a4d13e6` | ✅ MATCH | ≠ (null) | 14,8 | South Sudan 2021 offset change (underrepresented) |
| 30 | `Pacific/Bougainville` | australasia | `aea767d58e0749aa` | `aea767d58e0749aa` | `aea767d58e0749aa` | ✅ MATCH | byte-match | 14,15 | 2014 +10->+11 change (underrepresented) |
| 31 | `America/Punta_Arenas` | southamerica | `d80aa1edbaa8fa64` | `d80aa1edbaa8fa64` | `d80aa1edbaa8fa64` | ✅ MATCH | byte-match | 14 | split from Santiago 2017 (underrepresented) |
| 32 | `Asia/Atyrau` | asia | `d581b84332f13d16` | `d581b84332f13d16` | `d581b84332f13d16` | ✅ MATCH | byte-match | 14,8 | Kazakhstan offset change (underrepresented) |
| 33 | `America/Vancouver` | northamerica | `7d23681b36849ac7` | `76487b663cb1d144` | `76487b663cb1d144` | ✅ MATCH | ≠ (null) | 15,8 | the real 2026a->2026b release-diff zone |
| 34 | `Antarctica/Casey` | antarctica | `d6373e1408ef90a9` | `d6373e1408ef90a9` | `d6373e1408ef90a9` | ✅ MATCH | byte-match | 8,14 | flips +08/+11 (churn, underrepresented) |
| 35 | `Pacific/Apia` | australasia | `dc70c47c80ab2c87` | `4ff469339fb20ac0` | `4ff469339fb20ac0` | ✅ MATCH | ≠ (null) | 8 | Samoa 2011 dateline skip (lost a calendar day) |
| 36 | `Asia/Pyongyang` | asia | `3710b975af284d9e` | `3710b975af284d9e` | `3710b975af284d9e` | ✅ MATCH | byte-match | 8 | 2015 +8:30 then 2018 back to +9 |
| 37 | `Europe/Lisbon` | europe | `44d2f6cf84737e6a` | `36bcae5e21df5f7f` | `36bcae5e21df5f7f` | ✅ MATCH | ≠ (null) **[finding]** | 10 | slim/fat timecnt residual zone (structural edge) |
| 38 | `America/Adak` | northamerica | `abfb1980e20d5f84` | `124b05ba38bea317` | `a932f71ba5736bdd` | ✅ MATCH | ≠ (null) | 1,10 | abbreviation suffix-sharing (HST in AHST), Aleutian |
| 39 | `US/Eastern` | backward | `d7f2206b3a45989f` | `8d0d951800a68a0c` | `173e3f4292611578` | ✅ MATCH | ≠ (null) | 9,11 | backward Link -> America/New_York (linked identity) |
| 40 | `Etc/UTC` | etcetera | `fddce1e648a1732a` | `fddce1e648a1732a` | `fddce1e648a1732a` | ✅ MATCH | byte-match | 7,13 | fixed sparse; + right/ leap profile witness (see reader-compat.2) |

**Provenance-set bundle hash** (sha256 over the 40 zic-rs *fat* outputs, sorted `relpath\0content-hash`):
`c489cbb68c1a92c9c94eb241b197fd910505fd896d2a8f2a788626e1660a517f`.

## Results

- **Behaviour (zdump 1900–2040): 40 / 40 MATCH · 0 findings.** zic-rs reproduces reference `zic`
  behaviour for every selected case (compiler-equivalence). For the 23 zones whose *slim bytes* differ,
  this was further verified **zdump-identical over all time** (not just the horizon) — see the finding below.
- **Structural (zic-rs `zic-slim` bytes vs reference slim bytes): 17 byte-identical · 23 differ.**

## Named finding (acceptance #6 — not a hidden exclusion)

The 23 slim byte-differences are **all 23 zdump-identical over all time** — i.e. behaviourally null. Of
these, **22 have byte-identical TZif counts** (version · timecnt · typecnt · charcnt · leapcnt); the byte
difference is *equivalent internal type/abbreviation ordering* produced when compiling the **region source
files directly** (vs the canonical zishrunk `tzdata.zi` against which zic-rs achieves near-total slim
byte-parity — T8). This is the documented structural-vs-behaviour axis distinction, not a regression.

**The one count-difference, named:**

- **Europe/Lisbon** — zic-rs `zic-slim` emits **timecnt 141** vs reference **142** (typecnt/charcnt/version
  identical), and is **zdump-identical over all time**. This is exactly the **T8 "Europe/Lisbon residual:
  a ref-fatter-by-1 no-op"** — reference keeps one extra footer-redundant transition at the slim boundary;
  behaviourally null. Surfaced again here on the region-file input, recorded openly.

No behaviour mismatch was found, and nothing was excluded to reach the 40.

## Gate

- **CORE.1 byte-stable (acceptance #5):** sweep **341 / 0 / 0** — this campaign added a harness + this
  receipt; **no `src/` change**, so default output is byte-unchanged.
- Reproduce: `bash reports/provenance/gauntlet.sh` (needs reference `zic`/`zdump` 2026b + the pristine
  2026b source at `/tmp/prov/src/tzdb-2026b`, re-fetchable + sig-verifiable by the pins above).

## Non-claims

- **Not timezone truth.** The archive witnesses that zic-rs preserves the *admitted source semantics*
  against reference `zic`/`zdump`; it does not assert the tzdb data is historically/legally correct.
- **Not all-time-for-all-zones byte parity.** Byte parity is claimed only in `zic-slim` mode where it
  holds (17/40 here on region-file input); the rest are behaviourally-null encoding-layout differences.
- **Not reader-universal.** Reader compatibility is the separate `T23.reader-compat` gauntlet (two real
  readers, bounded instants); this archive does not re-claim it for all 40.
- **Not exhaustive.** 40 diversity-selected cases over the `1900,2040` horizon — chosen for stress, not
  coverage of all 598 identifiers (CORE.1 covers all 341 canonical zones separately).
- **backzone:** off by default; case 39 (`US/Eastern`) exercises a `backward` **Link** identity. True
  `backzone` pre-1970 supersession is not in the default build and is not claimed here.
