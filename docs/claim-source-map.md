# Claim → source map (T18)

> For each load-bearing zic-rs claim (or non-claim), which source(s) support it and at what **authority
> tier**. The point of the index is exactly this traceability: *"this claim is supported by an RFC" vs
> "…by our own lab observation" vs "…is a boundary we refuse."* Sources are the `S#` rows in
> [`zic-knowledge-index.md`](zic-knowledge-index.md) / [`source-ledger.tsv`](source-ledger.tsv).

| zic-rs claim / non-claim | Supporting source(s) | Authority tier | Notes |
|---|---|---|---|
| TZif output conforms to the binary format (v1–v4, counted arrays, leap table + expiry, footer) | **S1** (RFC 9636), S2 (8536, history) | `primary_standard` | the normative format; the T15.4 validator checks it. **Standards currency (RFC9636-ERRATA.1):** RFC 9636 has **0 published errata** (authoritative `errata.json`, 2026-06-05) — the conformance target is uncorrected; predecessor RFC 8536's 4 errata are all example/appendix-only, none in the normative §3 rules — `reports/rfc9636-errata/RECEIPT-RFC9636-ERRATA-1.md` |
| The POSIX-TZ **footer** grammar (recurring-rule projection) | **S4** (POSIX/IEEE 1003.1) | `primary_standard` | copyrighted → link-only |
| **Reference `zic` behaviour** (the laws zic-rs pins, zishrink, DATAFORM, backward/backzone) | **S6** (`zic.c`/`theory.html`/NEWS) | `primary_source_repository` | the behavioural ground truth, pinned at the admitted release |
| The compiled **data** + the admitted release (only 2026b) | **S5** (IANA dist), **S7** (admitted 2026b, hash-pinned) | `primary_upstream_release` | admission = fetch + verify-sig + hash-pin |
| **CORE.1**: 341/341 behaviour-match over 1900..2040 | **S9** (the sweep) + S6/S7 (the oracle + source) | `reproducible_lab_evidence` (vs `primary` oracle) | *our* measurement against the reference oracle |
| The **two `zic` lineages**, vendor lag spectrum, packaging axes, glibc-version-stratification | **S8** (vendor-oracle lab) | `reproducible_lab_evidence` | measured receipts; S11 (glibc) is *context*, not the proof |
| zic-rs is a TZif **producer/verifier**, not a datetime library | **S10** (Jiff/tz-rs/chrono-tz) | `primary_source_repository` | they are the consumers/higher layer |
| **NON-claim:** zic-rs does not curate or define **civil time** | **S3** (RFC 6557/BCP 175) | `primary_standard` | IANA owns the maintenance process; zic-rs is downstream |
| **NON-claim:** zic-rs does not own **display names / `Etc/Unknown`** | **S12** (CLDR/Unicode) | `vendor_documentation` | CLDR's territory, explicitly out of scope |
| **NON-claim:** not a full `zic` replacement; behaviour parity is exactly CORE.1 | **S9** + `docs/differences-from-reference-zic.md` | `reproducible_lab_evidence` | parity/operational axes stay separate |
| **NON-claim cluster (ecology scope):** not TZDIST/`VTIMEZONE`/xCal/jCal · not CLDR localization / ICU / Java / Windows-registry formats · not geospatial zone lookup · not NTP/PTP/leap-smear/precision-clock authority | **S13** (`tz-link` upstream ecology map) | `primary_upstream_release` (context) | the surrounding ecosystem the project must not let its claim surface contaminate (`not-yet-ready.md`, `drop-in-compatibility-contract.md`) |
| **NON-claim:** abbreviations are observed labels (not unique zone IDs); POSIX `TZ` sign ≠ numeric-offset sign | **S13** (notation traps) | `primary_upstream_release` (context) | `misuse-resistance-ledger.md` rows |

## Authority-tier discipline

- A claim resting only on `vendor_documentation` / `secondary_reference` / `anecdotal` is **weaker** than
  one resting on a `primary_standard` or `primary_upstream_release` — the map makes that visible rather
  than flattening all citations to "a link."
- **Our own evidence** (`reproducible_lab_evidence`: S7–S9) is labelled as such — it is strong (reproducible,
  hash-pinned) but it is *our measurement*, not an external authority, and is never dressed up as one.
- **Context ≠ proof:** where a vendor doc (S11 glibc) merely *contextualises* a finding whose proof is a
  lab receipt (S8), the map says so — the measured receipt is the evidence, the doc is background.

This pairs with `risk-register.md` (the `failure_mode` column links each source to the risk that
materialises if it is wrong/misread) and `copyright-non-republication-policy.md`.
