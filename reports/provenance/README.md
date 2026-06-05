# reports/provenance — the zone provenance archive (T18.breadth-to-40)

> **A provenance case is not admitted because it exists.** Each case is signature-verified +
> hash-pinned at the source, compiled by **both** reference `zic` and `zic-rs` from the same pristine
> region file, and witnessed by `zdump` — and the receipt states what it proves and what it does not.

This directory is the broad, **less-curated** historical/structural/ecological witness set that
complements the narrow correctness surfaces (CORE.1 = all 341 canonical zones; `T23.reader-compat` =
real-reader equivalence). It expands provenance breadth to **40 receipt-bearing cases** selected for
stress (pre-1970 weirdness · negative DST · double summer time · non-hour & sub-minute offsets · many /
sparse transitions · late political churn · renamed/linked identity · unusual footers · underrepresented
regions · reader-compat stress · leap/right · reference-vs-vendor divergence history), not for convenience.

## Contents

- **`RECEIPT-2026-06-04-breadth-to-40.md`** — the receipt: source admission (pristine 2026b,
  sig `7E37…7E34`, sha `ffad46a0…`), the 40-case table (per-zone source file + ref/zic-rs/slim hashes +
  behaviour + structural verdicts), the four claim types kept distinct, the named **Europe/Lisbon**
  finding, the bundle hash, the gate (CORE.1 341/0/0), and the non-claims.
- **`gauntlet.sh`** — the reproducible harness (compile region files with both compilers; per-zone
  zdump + sha witnesses).
- **`prov.tsv`** — machine-readable per-case rows.
- **`source-files.sha256`** — the pinned region-file hashes.
- **`zdump/`** — per-zone `zdump -v` witnesses (`<zone>.ref.zdump`, `<zone>.zrs.zdump`).

## The headline (maintainer summary)

- **40 / 40 behaviour `zdump`-MATCH** (zic-rs ≡ reference `zic` over 1900–2040).
- **23 slim byte-diffs, all behaviourally null** (verified `zdump`-identical over **all** time); 17/40 slim
  byte-parity on region-file input.
- **80 witnesses stored** (`zdump/<zone>.{ref,zrs}.zdump`); provenance-set bundle hash `c489cbb6…`.
- **Pristine 2026b signature-verified** (`tzdb-2026b.tar.lz` sha `ffad46a0…`, VALIDSIG `7E37…7E34`).
- One named finding: Europe/Lisbon, the documented T8 ref-fatter-by-1 residual.

**Reader-facing ledger:** **`docs/provenance-ledger.md`** (T18.3) — the compact 40-case table, the 15-category
coverage matrix, the source-file matrix, the named findings, and the explicit claim boundaries.

## What it claims / does not

Claims **source-provenance** (pinned pristine 2026b) + **compiler-equivalence** (behaviour vs reference).
**Cross-references** reader-compatibility (`T23.reader-compat.1/.2/.3` — incl. the cross-reader ecology:
glibc · Go · CCTZ all see zic-rs ≡ reference over 6 of these fixtures, 0 mismatch). Does **not** claim **timezone truth**
(IANA's authority), all-time byte parity, reader-universality, or exhaustiveness. See the receipt's
non-claims.
