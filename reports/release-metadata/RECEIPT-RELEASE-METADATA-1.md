# RECEIPT — RELEASE-METADATA.1 (tzdb release-identity contradiction receipt) — 2026-06-05

> **Claim wording (binding):** *RELEASE-METADATA.1 does not assert civil-time correctness. It verifies that
> the release identity used by zic-rs evidence receipts is derived from multiple upstream metadata surfaces
> and records contradictions as provenance facts rather than silently trusting a single label.*

## Method

For every cached tzdata release (the **same 276** as RELEASE-ALL.DATA.1), derive the release identity from
several independent upstream surfaces and classify their agreement. A contradiction is **recorded, not
resolved or hidden**. Surfaces:

| surface | source | role |
|---|---|---|
| `filename_id` | the IANA index filename (`tzdata<X>.tar.gz` → `X`) | **authoritative tag** |
| `version_file` | the `version` file inside the archive | **authoritative tag** |
| `news_top` | the top `Release <X>` heading in the `NEWS` changelog | changelog surface |
| `sig` | OpenPGP signature status (`reports/release-all/verified.tsv`) | provenance |
| `bundle` | does a `tzdb-<X>.tar.lz` complete bundle exist alongside the tzdata/tzcode pair? (index) | packaging |

Reproduce: `bash reports/release-metadata/gauntlet.sh` → `reports/release-metadata/release-metadata.tsv`.

## Result — 276 releases

| verdict | count | meaning |
|---|--:|---|
| **consistent** | **63** | every present surface gives the same id (modern releases) |
| **legacy_unstructured** | **211** | pre-structured-metadata era — only the filename tag exists (no `version` file, no `NEWS` heading) |
| **minor_metadata_contradiction** | **2** | the two **authoritative** tags (filename == `version` file) agree, but the `NEWS` changelog heading deviates |
| **major_metadata_contradiction** | **0** | — |
| unavailable | 0 | — |

**The headline:** across all 276 releases the **two authoritative identity tags (filename + `version` file)
never disagree** — the release identity zic-rs's evidence receipts use (the atlas `release_id`) is internally
consistent. Only the *changelog* surface deviates, in exactly **2** releases, and both are now recorded facts.

### The 2 minor contradictions (the campaign earning its keep)

| release | filename | version file | NEWS heading | what it is | pinned hash |
|---|---|---|---|---|---|
| **2019a** | `2019a` | `2019a` | **`Release 20198`** | a genuine **upstream NEWS typo** — `2019a` mistyped as `20198`. The `version` file is correct; trusting the NEWS label alone would record a nonsensical id | `tzdata2019a.tar.gz` sha256 `90366ddf…` |
| **2026b** | `2026b` | `2026b` | `Release 2026a` | the data NEWS has **no `2026b` heading** — its latest entry is `2026a` (the headline BC/Vancouver change is logged under the prior heading; a code-fix-style release where the `version` tag advances but the data changelog heading lags) | `tzdata2026b.tar.gz` sha256 `114543d9…` |

Both were **verified against the pristine archives** (clean re-extract, hashes above), not the gauntlet cache.
The classification is `minor` (not `major`) precisely because the two authoritative tags still agree — only
the human-readable changelog surface deviates. *This is the exact failure a single-label release-id would
hide:* a receipt that read the NEWS heading for 2019a would stamp `20198`.

### Surface coverage (recorded honestly)

- **`version` file + structured `NEWS` heading** exist only from the structured-metadata era (~2016+); the
  **211 legacy_unstructured** releases (mostly pre-2013, incl. the `.tar.Z` era) carry only the filename tag.
  This is recorded as a state, not a failure — old releases simply had fewer identity surfaces.
- **Signatures:** 7 `GOODSIG` (the RELEASE-ALL.1 verified subset, Paul Eggert), 269 `unverified` **here** —
  i.e. not in our verified subset, **not** a claim they are unsigned.
- **Complete bundle vs pair:** 48 releases ship a `tzdb-<X>.tar.lz` complete bundle alongside the
  tzdata/tzcode pair; 228 are pair-only.

## How this supports the atlas

The TZDB Evidence Atlas and the RELEASE-ALL receipts key every row on a `release_id`. RELEASE-METADATA.1
shows that id is **derived from the authoritative `version` file / filename (which never contradict each
other across 276 releases)**, and that the only deviations are in the *changelog* surface — now enumerated
(2019a, 2026b) rather than silently absorbed. The atlas's release identity is provenance-grounded.

## Non-claims

- **Not a civil-time-correctness claim** — this is about *release identity metadata*, not the time data.
- A `minor_metadata_contradiction` is a **provenance fact**, not a defect verdict on the release (the 2026b
  NEWS lag is a normal code-fix-release pattern; the 2019a `20198` is an upstream typo we record, not fix).
- `unverified` signature = not in our verified subset here, not "unsigned".
- `legacy_unstructured` = fewer identity surfaces in that era, not a contradiction.
- Bounded to the 276 cached stable tzdata releases; the announcement/mailing-list surface is **not** ingested
  (no admitted local copy) — recorded as out-of-scope rather than guessed.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 519 tests unaffected; doc-staleness green. Cross-linked
from STATUS · the atlas · RELEASE-ALL.DATA.1.
