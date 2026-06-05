# zic-rs knowledge index (T18)

> The curated, cited, provenance-recorded index of the **primary sources** zic-rs's claims rest on. It is
> an **evidence surface**, not scraped notes: every row records URL · authority · rights posture · archival
> status · supported claim. It records **claim provenance, not copyrighted bodies of text** (see
> [`copyright-non-republication-policy.md`](copyright-non-republication-policy.md)).
>
> **Humble framing:** this is **not** an alternative tzdb distribution, an authority, or an IANA/CLDR
> replacement — it is a maintainer's map of where the project's facts come from. *Start small + impeccable*
> (the **13 core sources** the project actually relies on, S1–S13, not an exhaustive 40-row sweep — breadth
> →~40 is tracked, T18.2/T18.3). Grows additively.
>
> **Archival status (operator capture pass, 2026-06-02): ✅ COMPLETE for every external source.** All ten
> external sources (S1–S6, S10–S13) are `captured` — Wayback (+ archive.today where given); S6 archived the
> actual 2026b release tarballs; the copyrighted S4/S11/S12 record only **existing external** witnesses (no
> self-mirror). S7–S9 are in-repo own-work (`not_applicable`). **Honesty:** the capture URLs were
> operator-provided and are recorded **verbatim** — zic-rs fetched nothing here and fabricates no URL.
> `canonical_url` = authority; `archive_url` = preservation witness (never promoted to primary). Residual:
> a few archive.today secondary copies (S6/S10/S13).

## `SourceLedgerEntry` (the typed row schema)

`source_id · canonical_url · title · publisher · source_kind · retrieval_date · archive_status ·
wayback_url · archive_today_url · license_or_rights_note · quote_policy · summary · claims_supported ·
evidence_strength · failure_mode`. The machine-readable form is [`source-ledger.tsv`](source-ledger.tsv);
the archival status is [`archive-ledger.tsv`](archive-ledger.tsv); claim→source mapping is
[`claim-source-map.md`](claim-source-map.md).

- **`canonical_url` = authority; `archive_url` = preservation witness** (never promoted to primary unless
  the original disappears).
- **`source_kind`** ∈ { RFC · standards_document · IANA_release_artifact · source_repository ·
  tzdb_NEWS_or_commit · vendor_documentation · vendor_package_page · OS_manual_page · package_metadata ·
  mailing_list_post · bug_report · release_note · academic_paper · community_forum_weak ·
  project_internal_evidence · **upstream_ecology_index** (a maintainer-authored map of the surrounding
  ecosystem — authority for *what exists around tzdb*, context-only for any single downstream behaviour) }.
- **`evidence_strength`** (`AuthorityKind` tier) ∈ { `primary_standard` · `primary_upstream_release` ·
  `primary_source_repository` · `primary_vendor_package` · `vendor_documentation` ·
  `maintainer_mailing_list` · `reproducible_lab_evidence` · `secondary_reference` · `anecdotal` } — so the
  index can say *"supported by an RFC"* vs *"…by a forum post only"* vs *"…is our own lab observation."*
- **`failure_mode`** = the project risk that materialises if the source is wrong or misread (links to
  `risk-register.md`).

## Sources (first cut)

### S1 — RFC 9636 · The Time Zone Information Format (TZif)
- **canonical_url:** https://www.rfc-editor.org/info/rfc9636 · **publisher:** IETF · **source_kind:** RFC ·
  **evidence_strength:** `primary_standard`.
- **summary:** the normative TZif binary-format specification (v1–v4 blocks, counted arrays, leap-second
  table + expiration, the trailing POSIX-TZ footer). Obsoletes RFC 8536 (S2). The format zic-rs emits.
- **claims_supported:** the TZif output contract; the RFC-9636 structural validator (T15.4); leap-expiry
  (T11). **failure_mode:** misreading it → a structurally-valid-but-wrong TZif (`RISK.TZIF.1`).
- **rights:** `ietf_trust_freely_available_summary_only`; quote: short compliant only. **archive:** `captured`
  (operator pass, 2026-06-02) — **wayback:** https://web.archive.org/web/20260602172641/https://www.rfc-editor.org/info/rfc9636/
  · **archive.today:** https://archive.ph/QIc1m

### S2 — RFC 8536 · The Time Zone Information Format (TZif) [predecessor]
- **canonical_url:** https://www.rfc-editor.org/info/rfc8536 · **IETF** · RFC · `primary_standard` (historical).
- **summary:** the prior TZif RFC, **obsoleted by RFC 9636**; recorded for lineage/provenance (a consumer
  may cite either). **claims_supported:** TZif-format history. **failure_mode:** citing 8536 as current
  where 9636 changed something. **rights:** as S1. **archive:** `captured` (operator pass, 2026-06-02) —
  **wayback:** https://web.archive.org/web/20260602172910/https://www.rfc-editor.org/info/rfc8536/
  · **archive.today:** https://archive.ph/jjhb4

### S3 — RFC 6557 (BCP 175) · Procedures for Maintaining the Time Zone Database
- **canonical_url:** https://www.rfc-editor.org/info/rfc6557 · **IETF** · RFC · `primary_standard`.
- **summary:** documents the **process** by which the IANA tz database is maintained (the authority
  boundary). zic-rs is strictly downstream of this — it compiles admitted source, it does not curate time.
- **claims_supported:** the civil-time-authority boundary (`does_not_claim_future_civil_time_authority`).
  **failure_mode:** treating zic-rs as a civil-time authority (`RISK.TIME.1`). **rights:** as S1. **archive:**
  `captured` (operator pass, 2026-06-02) — **wayback:** https://web.archive.org/web/20260602173246/https://www.rfc-editor.org/info/rfc6557/
  · **archive.today:** https://archive.ph/uU17k

### S4 — POSIX `TZ` environment-variable / footer grammar (IEEE Std 1003.1, The Open Group Base Specs)
- **canonical_url:** https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap08.html ·
  **publisher:** IEEE / The Open Group · **source_kind:** standards_document · `primary_standard`.
- **summary:** the POSIX `TZ` string grammar (`std offset dst [offset],rule,rule`) that the TZif footer
  encodes (the recurring-rule projection). **claims_supported:** the POSIX-footer emission/parse (T3/T8/the
  `posix_footer` surface). **failure_mode:** a wrong footer → wrong far-future projection.
- **rights:** `copyrighted_source_not_republished`; **quote:** `link_only_or_existing_archive_only`.
  **archive:** `captured` (operator pass, 2026-06-02 — *existing external preservation witnesses, created by
  the archive services, not a mirror by us*; consistent with `link_only_or_existing_archive_only` for a
  copyrighted source) — **wayback:** https://web.archive.org/web/20260531182809/https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap08.html
  · **archive.today:** https://archive.ph/5fWeS

### S5 — IANA Time Zone Database (data + code distribution)
- **canonical_url:** https://www.iana.org/time-zones · **publisher:** IANA · **source_kind:**
  IANA_release_artifact · `primary_upstream_release`.
- **summary:** the authoritative distribution of the tz **data** (`tzdataNNNN.tar.*`) and **code**
  (`tzcodeNNNN.tar.*`, incl. `zic`), signed by the tz maintainer. The source of the admitted release.
- **claims_supported:** the release-admission matrix (only **2026b** admitted); the data zic-rs compiles.
  **failure_mode:** admitting an unverified/dirty release as pristine (`RISK.HIST.1`/admission discipline).
  **rights:** `public_domain_tzdb_provenance_still_recorded`. **archive:** `captured` (operator pass,
  2026-06-02) — **wayback:** https://web.archive.org/web/20260601213037/https://www.iana.org/time-zones
  · **archive.today:** https://archive.ph/QHRtv

### S6 — tz database source (tzcode `zic.c`, `theory.html`, `NEWS`)
- **canonical_url:** https://www.iana.org/time-zones (release tarballs) · dev mirror
  https://github.com/eggert/tz · **source_kind:** source_repository · `primary_source_repository`.
- **summary:** the reference `zic` **C source** (`zic.c` — the behaviour zic-rs pins law-by-law), the
  design rationale (`theory.html`), the release history (`NEWS`), and the generators (`ziguard.awk`,
  `zishrink.awk`). The behavioural ground truth, pinned at the admitted release.
- **claims_supported:** every "`zic.c` pins …" in `docs/reference-zic-semantics.md` / `zic-deep-semantics.md`;
  zishrink/DATAFORM/backward/backzone evidence. **failure_mode:** misreading the source → a behaviour
  divergence sold as parity. **rights:** `public_domain_tzdb_provenance_still_recorded`. **archive:**
  `captured` (operator pass, 2026-06-02 — the actual **2026b release tarballs**, the admitted bytes
  themselves, archived; the strongest possible witness for S5/S6/S7): **wayback (tzdb-2026b.tar.lz):**
  https://web.archive.org/web/20260602175253/https://data.iana.org/time-zones/releases/tzdb-2026b.tar.lz
  · **(tzdata2026b.tar.gz):** https://web.archive.org/web/20260602175514/https://data.iana.org/time-zones/releases/tzdata2026b.tar.gz
  · **(tzcode2026b.tar.gz):** https://web.archive.org/web/20260602175631/https://data.iana.org/time-zones/releases/tzcode2026b.tar.gz

### S7 — Admitted tzdb 2026b reference set (in-repo provenance)
- **canonical_url:** `reports/t12_5a2-reference-admission.md` (this repo) · **source_kind:**
  project_internal_evidence (admission of an IANA_release_artifact) · `primary_upstream_release` (admitted).
- **summary:** the **signature-verified, sha256-pinned** 2026b distribution zic-rs admits (the only
  admitted release), incl. the `2026b-dirty`-vs-pristine finding and the generated-`.zi` recipe hashes.
- **claims_supported:** CORE.1's source identity; the T12.5a admission discipline. **failure_mode:**
  generalising a 2026b claim to an unadmitted release. **rights:** `own_work` (records a public-domain
  upstream). **archive:** n/a (in-repo, hash-pinned).

### S8 — zic-rs vendor-oracle lab (in-repo / external lab)
- **canonical_url:** `../zic-rs-vendor-oracle-lab/` (README · `RECEIPT-MATRIX.md` ·
  `IMAGE-PROVENANCE.md` · `receipts/*.json`) · **source_kind:** project_internal_evidence ·
  **evidence_strength:** `reproducible_lab_evidence`.
- **summary:** 17 ecology rows / 19 receipts measuring real vendor `zic` binaries (BSD · illumos · Linux ·
  embedded). The basis for the two-`zic`-lineage finding, the vendor lag spectrum, the packaging-model
  axes. **claims_supported:** every vendor-ecology statement (`RISK.VENDOR.1`); these are *our measurements*,
  not external assertions. **failure_mode:** over-generalising one receipt to a family. **rights:** `own_work`.

### S9 — CORE.1 behaviour-match sweep (in-repo)
- **canonical_url:** the sweep (`bash /tmp/t9sweep.sh`) + the milestone receipts · **source_kind:**
  project_internal_evidence · `reproducible_lab_evidence`.
- **summary:** all 341 canonical zones in 2026b behaviour-match reference `zic`/`zdump` over 1900..2040
  (341/0/0). **claims_supported:** the live behaviour claim. **failure_mode:** inflating CORE.1 into "full
  replacement". **rights:** `own_work`.

### S10 — Rust TZif consumers (Jiff · tz-rs · chrono-tz)
- **canonical_url:** https://docs.rs/jiff · https://crates.io/crates/tz-rs · https://crates.io/crates/chrono-tz ·
  **source_kind:** source_repository · `primary_source_repository`.
- **summary:** downstream Rust crates that **read** TZif at runtime — zic-rs is their **producer/verifier**,
  not a competitor. The optional `ecosystem-tests` bench loads zic-rs output with `tz-rs`.
- **claims_supported:** the producer-not-datetime-library positioning (`docs/rust-ecosystem.md`); TZif
  reader-compatibility context. **failure_mode:** marketing zic-rs as a datetime library. **rights:**
  `summary_only` (their docs are the projects'). **archive:** `captured` (operator pass, 2026-06-02) —
  **wayback (jiff):** https://web.archive.org/web/20260602182627/https://docs.rs/jiff/latest/jiff/
  · **(tz-rs):** https://web.archive.org/web/20260602182833/https://crates.io/crates/tz-rs
  · **(chrono-tz):** https://web.archive.org/web/20260602183159/https://crates.io/crates/chrono-tz

### S11 — glibc (the glibc-`zic` lineage)
- **canonical_url:** https://sourceware.org/glibc/ · **publisher:** the GNU C Library project ·
  **source_kind:** vendor_documentation · `vendor_documentation`.
- **summary:** glibc ships its **own** `zic` (version-stamped as glibc), one of the two lineages the lab
  found. *Context for* the lineage finding — the **measured** claim is the lab evidence (S8), not glibc's
  docs. **claims_supported:** the glibc-`zic` lineage (`RISK.VENDOR.1`, supporting context). **failure_mode:**
  asserting glibc behaviour from docs instead of the measured receipt. **rights:** `copyrighted_source_not_republished`,
  link-only. **archive:** `captured` (operator pass, 2026-06-02 — *existing external preservation witness,
  not a self-mirror*; policy-consistent for a copyrighted source) — **wayback:**
  https://web.archive.org/web/20260508202414/https://sourceware.org/glibc/

### S12 — CLDR / Unicode (display names · `Etc/Unknown`)
- **canonical_url:** https://cldr.unicode.org/ · **publisher:** Unicode Consortium · **source_kind:**
  vendor_documentation · `vendor_documentation`.
- **summary:** CLDR owns localized **display names** and the `Etc/Unknown` invalid-zone marker — explicitly
  **out of zic-rs's scope**. **claims_supported:** the CLDR boundary non-claim (`does_not_curate_time_or_define_display_names`).
  **failure_mode:** zic-rs claiming display-name/CLDR territory (`RISK.TIME.1`/`RISK.ADOPT.1`). **rights:**
  `copyrighted_source_not_republished`, link-only. **archive:** `captured` (operator pass, 2026-06-02 —
  *existing external preservation witness, not a self-mirror*; policy-consistent for a copyrighted source) —
  **wayback:** https://web.archive.org/web/20260601011314/http://cldr.unicode.org/

### S13 — `tz-link` (the upstream ecology map · the boundary atlas)
- **canonical_url:** https://data.iana.org/time-zones/tz-link.html (ships in the tz distribution) ·
  **publisher:** IANA tz project (Eggert et al.) · **source_kind:** `upstream_ecology_index` ·
  **evidence_strength:** `primary_upstream_release` (maintainer-authored, **public-domain** page — one of
  the safest T18 sources to quote/archive). **archive:** `captured` (operator pass, 2026-06-02) —
  **wayback:** https://web.archive.org/web/20260602175855/https://data.iana.org/time-zones/tz-link.html
  (archive.today pending).
- **summary:** the maintainer's own map of the *whole* civil-time ecosystem — release process, downstream
  distributors, **other tz compilers** (Vzic · ICU · DateTime::TimeZone · Howard Hinnant's C++ · Time4J ·
  Noda Time · …), **other TZif readers** (glibc · GLib · CCTZ · Go · Timelib/PHP · Python `zoneinfo` · …),
  TZDIST/CalDAV/`VTIMEZONE`/xCal/jCal, national legal-time histories (heterogeneous, some self-noted as
  imperfect), geospatial boundary tools, precision timekeeping (NTP/PTP/IERS/leap files; `right` vs `posix`),
  CLDR localization, Windows mappings, and notation traps (ambiguous `CST`; POSIX `TZ` sign reversal).
- **claims_supported (this is its load-bearing value — it bounds zic-rs's scope, it does not extend it):**
  *tzdb is the operational dataset, **not** civil-time law* (the page says the code/data are "by no means
  authoritative") → reinforces `RISK.TIME.1`; *the ecosystem is broader than `zic`/TZif/tzdb/one vendor* →
  the non-claim cluster on `not-yet-ready.md` (no TZDIST/`VTIMEZONE`/xCal/jCal · no CLDR localization · no
  Windows-registry mapping · no geospatial zone lookup · no NTP/PTP/leap-smear/precision-clock authority);
  *other compilers/readers exist* → zic-rs must name **which** compiler surface it replaces
  (`drop-in-compatibility-contract.md`) and that reader-ecology parity is a separate future axis
  (`adjacent-compiler-ledger`/`tzif-reader-ledger`, **T18.3/T21**, named-not-built); *source variants matter*
  (the page notes `make rearguard_tarballs` for old compilers) → corroborates the T12.5 source-variant arc +
  DragonFly's zishrink-input gap; *no fixed release schedule + short-notice government changes* → the
  `maintenance-policy.md` emergency-intake procedure.
- **failure_mode:** the project's single biggest risk — **claim-surface contamination** (zic-rs silently
  read as covering the wider ecology) → `RISK.ADOPT.1`/`RISK.TIME.1`. The defense is exactly the non-claims
  this source motivates.

## Provenance archive (T18.breadth-to-40)

Beyond the *source* ledger above, T18 maintains a **zone provenance archive** — the broad, less-curated
witness set that expands provenance breadth to **40 receipt-bearing cases** selected for historical,
structural, regional, and reader-compatibility *stress* rather than convenience. Each case is
signature-verified + hash-pinned at the source (pristine 2026b region files), compiled by **both**
reference `zic` and `zic-rs`, and `zdump`-witnessed. **Result: 40/40 behaviour-match**; it claims
**source-provenance + compiler-equivalence**, cross-references reader-compatibility (`T23.reader-compat.3`
widened this to a cross-reader ecology — glibc · Go · CCTZ all interpret zic-rs output ≡ reference over 6
ledger fixtures, 0 mismatch; Java/PHP/ICU classified `unsupported_by_reader`), and explicitly does
**not** claim timezone truth. Lives at **`reports/provenance/`** (`RECEIPT-2026-06-04-breadth-to-40.md` ·
`gauntlet.sh` · `prov.tsv` · `zdump/`); one named null finding (Europe/Lisbon slim residual). The
**reader-facing ledger** (T18.3) — compact 40-case table + 15-trap-category coverage matrix + source-file
matrix + named findings + claim boundaries — is **`docs/provenance-ledger.md`**. This is a distinct artifact
from the `SourceLedgerEntry` rows below (those index *documents*; the archive witnesses *compiled zones*).

## Pending (operator/lab task)

- **Archival capture — ✅ COMPLETE for every external source (operator pass, 2026-06-02).** All ten
  external sources are now `captured`: S1/S2/S3 (RFCs, Wayback + archive.today), S5 (IANA), **S6 = the actual
  2026b release tarballs** (`tzdb-2026b.tar.lz` + `tzdata2026b.tar.gz` + `tzcode2026b.tar.gz`), S10 (the three
  Rust consumer crates — jiff/tz-rs/chrono-tz), S13 (`tz-link`); the **copyrighted** S4 (POSIX) / S11 (glibc) /
  S12 (CLDR) recorded their **existing external** Wayback witnesses (not self-mirrors — consistent with
  `link_only_or_existing_archive_only`). URLs live in the per-source entries + `archive-ledger.tsv`. S7–S9 are
  in-repo own-work (`not_applicable`). **Residual (minor, secondary witness only):** S6/S10/S13 archive.today
  copies. *Honesty: these are operator-provided capture URLs recorded **verbatim** — zic-rs did not fetch them
  in this no-network env and fabricates none.*
- **Breadth:** grow toward the 20–40 primary-source target (tzdb NEWS entries, specific vendor package
  pages, OS man-pages) — additively, each impeccable, never a scraped dump.
