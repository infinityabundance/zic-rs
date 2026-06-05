# Risk register — how zic-rs can hurt people, and the armor

> **Doctrine.** *zic-rs treats every serious failure mode as a **claim-boundary problem** first: the
> project must either prove the claim with typed evidence, receipts, hashes, and tests, or refuse the
> claim explicitly.* The dangerous failures here are not "Rust memory bugs" first — they are
> **claim-boundary bugs**: wrong claim · wrong authority · wrong oracle · wrong source release · wrong
> vendor generalization · wrong semantic conclusion · wrong install-durability assumption. Those are the
> ones that propagate into serious infrastructure.
>
> **The thing that makes timezone tooling dangerous is not a crash — it is a structurally-valid,
> plausible-but-wrong artifact that loads fine and is wrong at a civil-time boundary.** A loud failure is
> safe; a silent wrong answer is not. This register exists so every such mode is named and bound to a
> guard or an explicit non-claim, never left to folklore.
>
> This is a standing, **append-only** document (drafted early — like `differences-from-reference-zic.md`
> and `panic-policy.md` — because it summarises already-shipped receipts; its formal owners are T17
> reliability + T20 security). Rows are never deleted or softened; status fields move forward only.
> **Status vocabulary:** `guarded` (a real, tested guard exists) · `partial` (guarded in part; named
> residual) · `doctrine` (the rule + non-claim are recorded; the enforcing build is a tracked future
> milestone) · `deferred` (named, owned by a later milestone, not yet built). Per the project's
> **doc-evidentiary-density**
> rule, this is dense on purpose: a serious reviewer should be able to trace each risk to its evidence,
> not skim a conclusion.

---

## RISK.TIME.1 — civil-time-truth overclaim

- **Failure mode.** zic-rs is taken to *define* civil time, rather than to *compile admitted IANA tzdb
  source*. Time is a legal/political fact set by governments; the tzdb is a maintained public process
  (RFC 6557 / BCP 175) coordinated around IANA — zic-rs is strictly **downstream** of that authority and
  of the data itself.
- **Consequence.** cron at the wrong local time · billing windows close wrong · logs sort into the wrong
  hour · audit trails disagree · compliance/court deadlines miscompute · distributed systems compare
  wrong local timestamps · calendars drift across a DST transition.
- **Where it enters.** marketing language · a reviewer assuming "compiles tz ⇒ owns tz" · using a future
  projection (POSIX footer) as a civil-time guarantee.
- **Evidence surfaces.** `docs/tzdb-governance.md` (the IANA/CLDR authority boundary) · release-admission
  matrix (only 2026b admitted; `ReferenceLocatorKind`/`SignatureTrustModel`) · `release-diff` +
  semantic-report (changes are *measured against the admitted release*, not asserted).
- **Tests / receipts / hashes.** the admitted-2026b archive (`reports/t12_5a2-reference-admission.md`,
  signature + sha256-pinned) · `declared_scope_hash` (the claim envelope is a hash, not a slogan).
- **Status.** `guarded`.
- **Non-claim.** `does_not_claim_future_civil_time_authority` · `does_not_curate_time_or_define_display_names`.
- **Hardening owner.** T19 `TRUST.md` (front-door restatement) · T20 (legal/compliance persona).
- **Boundary.** *zic-rs compiles admitted tzdb source; it does not determine civil-time truth.*

## RISK.TZIF.1 — structurally-valid-but-semantically-wrong TZif

- **Failure mode.** a compiled TZif parses cleanly and loads in every reader yet diverges semantically
  from reference output. Format validity (RFC 9636) is **not** behavioural parity.
- **Consequence.** readers accept the file · operators trust it · the drift stays hidden until a
  transition boundary, then every consumer is silently wrong at once.
- **Where it enters.** conflating the structural validator's "conformant" verdict with "correct" · byte
  differences assumed semantic, or semantic differences assumed structural.
- **Evidence surfaces.** **CORE.1** (341/341 behaviour-match over 1900..2040, the binding claim) · T15.4
  `tzif-validate` — **five separate verdicts**, never one `valid:true` · `semantic-report`
  (`zdump`-backed) kept a separate surface · `release-diff` keeps the **structural axis and behavioural
  axis strictly separate** · `VALID_DISAMBIGUATION` (7 distinct senses of "valid", public in
  `conformance_status`).
- **Tests / receipts / hashes.** the CORE.1 sweep (`bash /tmp/t9sweep.sh` → 341/0/0) · `tests/tzif_*` /
  `tests/semantic_*` / the `rfc9636` validator tests (incl. the type-index-bounds violation) ·
  **standards-currency check (RFC9636-ERRATA.1):** RFC 9636 has **0 published errata** (authoritative
  `errata.json`, sha `cc92d64d…`, 2026-06-05) — the validator's normative target is uncorrected; the
  predecessor RFC 8536's 4 errata are all example/appendix-only (`reports/rfc9636-errata/`). Re-check each
  release; a new RFC 9636 erratum is a signal to re-assess the validator.
- **Status.** `guarded`.
- **Non-claim.** `does_not_claim_arbitrary_tzif_roundtrip` · structural validity is explicitly *not*
  semantic behaviour (in-report note).
- **Hardening owner.** T15.4 enrichment (dual-block consistency, after-last-transition witness) — tracked.
- **Boundary.** *TZif structural validity is not semantic parity.*

## RISK.LEAP.1 — leap-second / expiry mistake

- **Failure mode.** leap handling is rare, specialized, and easy to accidentally flatten or mis-expire.
  RFC 9636 gives leap-table expiration + post-expiry interpretation real semantics; treating leap as a
  normal timestamp edge is the trap.
- **Consequence.** `right/` zones wrong · post-expiry timestamps interpreted inconsistently · far-future
  timestamps look authoritative when they are not · `time_t`/range × leap interactions go wrong.
- **Where it enters.** inferring leap behaviour from output shape · applying `-r` truncation with a
  rolling leap · assuming smear.
- **Evidence surfaces.** leap is **opt-in** (`-L`, T11), never the default; `LeapSourceMode` typed
  (T17.2) · T15.4 `LeapExpiryVerdict` separate · `release-diff` `leap_only` change-kind exists ·
  rolling-leap + `-r` is a **hard error** (`compile/leap.rs`).
- **Tests / receipts / hashes.** `fixtures/leap/reference/*.tzif` (stationary/rolling/v4-expires,
  byte-pinned) · `tests/leap.rs` · the `right/UTC` 27-leap reproduction · **T23.reader-compat.2**
  (`reports/reader-compat/RECEIPT-2026-06-03-appendix-a.md`) which **found a real divergence** below.
- **FOUND + FIXED (T23.reader-compat.2, 2026-06-03).** The reader gauntlet found that zic-rs's `right/`
  profile leaped the **table** but left a zone's **transition times** at POSIX values, so `right/` output for
  zones with transitions drifted behind reference by the accumulated leap count (`zdump` on
  `right/America/New_York`); `right/Etc/UTC` matched only because it has no transitions — exactly why T11's
  `right/UTC` witness missed it. **Fixed:** `apply_leaps` now shifts each transition by the cumulative leap
  correction effective there (`right/` only — POSIX never calls it). Verified `right/{America/New_York,
  Europe/London,Etc/UTC}` `zdump`-match reference · POSIX New_York unchanged · CORE.1 341/0/0 · +1 regression
  test (`right_profile_shifts_transitions_by_cumulative_leap_correction`).
- **Status.** `guarded` for the implemented surface (stationary/rolling/expiry, opt-in), **including `right/`
  transition-bearing zones** for the tested fixtures; POSIX/default unaffected. The `-r`×leap-expiry
  *interaction* still has no semantic witness → non-claim.
- **Non-claim.** `does_not_claim_leap_smear_semantics` ·
  `does_not_claim_range_truncation_leap_expiry_interaction_parity_without_witness` ·
  `does_not_claim_universal_leap_profile_parity_beyond_tested_fixtures` (verified for the tested `right/`
  zones, not a universal leap-profile theorem).
- **Hardening owner.** T15.4 (`LeapExpiryVerdict` matches-ref/mismatch needs the oracle) · T20.
- **Boundary.** *Leap records are explicit evidence; zic-rs does not infer smear or future-leap truth.*

## RISK.VENDOR.1 — vendor-parity overgeneralization

- **Failure mode.** one vendor's measured result is taken to generalize to another. The vendor lab proved
  the world is fractured (tzcode-zic vs glibc-zic, old forks, fat/slim defaults, version-stratified
  diagnostics, package-selected lineages, build-host vs runtime-target split).
- **Consequence.** packagers assume the wrong bloat default · operators assume glibc version implies
  tzdata behaviour · `doctor` output masks old-fork-like diagnostics · vendor-specific compat gaps missed.
- **Where it enters.** collapsing the six-axis matrix · reading "glibc present" as "glibc-zic selected" ·
  reading "RPM" as "glibc-zic".
- **Evidence surfaces.** `vendor-oracle-receipt-v1` receipts are **immutable, per-vendor**, admitted by
  rule, never inferred · `known_divergences` explicit per receipt · `receipt_production_mode` recorded
  (`RECEIPT-MATRIX.md`) · the six axes {lineage · tzcode/glibc version · behaviour-tier · bloat_default ·
  packaging_model · lineage_selection_source} are kept **independent** · the "do not collapse" map.
- **Tests / receipts / hashes.** 19 receipts / 17 ecology rows (`../zic-rs-vendor-oracle-lab/`,
  each with binary sha256 + package ownership + `IMAGE-PROVENANCE.md`) · `tests/vendor_oracle_receipt.rs`.
- **Status.** `guarded`.
- **Non-claim.** `does_not_claim_unadmitted_vendor_parity` · `does_not_ship_or_operate_vendor_qemu_labs_in_core_repo`.
- **Hardening owner.** T16.6.x (matrix renderer derives only what receipt fields support; honest unknowns).
- **Boundary.** *A vendor receipt admits one measured vendor ecology, not a family-wide theorem.*

## RISK.DIAG.1 — safe rejection confused with diagnostic-class parity

- **Failure mode.** a vendor (OpenBSD/DragonFly/Ubuntu-2.39/Alma-2.34) **safely rejects** a malformed
  input (exit ≠ 0) but classifies it differently than modern zic / zic-rs — and a naïve test reads
  "rejected" as "diagnostic parity."
- **Consequence.** a test suite says "passes" on exit code alone · diagnostic tooling silently loses
  class fidelity · operators believe a typed diagnostic contract matches when only the *rejection* does.
- **Where it enters.** comparing exit status instead of class+location · a vendor receipt admitted on
  rejection without recording the class divergence.
- **Evidence surfaces.** per-receipt `class_location_verdicts` (a real class+location compare, **not**
  exit-code only) · `known_divergences` (e.g. `continuation_without_zone`, `nul_byte`→"line too long") ·
  the append-only `ZIC001`–`ZIC026` contract · the standing **"safe rejection ≠ diagnostic-class
  parity"** doctrine.
- **Tests / receipts / hashes.** `tests/diagnostic_parity.rs` (class→location→wording-last vs `zic -v`) ·
  the per-vendor 3–4/5 core-5 verdicts in the lab.
- **Status.** `guarded`.
- **Non-claim.** `does_not_claim_byte_exact_stderr_wording_parity`.
- **Hardening owner.** T13 contract (closed); vendor rows extend it on demand.
- **Boundary.** *Rejection parity is not diagnostic parity.*

## RISK.ORACLE.1 — host / oracle contamination

- **Failure mode.** `zdump` reads the host's installed zoneinfo instead of the generated file, or a report
  uses a PATH tool that is not the admitted reference — so the "match" is against the wrong bytes.
- **Consequence.** a semantic report validates the host's installed zone, not zic-rs output ·
  `release-diff` uses the wrong `zdump` · an operator believes behaviour matched when the oracle read
  different bytes.
- **Where it enters.** invoking `zdump <name>` (zone-DB lookup) instead of an explicit path · trusting a
  PATH binary without recording its identity.
- **Evidence surfaces.** `zoneinfo_resolution = explicit_tzif_path_argument` (the oracle reads *our*
  bytes, T15.5) · `oracle_identity` (binary sha256 · argv · env `TZ`/`LC_ALL`) · `doctor`
  `ToolVersionStatus` + `HashReadStatus` (path **and** hash of the resolved tool, T17.3) ·
  `OracleFailureScope` (T17.3) · the live-vs-sealed reference split (T16.3).
- **Tests / receipts / hashes.** `tests/reference_admission.rs` · the doctor present-tool test (path +
  read hash) · `zdump` always invoked on an absolute path to a freshly-written file.
- **Status.** `guarded`.
- **Non-claim.** an oracle result is admitted only when the oracle identity **and** input path are known.
- **Hardening owner.** T17.3 (shipped) · T20 (supply-chain persona).
- **Boundary.** *An oracle result is only admitted when the oracle identity and input path are known.*

## RISK.DIFF.1 — "unknown" silently converted into "no change"

- **Failure mode.** `release-diff` lacks a `zdump` oracle (or it fails) and the behaviour axis is reported
  as "same" instead of "not assessed." This is a silent killer: a release looks safer than it is.
- **Consequence.** release changes look safer than they are · operators skip review · silent semantic
  drift survives into deployment.
- **Where it enters.** absent oracle defaulting to "unchanged" · one failed identifier poisoning the whole
  run or being read as "no diff."
- **Evidence surfaces.** `oracle_mode: unavailable` (absence is **visible**) · the
  `behaviour_unassessed` change-kind (an explicit "we did not check") · `OracleFailureScope`
  {`global_tool_unavailable` vs `row_or_identifier_failure`} (T17.3 — a per-row failure is recorded on
  that row's `behaviour_error`, never poisoning the rest) · the report's `non_claim`.
- **Tests / receipts / hashes.** `tests/release_diff.rs`: `changed_zone_without_oracle_is_behaviour_unassessed`
  · `unresolvable_zdump_is_global_unavailable_up_front` · the split-seam test.
- **Status.** `guarded`.
- **Non-claim.** *absence of the oracle is not evidence of no behaviour change.*
- **Hardening owner.** T16.6/T17.3 (shipped).
- **Boundary.** *Unassessed behaviour is not unchanged behaviour.*

## RISK.PATH.1 — output traversal / materialization mistake

- **Failure mode.** a compiler that materializes an output tree writes outside `--out`, clobbers host
  files, follows a symlink, or confuses copy vs symlink materialization.
- **Consequence.** write outside the output dir · clobber host files · unsafe symlink following · wrong
  localtime/posix/right layout.
- **Where it enters.** a hostile zone/link *name* used as a path · a pre-planted file/symlink/dir at the
  output leaf · `--force` following a symlink.
- **Where it enters (cont.)** · `--force` replacing a symlink via a remove-then-create gap.
- **Evidence surfaces.** `ZIC008` path-traversal reject (absolute/`..`/`//`/trailing-`/`/leading-`-`/NUL)
  · the operational/materialization diagnostic layer · `OutputTree` + `LinkMode` typed (T17.2) · T9.3
  compile-all-to-memory-then-write (no partial install after a fatal) · T14.6 hostile-output-tree
  (pre-planted leaf fails closed, **never written through**; regular-file `--force` = `rename`, replaces
  not follows) · **T17.4: every leaf publish is now check-then-act-free** — regular files via `hard_link`
  (exclusive create) / `rename` (`--force`); **symlinks** via `symlink_exclusive` (the `symlink(2)`
  `EEXIST` is the exclusive create, no `exists()` race) and, under `--force`, a **temp-symlink + `rename`**
  (atomic, operates on the link itself — replaces, never follows; no remove-then-create gap).
- **Tests / receipts / hashes.** `tests/output_safety.rs` · `tests/hostile_output_tree.rs` ·
  `tests/zone_name_path_policy.rs` · **T17.4** `fs::output_tree::tests::{symlink_exclusive_create_rejects_preexisting_without_following,
  symlink_force_overwrite_replaces_atomically}`.
- **Status.** `partial` — every *leaf* operation is now race-free / fail-closed (guarded, incl. the
  symlink overwrite, T17.4); the residual is narrower and named precisely: a **concurrent
  parent-*component* symlink swap** *during path resolution* (`create_dir_all`/open resolve parent
  symlinks at syscall time) remains **not claimed** — closing it needs fd-relative `openat`/`O_NOFOLLOW`,
  which std does not expose without `unsafe`/a new dep (both forbidden), so it stays
  `RequiresOpenatStyleHardening`.
- **Non-claim.** `does_not_claim_full_toctou_resistance` (now scoped specifically to the parent-component
  swap race, not the leaf).
- **Hardening owner.** T17.4 closed the leaf races; the parent-component residual → T20 (an `openat`-style
  materialization would need the unsafe/dep decision revisited).
- **Boundary.** *Compilation correctness and installation safety under hostile concurrency are separate
  claims; T17.4 closed the leaf-level races, the parent-component swap race is explicitly still open.*

## RISK.INSTALL.1 — atomic-write / crash-durability overclaim

- **Failure mode.** the install is believed crash-durable, but syncing the temp file is **not** the same
  as a parent-directory fsync + rename durability contract.
- **Consequence.** an operator believes the install is crash-durable · power loss leaves a missing or
  stale file · a deployment pipeline trusts a stronger guarantee than is implemented.
- **Where it enters.** reading "atomic publish" as "crash-durable" · assuming `rename` alone survives
  power loss without a directory fsync.
- **Evidence surfaces.** the atomic temp-file + no-clobber publish (real) · **T17.4 implemented the full
  three-layer durable publish for the install path**: (1) content `sync_all()` on the temp file before
  publish, (2) atomic publish (`hard_link`/`rename`), (3) **parent-directory fsync** (`fsync_dir`, Unix)
  after the publish — so each written file's *directory entry* is crash-durable, not just its content.
  Ephemeral scratch writers (the `compare` oracle tree, the release-diff zdump tree) pass `durable=false`
  to skip the pointless fsync on soon-deleted files.
- **Tests / receipts / hashes.** `fs::output_tree::tests::durable_write_succeeds_and_round_trips` (the
  install path incl. dir-fsync must succeed on the test FS) + the existing atomic-publish/cleanup tests;
  the three-layer contract is documented in `src/fs/atomic_write.rs`'s module header.
- **Status.** `partial` → **improved**: **per-file crash-durable publish is now guarded** (content fsync +
  atomic publish + parent-dir fsync, **Unix**). What remains *explicitly not claimed*: **whole-tree
  crash-atomicity** — a crash *mid-run* can leave some files durably published and others not (there is no
  tree-level transaction), and directory-entry fsync is a Unix guarantee (a documented no-op elsewhere).
- **Non-claim.** `does_not_claim_whole_tree_crash_atomic_install` (a crash mid-run may leave a partial
  tree; per-file publish is durable, the *set* is not transactional) · durability is Unix-scoped.
- **Hardening owner.** T17.4 (shipped the per-file durable publish); whole-tree transactional install (if
  ever needed) → a future milestone, named not faked.
- **Boundary.** *Each file is durably published (content + entry); the whole output tree is not a single
  crash-atomic transaction.*

## RISK.RESOURCE.1 — hostile giant input / resource exhaustion

- **Failure mode.** even memory-safe, unbounded parsing of a malicious/huge source set becomes an
  availability problem.
- **Consequence.** CI-runner exhaustion · packager build failure · hostile input triggers huge
  memory/time · DoS against any service wrapper exposing zic-rs.
- **Where it enters.** a `.zi` with millions of zones/rules/links/leaps · a pathological link chain · a
  giant single file.
- **Evidence surfaces.** `limits::ResourceLimits` (T17.1b): per-file **source bytes** · **zone / rule /
  link / leap** counts · **link-chain depth** · **continuation-eras/zone** · the pre-existing line-length
  cap (`MAX_LINE_LEN`, `ZIC017`) and per-zone transition cap (`MAX_TRANSITIONS`, `ZIC009`) ·
  `overflow-checks = true` as a last-resort backstop · `docs/panic-policy.md`.
- **Tests / receipts / hashes.** `src/limits.rs` tests (each dimension breached with a tiny
  `ResourceLimits`; the 400-link acyclic chain hits the depth cap; the production defaults pass a
  500-zone DB).
- **Status.** `guarded` (caps are generous — they bound the pathological tail, not a tight quota).
- **Non-claim.** not total DoS resistance; a cap breach is an **operational safety refusal**
  (`Error::config`), **not** a `ZIC###` grammar diagnostic.
- **Hardening owner.** T17.1b (shipped) · CLI-configurable caps → T17.2+.
- **Boundary.** *A resource-cap breach is an operational safety refusal, not a grammar diagnostic.*

## RISK.COUNT.1 — integer / count arithmetic (overflow / OOB index)

- **Failure mode.** TZif header count fields and transition arrays are classic overflow /
  multiplication-overflow / out-of-bounds-index sites — *counts from input are hostile until checked.*
- **Consequence.** panic · misparsed TZif · truncated arrays · wrong transition/type mapping · a false
  structural verdict.
- **Where it enters.** `block_len` count arithmetic on untrusted u32s · `Vec::with_capacity(count)`
  pre-allocating from a declared count · a transition `type_index` past `typecnt` indexing `types[]`.
- **Evidence surfaces (T17.5 `CountArithmeticVerdict`, `reports/t17-count-arithmetic-verdict.md`).** the
  bounds-checked `Cursor` (Err on truncation, never panics) · **`checked_block_len`** — every
  `count × element-size` term via `checked_mul`/`checked_add` → typed `Err` on overflow (no 32-bit wrap,
  no `overflow-checks` panic) · the **pre-allocation bound**: `checked_block_len ≤ cursor.remaining()`
  checked **before any `with_capacity`**, so a header claiming `timecnt = 1e9` with a tiny body is
  rejected *before* a multi-GB allocation (the headline DoS fix) · `Cursor::skip(checked_block_len)` for
  the v1-block skip · the **T17.1a** `type_index < typecnt` guard · `read_cstr`'s `desigidx` bound ·
  `overflow-checks = true` as a backstop only.
- **Tests / receipts / hashes.** `tzif::validate::tests::{implausibly_large_declared_count_is_rejected_not_ooming,
  count_block_len_is_checked_arithmetic, out_of_range_transition_type_index_is_rejected_not_panic}` · the
  `rfc9636` type-index-bounds violation test · `reports/t17-count-arithmetic-verdict.md` (the per-surface
  table).
- **Status.** **`guarded`** (T17.5) — count/size/offset arithmetic is checked before allocation, slicing,
  indexing, iteration, and cursor advance; hostile counts reject with a typed `Err`, not panic/wrap/OOM.
  (The *structural* indicator-count consistency / ascending invariants remain the separate T15.4 verdict
  axis — count-*arithmetic* safety ≠ RFC-structural conformance.)
- **Non-claim.** structural-valid ≠ resource-safe ≠ count-arithmetic-safe (three separate axes); T17.5
  guards the arithmetic, not the semantics.
- **Hardening owner.** T17.5 (shipped); structural count-relations stay with T15.4.
- **Boundary.** *Counts from input are hostile until range-checked — and now they are, before use.*

## RISK.REPORT.1 — report-as-attestation confusion

- **Failure mode.** a JSON report looks official and is treated as a signed certification.
- **Consequence.** someone treats a local report as signed certification · a report is copied without its
  build context · a stale report is used as proof.
- **Where it enters.** a `support-report` JSON shared as "the proof" · a report without its compiler /
  scope context.
- **Evidence surfaces.** `ReportProvenance = unsigned_local_report` (a report is **not** an attestation) ·
  `CompilerIdentity` (rustc/git honestly `null` — no `build.rs`) · `declared_scope_hash` (the claim
  envelope is a deterministic hash) · `ConformanceLevel` bounded (`release_admitted_compile_coverage`,
  never `compatible`/`conformant:true`).
- **Tests / receipts / hashes.** `tests/conformance_golden.rs` (the golden pins provenance =
  `unsigned_local_report`, no unbounded verdict) · the declared-scope-hash determinism test.
- **Status.** `guarded`.
- **Non-claim.** `does_not_claim_report_authenticity_without_signature_or_reproducible_context`.
- **Hardening owner.** T19 (signing only when real) · T20.
- **Boundary.** *A local unsigned report is evidence, not attestation.*

## RISK.SOURCE.1 — knowledge-index source rot / copyright mistake

- **Failure mode.** T18 turns into scraping/mirroring rather than provenance, or treats archive links as
  authority, or weak sources as primary.
- **Consequence.** copyright violation · mirrored proprietary content · stale links · archive links
  treated as authority · forum posts treated as primary evidence.
- **Where it enters.** the T18 knowledge-index build step (network) · promoting an archive URL to primary
  · admitting a source without recording rights posture.
- **Evidence surfaces.** the **T18 provenance doctrine** (plan T18): typed `SourceLedgerEntry`,
  `AuthorityKind` evidence tiers, `canonical_url = authority` / `archive_url = preservation witness`,
  Wayback + archive.today capture *where lawful*, copyright boundary
  (`copyrighted_source_not_republished` / `link_only_or_existing_archive_only`).
- **Tests / receipts / hashes.** none yet — **T18 is not built**; the doctrine is recorded so the build
  is born compliant.
- **Status.** `doctrine` (recorded; T18 not yet implemented).
- **Non-claim.** *T18 preserves claim provenance, not copyrighted bodies of text.*
- **Hardening owner.** T18.
- **Boundary.** *Provenance is preserved; copyrighted bodies of text are not republished; an archive URL
  is a witness, not the authority.*

## RISK.HIST.1 — historical archaeology contaminating current claims

- **Failure mode.** old-OS `zic` archaeology (the parked `T16.HIST` campaign) leaks into the current
  vendor matrix, so historical behaviour is mistaken for current vendor behaviour.
- **Consequence.** historical `zic` behaviour mistaken for current · unsupported archive images weaken the
  provenance chain · old bugs become current claims.
- **Where it enters.** mixing historical receipts into the current matrix · admitting an image with weak
  checksum authority.
- **Evidence surfaces.** `T16.HIST` is **parked and separated** from the current matrix · a typed
  `receipt_epoch` {`current`/`historical`/`archival`} is reserved · `IMAGE-PROVENANCE.md` records
  checksum authority (published-vs-computed) per image.
- **Tests / receipts / hashes.** the current 19-receipt matrix is the only admitted set; no historical
  rows exist yet.
- **Status.** `doctrine` (separation recorded; `T16.HIST` not run).
- **Non-claim.** historical rows explain lineage; they do not establish current vendor parity.
- **Hardening owner.** T16.HIST (if/when run).
- **Boundary.** *Historical rows explain lineage; they do not establish current vendor parity.*

## RISK.ADOPT.1 — premature replacement / adoption misuse

- **Failure mode.** zic-rs is used as a primary system-`zic` replacement before its install ecology is
  admitted, or a consumer assumes full reference parity / CLDR-ICU API / a measured resource profile it
  never had.
- **Consequence.** a primary `zic` replacement before install ecology is admitted · a packager assumes
  full reference parity · a runtime assumes CLDR/ICU API compatibility · an embedded consumer assumes a
  resource profile that was never measured.
- **Where it enters.** skipping the staged adoption path · reading CORE.1 (behaviour parity over a
  horizon) as full operational parity.
- **Evidence surfaces.** the **staged adoption** model (T19: verification-only → side-by-side → optional
  package mode → replacement candidate → default) · `negative_capabilities` (typed, guard-enforced) · the
  four-bucket `differences-from-reference-zic.md` map · the precise live claim (CORE.1 over 1900..2040,
  *not* full replacement).
- **Tests / receipts / hashes.** the per-milestone receipts each bound their own claim; `negative_capabilities`
  has a test asserting each entry maps to an enforced guard.
- **Status.** `partial` / `doctrine` — the staging + non-claims are recorded; the persona-specific
  "may conclude / may not conclude" packets are T20; the install-ecology admission is bounded
  (`compile_output_tree_only`).
- **Non-claim.** `does_not_claim_reference_install_directory_layout_without_a_REDO_layout_witness` ·
  not-a-runtime-`localtime`-library · not-a-`tzselect`/CLDR replacement.
- **Hardening owner.** T19 (adoption staging + `TRUST.md`) · T20 (per-persona conclusions) · T21
  (embedded resource profile).
- **Boundary.** *zic-rs is replacement-grade only for the surfaces it has admitted.*

---

## The claim-boundary summary (the armor in one screen)

The dangerous failures are claim-boundary bugs, not (first) memory bugs. For each, zic-rs either proves
the claim with **typed evidence + receipts + hashes + tests**, or refuses it with an **explicit
non-claim**:

| Risk | One-line boundary | Status |
|---|---|---|
| RISK.TIME.1 | compiles admitted tzdb source; does not determine civil-time truth | guarded |
| RISK.TZIF.1 | structural validity ≠ semantic parity | guarded |
| RISK.LEAP.1 | leap records are explicit evidence; no smear / no future-leap truth | guarded (opt-in) |
| RISK.VENDOR.1 | a receipt admits one ecology, not a family theorem | guarded |
| RISK.DIAG.1 | rejection parity ≠ diagnostic-class parity | guarded |
| RISK.ORACLE.1 | an oracle result needs known identity + input path | guarded |
| RISK.DIFF.1 | unassessed behaviour ≠ unchanged behaviour | guarded |
| RISK.PATH.1 | compile correctness ≠ hostile-concurrency install safety | partial — **leaf races closed (T17.4)**; parent-component swap still open |
| RISK.INSTALL.1 | each file durably published; the whole tree is not one crash-atomic transaction | partial — **per-file durable publish guarded (T17.4)**; whole-tree atomicity not claimed |
| RISK.RESOURCE.1 | a cap breach is an operational refusal, not a grammar diagnostic | guarded |
| RISK.COUNT.1 | input counts are hostile until range-checked — and now are, before use | **guarded (T17.5)** |
| RISK.REPORT.1 | a local unsigned report is evidence, not attestation | guarded |
| RISK.SOURCE.1 | provenance preserved; copyrighted text not republished | doctrine (T18) |
| RISK.HIST.1 | historical rows explain lineage, not current parity | doctrine |
| RISK.ADOPT.1 | replacement-grade only for admitted surfaces | partial / doctrine |

> **The doctrine sentence, restated:** *zic-rs treats every serious failure mode as a claim-boundary
> problem first — prove the claim with typed evidence, receipts, hashes, and tests, or refuse it
> explicitly.* This register is where each refusal or proof is recorded; it grows (never shrinks) as
> milestones land.
