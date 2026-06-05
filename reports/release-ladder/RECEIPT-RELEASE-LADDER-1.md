# RELEASE-LADDER.1 — historical tzdb release parity ladder (2026-06-05)

> **Claim wording (binding):** *RELEASE-LADDER.1 does not claim all historical tzdb releases are
> supported. It attempts a bounded release ladder and records which releases match, which require
> source-profile handling, and which remain deferred or divergent.*

**Question answered:** does zic-rs hold across tzdb source *eras*, or only on the current 2026b source?

## Method (each release a real run)

For each release, `reports/release-ladder/gauntlet.sh`: fetches the official `tzdataYYYYx.tar.gz` +
`.asc` from `data.iana.org`, **hash-pins** it, **verifies the OpenPGP signature**, extracts, then
compiles the present region files (`africa antarctica asia australasia europe northamerica southamerica
etcetera backward [factory]`) with **both** reference `zic` (the host build — backward-compatible) and
`zic-rs` (`--unsupported skip`: compile the supported set, classify the rest), and compares per-fixture
`zdump` behaviour over 1900..2037. The reference oracle is *current `zic` on the old source*, so this
tests whether **zic-rs's parser+compiler reproduces reference behaviour on each era's source**.

Signature: every release is **GOODSIG** (Paul Eggert, the tz key) — fingerprint-anchored, recorded honestly.

## Fixture set (stable across releases)

`Etc/UTC` · `Europe/Lisbon` · `Europe/London` · `America/New_York` · `Asia/Singapore` ·
`Pacific/Kiritimati` · `Australia/Lord_Howe` — a spread of fixed / footer / many-transition /
non-hour-offset / +14 / sub-hour-DST zones, all present in every release tried.

## Verdict vocabulary

`match` (byte-identical) · `behaviour-match-byte-diff` (`zdump`-identical, bytes differ — zic-rs fat vs
reference slim) · `source-incompatible` (zic-rs cannot compile that zone's source form) ·
`unsupported-by-design` · `deferred` · `divergent` (a named mismatch finding) · `not-applicable`.

## The ladder

| release | era | archive sha256 | sig | ref `zic` files | zic-rs files | zic-rs skipped | fixtures | verdict |
|---|---|---|--|--|--|--|--|--|
| **2026b** | current baseline | `114543d9f19a6bfe…` | GOODSIG | 598 | 598 | 0 | 7/7 | **match** |
| **2026a** | adjacent prior | `77b541725937bb53…` | GOODSIG | 598 | 598 | 0 | 7/7 | **match** |
| **2025b** | recent | `11810413345fc780…` | GOODSIG | 598 | 598 | 0 | 7/7 | **match** |
| **2024a** | recent transition ctx | `0d0434459acbd205…` | GOODSIG | 597 | 597 | 0 | 7/7 | **match** |
| **2020a** | older modern | `547161eca24d344e…` | GOODSIG | 594 | 591 | 3 | 7/7 | **match** |
| **2018e** | pre-RFC8536-era ecology | `6b288e5926841a4c…` | GOODSIG | 592 | 589 | 3 | 7/7 | **match** |
| **2015g** | old-grammar/source-shape | `b923cdbf07849169…` | GOODSIG | 583 | 583 | 0 | 7/7 | **match** |

**Per-fixture tally (49 = 7 fixtures × 7 releases): 21 `match` (byte-identical) · 28
`behaviour-match-byte-diff` (`zdump`-identical) · 0 `divergent` · 0 `source-incompatible`.**
Every fixture, in every release, behaves identically to reference `zic` (`zdump` over 1900..2037).

## Named release facts (honest — recorded, not forced green)

- **Source-incompatible zones in 2018e & 2020a (3 each): `Asia/Tehran`, `Asia/Dushanbe`, `Iran`**
  (`Iran` is the `backward` link to `Asia/Tehran`). zic-rs skipped them (under `--unsupported skip`),
  reference `zic` compiled them — the file-count delta (594 vs 591 · 592 vs 589). Cause = **documented
  zic-rs deferred features** that those releases' source used: the inline **`+05/+06` slash STD/DST
  `FORMAT`** (Asia/Dushanbe) and a **recurring DST rule whose day form is not POSIX-footer-expressible**
  (Iran's pre-abolition rules). These are **not** in the fixture set and are **not** behaviour mismatches —
  they are source-form features zic-rs fails *closed* on (refuses rather than emitting an approximate
  footer). By 2024a+ the affected zones changed form (Iran abolished DST in 2022) and the skips vanish;
  in 2015g they used an older expressible form, so 0 skips. → classified `source-incompatible` per-zone,
  `deferred` as a zic-rs feature.
- **Zone-count drift is real tzdb history, not an error:** 583 (2015g) → 598 (2026b) reflects zones added
  over the decade; the fixture set was chosen to exist throughout.

## What this proves / does not

- **Proves:** zic-rs behaviour-matches reference `zic` on the fixture set across **7 releases spanning a
  decade** (current → adjacent → recent → older-modern → pre-RFC8536-era → old-grammar), with every
  archive signature-verified + hash-pinned. The compiler holds across source eras, not only on 2026b.
- **Does NOT claim:** all historical releases supported · all *zones* of those releases compile (3 are
  source-incompatible in two releases) · all releases ever made (7 of ~hundreds) · byte parity (slim/fat
  is a separate axis) · civil-time truth. The reference is *current `zic` on old source* (we hold no old
  `zic` binaries), so this is parser/compiler-era parity, not old-`zic`-binary parity.

## Reproduce

```sh
bash reports/release-ladder/gauntlet.sh   # fetches+verifies 7 tzdata archives; emits releases.tsv + fixtures.tsv
```

Raw results: `reports/release-ladder/{releases.tsv, fixtures.tsv}`. Archives cached under
`/tmp/release-ladder/` (re-fetchable + signature-verifiable from the pinned hashes above).
