# Copyright / non-republication policy (T18)

> **T18 preserves claim provenance, not copyrighted bodies of text.** The knowledge index records *where*
> a fact comes from and *what it says* (a short factual summary), enough to support a zic-rs claim — it does
> **not** mirror, reproduce, or store the source's protected text.

## Allowed (for any source)

- canonical URL · title · publisher/project/vendor · retrieval/recorded date
- a **short factual summary** in our own words (the claim the source supports)
- a **short, compliant quotation** only where strictly necessary (fair-use-sized, attributed)
- a claim mapping (which zic-rs claim this source supports, at which authority tier)
- a preservation/archival link **if one already exists or is lawful to create** (a *witness*, not the
  authority — see below)

## Not allowed

- copying full articles / pages
- storing full proprietary or copyrighted text in the repo
- mirroring copyrighted documents or vendor docs
- embedding large excerpts (beyond a short compliant quote)

## Per-source rights posture (recorded, not assumed)

Each ledger row carries a `license_or_rights_note` and a `quote_policy`:

- **Freely-available standards (IETF RFCs):** summarise + short compliant quote; do not reproduce the full
  RFC. `license_or_rights_note = ietf_trust_freely_available_summary_only`.
- **Copyrighted / vendor-owned (POSIX/IEEE-Open-Group, CLDR/Unicode, vendor docs, OS manuals):**
  `license_or_rights_note = copyrighted_source_not_republished`, `quote_policy = link_only_or_existing_archive_only`
  — link + factual summary only; **no archival mirror created by us**.
- **Public-domain-ish (the IANA tz database / tzcode):** `license_or_rights_note =
  public_domain_tzdb_provenance_still_recorded` — *public-domain ≠ no-provenance*: we still record source
  identity, version, and hash (the project's standing doctrine).
- **Our own work (the vendor-oracle lab, CORE.1 sweeps, reports):** `license_or_rights_note = own_work`.

## Archive policy (`canonical_url` = authority, `archive_url` = preservation witness)

For each pertinent public source we *attempt* (where lawful) to record Wayback / archive.today preservation
links **against link-rot** — but the **canonical URL stays the authority**; an archive URL is a witness,
**never promoted to primary** unless the original disappears. Copyrighted sources are **link-only** (no
archive mirror created by us). **Capture-honesty rule:** where capture cannot run (no network), the archival
columns stay **`pending_capture`** with the canonical URL pinned, and **no archive URL is fabricated**;
where an operator runs the pass, the resulting URLs are recorded **verbatim**. **Status (2026-06-02 operator
pass): COMPLETE for every external source** — all ten (S1–S6, S10–S13) are `captured` (copyrighted S4/S11/S12
record only **existing external** witnesses, never a self-mirror); see `zic-knowledge-index.md` + `archive-ledger.tsv`.

See `zic-knowledge-index.md` (the index), `source-ledger.tsv` / `archive-ledger.tsv` (the typed ledgers),
and `claim-source-map.md` (claim → source). This pairs with `RISK.SOURCE.1` in `risk-register.md`.
