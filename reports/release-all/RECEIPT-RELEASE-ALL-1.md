# RELEASE-ALL.1 receipt — complete IANA release archive index (2026-06-05)

> *RELEASE-ALL does not claim all historical civil-time data is true. It indexes and classifies the
> complete IANA release archive and records which release artifacts are admitted, signature-backed,
> source-compatible, compile-compatible, behaviour-matching, source-incompatible, unsupported, deferred,
> or divergent.* **Phase 1 = admit + classify the whole universe (no compiling).**

## What ran

- Scraped `data.iana.org/time-zones/releases/` → **785 file entries**, each classified, with release ID +
  size/date + signature availability + URL. Reproducible: `build-index.py` → `index.tsv`.
- **Signature-verified** the modern provenance backbone: `verify.sh` → `verified.tsv`.

## Result

- **Complete index: 785 entries** (1996l → 2026b, 256 release IDs): 276 `tzdata` (+1 beta) · 246 `tzcode` ·
  48 `tzdb` complete bundles · 214 signatures → **571 archives**.
- **Signature availability (571 archives):** 210 `.asc` · 4 legacy `.sign` · 357 unsigned (pre-signing era).
- **Verification pass: 55 / 55 GOODSIG, 0 failures** — all 48 signed complete `tzdb-*.tar.lz` bundles +
  the 7 RELEASE-LADDER.1 pilot tzdata releases (Paul Eggert, the tz key), hash-recorded.

## Findings / honesty

- **357 / 571 archives are unsigned** — not a defect, a recorded provenance fact (signing post-dates the
  oldest archives). Distinguishing `asc_available` / `sign_available_legacy` / `no_signature_available` is
  the point; the 4 legacy `.sign` entries are flagged as such.
- Two harness bugs were found+fixed before sealing: a malformed-URL/word-split issue (the tool shell is
  fish) and an `IFS=$'\t' read` empty-field collapse (tab is whitespace-IFS) that mis-aligned the signed-
  bundle column — both corrected; the 48/48 verification is real.

## Non-claims

- Not all 571 archives byte-hashed (a bounded high-value subset is); full hashing is the `.DATA`/`.CODE`
  phases' incremental work. Not a compile/behaviour claim (that is `RELEASE-ALL.DATA.1`). Not civil-time
  truth. The index is *availability + classification + a verified provenance subset*, not a global verdict.

## Forward

Axis 1 of the **TZDB Evidence Atlas** (`docs/iana-release-archive-ledger.md` §"Where this is going"):
upstream-archive × vendor-oracle × drop-in. Next: `RELEASE-ALL.DATA.1` (compile all stable tzdata — the
RELEASE-LADDER.1 harness scales), then `TZDB-ATLAS.1` (the three-axis join).

Reproduce: `curl -sSL https://data.iana.org/time-zones/releases/ -o /tmp/iana-index.html && python3
reports/release-all/build-index.py && bash reports/release-all/verify.sh`.
