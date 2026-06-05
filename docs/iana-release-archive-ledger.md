# IANA release archive ledger (RELEASE-ALL.1)

> *RELEASE-ALL does not claim all historical civil-time data is true. It indexes and classifies the
> complete IANA release archive and records which release artifacts are admitted, signature-backed,
> source-compatible, compile-compatible, behaviour-matching, source-incompatible, unsupported, deferred,
> or divergent.*

**Phase 1 = admit and classify the whole universe** (no compiling). The complete machine-readable index is
`reports/release-all/index.tsv`; the signature-verification pass is `reports/release-all/verified.tsv`;
rebuild with `reports/release-all/build-index.py` + `reports/release-all/verify.sh`.

## The complete index — every entry in `data.iana.org/time-zones/releases/`

**785 file entries** (the parent-dir/logo boilerplate excluded), spanning release IDs **1996l → 2026b**
(256 distinct IDs over ~30 years), classified:

| type | count | what it is |
|---|--:|---|
| `tzdata` | 276 | data-only release archives (`tzdataYYYYx.tar.gz`) |
| `tzdata_beta` | 1 | `tzdatabeta.tar.gz` |
| `tzcode` | 246 | code-only archives (incl. old `.tar.Z`, `tz32code`/`tz64code`) |
| `tzdb_complete_bundle` | 48 | complete `tzdb-*.tar.lz` bundles (2016g → 2026b) |
| `signature` | 214 | `.asc` (210) + legacy `.sign` (4) |
| **archives (non-signature)** | **571** | = 277 tzdata + 246 tzcode + 48 bundles |

## Signature coverage (provenance, across the 571 archives)

| signature availability | count | meaning |
|---|--:|---|
| `asc_available` | 210 | a modern OpenPGP `.asc` detached signature exists |
| `sign_available_legacy` | 4 | only the older `.sign` format exists |
| `no_signature_available` | 357 | unsigned (mostly pre-signing-era archives) |

So **214 / 571 archives are signature-backed**; the 357 unsigned are the older era (signing was not always
practised). This is recorded as *availability*, not a verdict — the actual cryptographic verification is the
pass below.

## Signature verification (Phase-1 stratified pass) — `verified.tsv`

- **All 48 signed complete `tzdb-*.tar.lz` bundles: GOODSIG** (Paul Eggert, the tz key) — the modern
  complete-distribution provenance backbone, hash-recorded + verified. **Opened + classified
  (RELEASE-ALL.TZDB.1):** all **48/48 `verified_combined_release`** — each is a combined data+code artifact,
  internally identity-consistent (filename == internal `version` file), with its data **byte-identical to the
  standalone tzdata pair** → behaviour inherits RELEASE-ALL.DATA.1's `match`; 0 identity/data contradictions
  (`reports/release-all/RECEIPT-RELEASE-ALL-TZDB-1.md`; decoded via the pinned `unlzip.py`).
- **The 7 RELEASE-LADDER.1 pilot `tzdata` releases (2015g–2026b): GOODSIG** (hash-recorded).
- **55 / 55 verified GOODSIG, 0 failures.** Full per-archive byte-hash + verify of the remaining signed
  tzdata/tzcode is the incremental work of the compile phases (`.DATA` / `.CODE`), not re-done here.

## What this is / is not

- **Is:** a complete, classified, machine-readable map of the entire IANA release archive — every artifact
  by type, release ID, signature availability, and URL — plus a verified provenance pass over the modern
  complete bundles. *We mapped the entire IANA release archive.*
- **Is not:** a claim that all releases compile / behaviour-match (that is `.DATA` / `.TZDB`), nor that all
  data is civil-time-true, nor a full byte-hash of all 571 (a bounded high-value subset is hashed/verified).

## Behaviour outcomes — RELEASE-ALL.DATA.1 (all 276 stable tzdata compiled)

Every stable `tzdata` archive (1993→2026b) compiled with **both** reference `zic` and `zic-rs`, per-fixture
`zdump` compared. Receipt: `reports/release-all/RECEIPT-RELEASE-ALL-DATA-1.md`; rows: `data-gauntlet.tsv`.

| outcome | count | era | meaning |
|---|--:|---|---|
| **match** | 150 | 2000f–2007 + 2013→2026b | both build; fixtures behaviour-match (1050/1050, **0 mismatch**) |
| **reference-build-incompatible** | 66→**0** | 1993→2000e | current `zic` can't build it (`yearistype` removed in tzcode 2020a); **ALL 66 CLOSED** — `--legacy-yearistype` (YEARISTYPE.1, 55) + `--legacy-empty-footer` (PERPETUAL-EXPANSION.1, the 11 perpetual-year-parity ones) → build & match a historical `zic` oracle over [1980,2037] (371/371 fixtures) |
| **source-shape-incompatible** | 60→**0** | 2008a–2012j | non-UTF-8 (Latin-1); **CLOSED by LEGACY-SOURCE.1** `--legacy-latin1` → all 60 move to match |
| **zic-rs-divergent** | 0 | — | no behaviour mismatch where both built |

**Two named breakpoints, both now fully replayed:** the `yearistype` reference-build breakpoint (~2000f — a
*reference* limit; **all 66/66** replayed via `--legacy-yearistype` + `--legacy-empty-footer` vs a historical
oracle over [1980,2037], YEARISTYPE.1 + PERPETUAL-EXPANSION.1) and the UTF-8/Latin-1 encoding breakpoint
(~2013 — a zic-rs admissibility decision; CLOSED via `--legacy-latin1`, LEGACY-SOURCE.1). **0 behaviour
divergence** across 30 years where both compilers built the fixtures; **under the explicit replay modes,
276/276 releases attempted with 0 divergence**.

## Where this is going — the TZDB Evidence Atlas

**The join is now built — see [`docs/tzdb-evidence-atlas.md`](tzdb-evidence-atlas.md) (TZDB-ATLAS.1).** This
index is **axis 1 of that three-axis evidence atlas**, the novelty being the *join*:

1. **Upstream archive** (this ledger) — what IANA released, when, in what form, with what signature/hash.
2. **Vendor-oracle matrix** (`../zic-rs-vendor-oracle-lab/` — 17 ecology rows) — what real vendor
   `zic`/`zdump` implementations did with fixtures.
3. **Drop-in receipt matrix** (`reports/drop-in/`) — whether zic-rs builds the same installed TZif tree in
   real OS environments.

**Forward phases:** `RELEASE-ALL.DATA.1` (compile all stable tzdata vs reference `zic` + zic-rs — the
RELEASE-LADDER.1 harness scales to it) → `TZDB-ATLAS.1` (join the three axes so a mismatch is *explainable*
by locating it on the upstream-release, vendor-oracle, or drop-in-environment axis — distinguishing
source-data changes from compiler differences from reader differences). This is the validated next direction;
`RELEASE-LADDER.1` (7 releases, 49/49 behaviour-match) is the working pilot.
