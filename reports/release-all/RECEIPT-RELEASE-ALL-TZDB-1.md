# RECEIPT — RELEASE-ALL.TZDB.1 (complete tzdb bundle gauntlet) — 2026-06-05

> **Claim wording (binding):** *RELEASE-ALL.TZDB.1 does not create new civil-time truth claims. It completes
> the modern complete-bundle archive phase by classifying every signed `tzdb-*.tar.lz` bundle as a combined
> data+code release artifact and recording how its data/code identity relates to the already-admitted
> RELEASE-ALL and atlas rows.*

## What was checked

The **48** complete-bundle archives `tzdb-2016g.tar.lz … tzdb-2026b.tar.lz` (the era IANA ships a single
signed lzip bundle containing **both** tzdata and tzcode). All 48 were already **GOODSIG**-verified by
RELEASE-ALL.1 (Paul Eggert; hashes in `reports/release-all/verified.tsv`). `lzip` is not installed here, so
the bundles were decoded with a **pinned pure-Python lzip decoder** (`reports/release-all/unlzip.py`) →
`tar -x`. Per bundle:

| axis | check |
|---|---|
| **provenance** | GOODSIG signature + sha256 (from `verified.tsv`) |
| **identity** | filename version `==` the internal `version` file (and the NEWS heading, for cross-ref) |
| **combined?** | does it contain **both** data (`africa`…) **and** code (`zic.c`)? |
| **data == pair** | are the bundle's 9 region files **byte-identical** to the standalone `tzdata<v>` pair's? |
| **behaviour** | inherited from RELEASE-ALL.DATA.1's verdict for `<v>` — *rigorously*, because byte-identical data ⇒ identical zic-rs compile |

Reproduce: `bash reports/release-all/tzdb-gauntlet.sh` → `reports/release-all/tzdb-bundles.tsv`.

## Result — 48 / 48

| | |
|---|--:|
| **verified_combined_release** | **48 / 48** |
| GOODSIG (Paul Eggert) | 48 / 48 |
| combined (data **and** code present) | 48 / 48 |
| data **byte-identical** to the standalone tzdata pair (9/9 region files) | 48 / 48 |
| internal `version` file == filename (identity) | 48 / 48 — **0 identity contradictions** |
| data differs from pair | **0** |
| bundle versions in the RELEASE-ALL.DATA.1 `match` band | 48 / 48 |

**Every signed complete bundle is an internally-consistent combined data+code release artifact whose data is
byte-identical to the standalone pair RELEASE-ALL.DATA.1 already compiled and behaviour-verified.**

## How the behaviour verdict is grounded (no new claim)

For each bundle the 9 region files are **byte-identical** to the standalone `tzdata<v>` pair (`9/9`). Because
zic-rs is deterministic, compiling byte-identical source yields byte-identical TZif — so the bundle's compile
result **is** RELEASE-ALL.DATA.1's result for `<v>`, which is `match` (zdump-behaviour-match vs reference
`zic` over the fixtures). This is an **inheritance by proven data identity**, not a re-asserted claim and not
a new civil-time statement: the bundle adds *packaging provenance* (one signed artifact), not new behaviour.

## Cross-consistency with RELEASE-METADATA.1

The bundles carry the **same** NEWS files as the standalone tarballs, so the two minor metadata deviations
RELEASE-METADATA.1 found appear **identically** in the bundles — confirming they are **upstream**, not a
packaging artifact:

- `tzdb-2019a` → `version` `2019a`, NEWS heading `Release 20198` (the upstream typo).
- `tzdb-2026b` → `version` `2026b`, NEWS heading `Release 2026a` (the NEWS lag).

Both bundles are still `verified_combined_release` — the verdict turns on the **authoritative** identity
(filename == `version` file), which holds; the NEWS surface deviation is the separately-recorded
RELEASE-METADATA.1 fact.

## How this relates to the archive-ecology rows

| phase | scope | this bundle phase |
|---|---|---|
| RELEASE-ALL.1 | 785 index entries indexed + 48 bundles GOODSIG | TZDB.1 **opens** the 48 bundles |
| RELEASE-ALL.DATA.1 | 276 standalone tzdata compiled (150 match …) | TZDB.1 proves the 48 bundles' data **== the pair** → inherits `match` |
| RELEASE-METADATA.1 | release-identity across 276 | TZDB.1 confirms the same identity holds **inside** the bundle |
| TZDB-ATLAS | the join | TZDB.1 adds the combined-artifact provenance row |

The `tzcode`-only archive (246 entries) is the remaining sub-phase (`.CODE.1`); the **code** half is already
**present and signed inside every bundle here** (`zic.c` etc.), so the bundle phase covers code *presence*
(not a zic-rs behaviour comparison — tzcode is the reference compiler's own source, out of zic-rs's output scope).

## Non-claims

- **No new civil-time truth claim** — the behaviour verdict is *inherited* from DATA.1 via proven byte-identity,
  not re-derived; the bundle adds provenance, not new time data.
- `verified_combined_release` = signed + identity-consistent + combined + data-matches-pair; it is **not** a
  statement that the bundle's *code* (tzcode) was compiled or run (it is the reference compiler's source).
- Bounded to the 48 modern signed bundles (2016g–2026b); pre-2016 releases predate the complete-bundle format.
- Signature trust is `fingerprint_anchored` (Paul Eggert's published key), not web-of-trust.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 519 tests unaffected; doc-staleness green. Cross-linked
from STATUS · the atlas · RELEASE-ALL.DATA.1 · RELEASE-METADATA.1.
