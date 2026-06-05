# Historical tzdb release ladder (RELEASE-LADDER.1)

> *RELEASE-LADDER.1 does not claim all historical tzdb releases are supported. It attempts a bounded
> release ladder and records which releases match, which require source-profile handling, and which
> remain deferred or divergent.*

**Does zic-rs hold across tzdb source eras, or only on 2026b?** This ladder feeds each historical
release's source to **both** reference `zic` and `zic-rs` and compares `zdump` behaviour on a stable
fixture set. The full evidence is `reports/release-ladder/RECEIPT-RELEASE-LADDER-1.md` (+ the harness
`gauntlet.sh` and `releases.tsv`/`fixtures.tsv`); this is the summary. **Scaled to the whole archive:** `docs/iana-release-archive-ledger.md` (RELEASE-ALL.1) indexes all 785 IANA release entries.

## Result (7 releases, each signature-verified + hash-pinned, GOODSIG)

| release | era | verdict | note |
|---|---|---|---|
| 2026b | current baseline | **match** | 7/7 fixtures |
| 2026a | adjacent prior | **match** | 7/7 |
| 2025b | recent | **match** | 7/7 |
| 2024a | recent transition ctx | **match** | 7/7 |
| 2020a | older modern | **match** | 7/7 fixtures; 3 non-fixture zones source-incompatible |
| 2018e | pre-RFC8536-era ecology | **match** | 7/7 fixtures; 3 non-fixture zones source-incompatible |
| 2015g | old-grammar / source-shape | **match** | 7/7 |

**Per-fixture: 49/49 behaviour-match** (21 byte-identical · 28 `zdump`-identical with the documented
slim/fat byte-diff) · **0 divergent.** zic-rs behaviour-matches reference `zic` across a decade of source.

## Named release facts (recorded, not forced green)

- **`Asia/Tehran`, `Asia/Dushanbe`, `Iran` are `source-incompatible` in 2018e & 2020a** — those releases'
  source used the inline `+05/+06` slash STD/DST `FORMAT` and a non-POSIX-footer-expressible recurring
  Iran DST rule, both **documented zic-rs deferred features** that it fails *closed* on. Not in the fixture
  set, not behaviour mismatches. By 2024a+ the zones changed form (Iran abolished DST in 2022) and the skips
  vanish; 2015g used an older expressible form (0 skips).
- Zone-count drift 583 (2015g) → 598 (2026b) is real tzdb history, not an error.

## Boundaries

Reference = **current `zic` on old source** (no old `zic` binaries held) → this is parser/compiler-era
parity, not old-binary parity. 7 of hundreds of releases; fixture set, not all zones; not byte parity;
not civil-time truth. See the receipt's "What this proves / does not".
