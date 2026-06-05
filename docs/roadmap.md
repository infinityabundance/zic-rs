# Roadmap

`zic-rs` grows **strictly by fixture class**: a new construct is added only when a fixture
(and its reference-`zic` oracle result) forces it. Each milestone ends green — build,
clippy, tests, and oracle all passing — so the project never sits in a half-working state.

## Honest scope (where we are vs "full drop-in `zic`")

The **civil-time compiler core** is essentially complete: every compile-supported canonical zone
(**341 / 341**) behaviour-matches reference `zic`/`zdump` over `1900..2040` (0 mismatch, **0
fail-closed**). Law 7 (inline negative SAVE → Prague) and law 10 (non-POSIX `ON` + v3 footer →
Gaza/Hebron) are both done — **the canonical-zone behaviour frontier is closed (341/341)**. The
the remaining work is the rest of the **operational shell** — not a canonical-zone behaviour gap.
Much of that shell has since **landed**: **T9** (CLI / filesystem / install: `-d`/`-D`/`-l`/`-t`/`-m`,
exit-status + no-partial-install), **T10** (range/emission `-b`/`-R`/`-r`, all `-r` profiles 341/341),
and **T11** (leap / `right/` / TZif v4, opt-in `-L`) are **done**. **T12 (build-profile provenance —
manifest `zic-rs-compile-manifest-v8`) is CLOSED** (`reports/t12-close-receipt.md`; the full
source-variant evidence arc T12.5a–d shipped + the all-IANA release-admission matrix T12.5a.3).
**T13 (warning/diagnostic parity) is CLOSED** (`reports/t13-close-receipt.md`; the executable diagnostic
contract `ZIC001`–`ZIC020`). **T14 (hostile-input/parser-edge parity) is CLOSED**
(`reports/t14-close-receipt.md`; `ZIC021`–`ZIC025`, incl. a removed panic on untrusted input).
**T15 (`support-report` as the public conformance engine) is CLOSED** (`reports/t15-close-receipt.md`):
T15.1–T15.4 (schema inventory · typed `OracleMode`+`negative_capabilities` · `zdump`-backed semantic
witnesses + `ArtifactCategory` · RFC 9636 TZif structural validator) and T15.5 + remainder
(`ConformanceStatus` rollup + `declared_scope_hash` + report-as-artifact provenance + the typed
claim-shape axes [`ReferencePinGate`/`ClaimPortability`/`EvidenceAuthorityKind`/`ClaimBoundary`/
`valid_disambiguation`] + `ZIC026`) — the public engine with 15 guard-enforced non-claims and a bounded
`ConformanceStatus` (no global verdict). The still-open declared work is the **T16–T21** ecology /
reliability / trust / security / embedded layers. **T16 (release ecology) is IN PROGRESS** — T16.1–T16.6
sealed (inventory · `ReferenceBuildProfile` · `ReferenceLocatorKind`+`SignatureTrustModel` · auxiliary-table
validator · vendor-oracle receipt admission + no-dep receipt ingestion · release-diff + doctor · **17 ecology rows / 19 receipts (18 admitted + 1 typed non-admission)** from the external lab — the full **BSD sweep** (FreeBSD 2022g · OpenBSD
old-fork · NetBSD 2022g · DragonFly oldest-fork) **+ illumos triad** (OmniOS · SmartOS · OpenIndiana, all 2025a)
**+ Linux** (Alpine/musl tzcode 2026b · Debian/NixOS/Ubuntu/AlmaLinux glibc · Gentoo/tzcode source-built ·
openSUSE/tzcode-via-`timezone`-RPM · SLES/commercial-SUSE-container · Arch/`tzdata`-pkg-tzcode) **+ Yocto/Poky** (build-host `tzcode-native` 2026b admitted vs target-runtime consumer non-admission); each
admitted row rejects every fixture (safe), 3–4/5 class+location match + declared divergences; measured a real
vendor lag spectrum, an old-fork input-compat gap [DragonFly's `zic` can't ingest the zishrink 2026b
`tzdata.zi`], complete illumos convergence on three independent builds, that the C library (musl vs glibc)
doesn't change `zic` diagnostics, **two `zic` implementation lineages — IANA tzcode `zic` vs glibc's own `zic`,
the glibc lineage version-stratified with a bracketed inflection** (glibc 2.34/2.39 old-fork-like + fat <
2.40/2.41 modern + slim, transition between 2.39 and 2.40), that **the `zic` lineage is a distro packaging
choice independent of libc AND package format** (Gentoo tzcode-`zic` via `sys-libs/timezone-data` on glibc 2.42;
openSUSE tzcode-`zic` via the `timezone` RPM where AlmaLinux's RPM gives glibc-`zic` → **RPM ≠ glibc-`zic`**; 5
packaging models), that **bloat-default is its own packaging axis** (fat in old-glibc + Gentoo-tzcode; slim in
Alpine/openSUSE-tzcode + modern-glibc), that **some artifacts ship no on-device `zic`** (the Yocto/Poky target image = TZif consumer — precompiled data, compiler off-device), and that **the embedded build-host *producer* ≠
the target-runtime *consumer*** (Poky's `tzcode-native` owns `zic`; the produced image ships only data); the two
non-admissions prove `vendor-oracle-admit` rejects-by-rule (negative results as first-class); pinned via
`pkg`/`apk`/`dpkg`/`rpm`/`qfile`/store-path-hash/`tzdata.zi`/byte-witness/recipe-SRC_URI-sha;
image-provenance ledger in the lab); **campaign breadth reached (17 ecology rows / 19 receipts across BSD ·
illumos · Linux · embedded-runtime · embedded-build-system)**. **T16.6 ✅ sealed** — the operator-facing layer:
`release-diff` (per-identifier OLD↔NEW classification — typed `ReleaseChangeKind`, structural axis always +
behavioural axis via two `zdump` year-windows split at a declared year, oracle absence ⇒ `behaviour_unassessed`
never "no change"; `zic-rs-release-diff-v1`) and `doctor` (read-only host probe — `zic`/`zdump`/`tzdata`
`ToolStatus`, always exit 0; `zic-rs-doctor-v2` since T17.3); both additive/read-only. **T17 (Rust
reliability hardening) is CLOSED** (`reports/t17-close-receipt.md`) — T17.1a parse bounds-guard (transition `type_index < typecnt` at
the decode choke point → typed rejection, not a latent OOB panic) + `docs/panic-policy.md` · T17.1b
`limits::ResourceLimits` (input-dimension caps as bucket-3 `Error::config`, not `ZIC###`) · T17.2
CONTRACT.TYPING (6 manifest/alias-map free strings → typed enums, JSON byte-identical, no schema bump;
`reports/contract-typing-audit.md`) · T17.3 doctor/release-diff failure taxonomy (`ToolVersionStatus`,
`HashReadStatus` → doctor **v2**; `OracleFailureScope` global-vs-row; the `--split` year is an exclusive
seam, behaviour change in the split year is `behavior_future`, never double-counted) · T17.4
install/materialization hardening (per-file crash-durable publish [content fsync + atomic publish +
parent-dir fsync, Unix] · leaf TOCTOU races closed [exclusive-create + atomic symlink overwrite];
`docs/install-materialization-contract.md`; whole-tree atomicity + parent-component swap race explicitly
**not** claimed) · T17.5 `CountArithmeticVerdict` (checked `count×element-size` arithmetic + the
pre-allocation bound that rejects an implausible declared count before any `with_capacity`;
`reports/t17-count-arithmetic-verdict.md`) · T17.6 schema/CLI stability policy
(`docs/schema-compatibility-policy.md` + `docs/cli-compatibility-policy.md` — schema surfaces classified
[10 at T17.6; **11 since T21.2 added `zic-rs-size-report-v1`**] + every command classified
gate/diagnosis/witness/admission/convenience; policy only, no behaviour change).
· T17.FUZZ receipt-bearing fuzz harness (`fuzz/`; sat `pending_capture` at T17 close, **later executed at T23.cargo-fuzz.1/.2** — bounded smoke found+fixed F1–F3) · T17.7
verify-here remainder (`tests/reliability.rs` · `SECURITY.md`/`CONTRIBUTING.md`/`CHANGELOG.md`/`RELEASE.md`
· `schemas/` + the registry/drift test). **480 tests, CORE.1 341/0/0. T17 CLOSED.** **T18 (knowledge index) — first cut (T18.1) shipped:**
`docs/zic-knowledge-index.md` + `source-ledger.tsv`/`archive-ledger.tsv` + `claim-source-map.md` +
`copyright-non-republication-policy.md` (12 impeccable sources; typed `SourceLedgerEntry`/`AuthorityKind`;
`canonical_url`=authority / `archive_url`=witness; own evidence labelled `reproducible_lab_evidence`, not
external authority; **archival `pending_capture`** — no fetch in a no-network env, no URL fabricated;
breadth→40 + the capture pass continue). **T19 (trust front door) — SEALED:** 8 docs, docs-only —
`TRUST.md` (the evidence-court entrypoint; §5 evidence-map · §6 the exact non-claims) +
`docs/effect-boundary-map.md` (per-module effect class, grep-audited) + `docs/not-yet-ready.md` +
`docs/replacement-readiness-ladder.md` (RRL-0…6; zic-rs at **RRL-1→2**) +
`docs/drop-in-compatibility-contract.md` (**NOT an argv drop-in by design**) +
`docs/misuse-resistance-ledger.md` + `docs/audit-readiness.md` (the packet, not a performed audit) +
`docs/maintenance-policy.md`; **480 tests, CORE.1 341/0/0**. **T20 (security rewrite-evaluation packet) —
SEALED:** `docs/security-rewrite-evaluation.md` (threat table · resource model · supply-chain [SBOM/SLSA/
signing **planned**, not present] · Scorecard-readiness · the honest rewrite-safety delta — *Rust removes
the memory/arithmetic/buffer class by construction; everything else is earned by an oracle check, a typed
contract, a bounded cap, or a named non-claim*) + `docs/security-personas.md` (10 personas × fears/evidence/
commands/may-conclude/**may-NOT-conclude**/risks/owner; consumes risk-register + SECURITY.md + T19);
**480 tests, CORE.1 341/0/0** (at the T20 seal). **T21 (embedded/container builder profile) — first cut:**
`docs/container-embedded-builder.md` (bundle contract · container recipe · no-host-contamination · copy-mode
baseline · producer≠consumer) + **`zic-rs size-report`** (read-only footprint + deterministic `bundle_hash`;
`zic-rs-size-report-v1`, `src/size_report.rs`, `tests/size_report.rs`); named bundle-profiles/`--link-policy`/
reader-gauntlet remain planned. **485 tests, CORE.1 341/0/0.** **T22 (performance/resource ledger) — first
cut:** `docs/perf-ledger.md` + `reports/perf/measure.py` + a **real measured receipt**
(`reports/perf/RECEIPT-x86_64-2026-06-02.md`): `tzdata.zi` 107,524 B → 598 TZif files / 435,761 B in ≈12 ms,
peak RSS ≈22 MiB — far inside the `ResourceLimits` caps; **measured + bounded, NOT a speed claim** (the
deterministic anchors are the regression contract; wall/RSS are host/run weather). **T23 (audit suite) —
first cut:** `audits/` built to the **exact authoritative list** (17 tool folders + README + index.html,
each receipt-bearing; *a folder existing is not a result*). **An operator batch ran 6 real audits
(2026-06-02):** `cargo-audit` ✅ clean (0 vulns / 67 deps) · `cargo-geiger` ✅ (zic-rs own 0 unsafe; dep
surface mapped) · `cargo-auditable` ✅ · `miri` ✅ clean over the `tzif` core (0 UB) · `panic-analysis` ✅ ·
`dsfb-gray` ✅, **`kani` ✅ — 10 bounded formal proofs (T23.kani.1 count-arithmetic guard + T23.kani.2 `Cursor` + T23.kani.3a `type_index<typecnt` + T23.kani.3b abbr-index/`charcnt` guards + T23.kani.3f.1–.4 four standards-precision predicates), and `cargo-fuzz` ✅ T23.cargo-fuzz.1/.2 (bounded smoke found+fixed F1–F3);
take/skip/≤-remaining bounds, all 0-fail; the count/cursor layer is now formally proven panic-/wrap-free and
the pre-allocation bound sound)**; `cargo-vet`/`cargo-valgrind` ◐ partial/inconclusive (uninitialised store ·
valgrind env-incompatible, SIGILLs on `/bin/true`); kani's broader full-parser harness (T23.kani.3) ◐ timed out;
rest honest `pending` — **no status upgraded without a receipt; not all green**. The maintenance-skeleton arc
**T17→T23 is laid down**, now with a formal-proof receipt at its tip.
**NEXT (on demand):** widen T23.kani.3 (full-parser entrypoint — larger solver budget) · `cargo-vet init` +
imports · the reader gauntlet · T18 breadth→40 · a real 2026a→2026b release-diff delta. The six-axis vendor-oracle matrix
renderer stays T16.6.x (on demand, no schema bump). (Diagnostic contract now `ZIC001`–`ZIC026`.)

"Full drop-in `zic` across *all modes*" is a larger, mostly **operational** surface (TZif
emission/reader-compat policy, CLI/filesystem/install options, leap/`right/`/`posix`/`-r`/`-R`/`-b`
modes, warning parity, multi-horizon reporting). With T9–T15 landed/closed and T16 next, a
**conservative rough self-estimate** of that surface is now **≈ 85 %** (up from an earlier ≈ 60–70 %) — but this is an
**estimate, not a measured claim**: the *measured* claim remains exactly CORE.1 (341/341 over
1900..2040) plus the structural/operational receipts each milestone shipped. The operational shell is a
**declared roadmap**, and the remaining warning/hostile-input/conformance/ecology layers are real work;
zic-rs **does not claim to be a full `zic` replacement** (see **T9–T21** in the plan and the named
campaigns in [zic-hidden-compatibility.md](zic-hidden-compatibility.md)).

## T1 — Skeleton + fixed-offset compiler ✅ (done)

* Library-first crate (`tzcompile` + `zic-rs` binary), `#![forbid(unsafe_code)]`.
* Full lexer/parser for `Rule`/`Zone`/`Link` + continuations (parse, not yet compile rules).
* In-house calendar/time primitives, unit-tested.
* TZif v2 writer (v1 stub + v2 block + footer), RFC 9636.
* Safe output tree (traversal-proof, atomic, no clobber).
* Semantic + byte oracle against reference `zic`.
* **Verified:** `Etc/UTC`, `Test/Fixed` byte-identical to reference `zic`.

## T2 — Transition compiler for finite DST ✅ (done)

* `compile::transitions`: expand finite `Rule` sets across their year span.
* Wall/standard/UT (`w`/`s`/`u`) → UT instant conversion using the **prevailing** offset
  (the classic `zic` subtlety).
* `compile::abbreviations`: `%s`, `STD/DST` slash, `%z`, literal.
* Fixed POSIX footer for the steady post-rules tail.
* **Verified:** `Test/Simple`, `Test/Euro` (`lastSun`, UT `AT`), `Test/Sle` (`Sun<=25`,
  standard `AT`) `zdump`-match reference `zic`. Recurring (`max`) rules fail closed.
* Tests: `calendar_rules`, `transition_generation`.

## T2.1 — Honest oracle + real `zdump` compare mode ✅ (done)

* `compare --mode {zdump|structural} --horizon LO,HI`, default `zdump`; `compare::zdump`
  runs `zdump -v -c` on absolute paths and diffs behaviour.
* Renamed the in-tool decoded comparison honestly ("decoded TZif match" vs "zdump behaviour
  match over LO..HI") — no overclaiming.

## T3 — Recurring (`max`) POSIX footer ✅ (done)

* `compile::posix_footer::recurring`, exact-or-fail (e.g. `EST5EDT,M3.2.0,M11.1.0`): default
  +1h DST offset omitted, default `/02:00` omitted, `Sun>=N`→`(N-1)/7+1`, `lastSun`→last,
  `AT`→local-wall conversion. Non-expressible day forms (`Sun<=N`) fail closed.
* **Verified:** `Test/Eastern` matches reference `zic` under the `zdump` oracle over
  `2019..2099`. Footer-string unit test asserts the exact string.

## T3.1 — Multi-era `UNTIL` stitching ✅ (done)

* Era walker (`compile::transitions::compile_multi_era`) carrying cross-era state; `UNTIL`
  converted in the ending era's context with the prevailing save; footer from the final era;
  boundary transitions de-duplicated only on true no-ops.
* **Final recurring era footer anchoring**: a final `TO=maximum` era emits a forced
  era-start anchor transition + the recurring footer (no fat per-year expansion); the footer
  projects from the anchor. `Rule FROM` semantics are not changed globally.
* **Verified** (`zdump` oracle): `Test/FF` (fixed→fixed, byte-identical), `Test/MidDst`
  (ends mid-DST), `Test/FR`/`Test/Q` (fixed→recurring). Pinned in
  [reference-zic-semantics.md](reference-zic-semantics.md).

## T3.2c — Effective-in-era final-era classification ✅ (done)

* A final era is classified by its **effective in-era** rule activations, not raw rule-set
  membership, into two **supported** shapes: (a) finite rows all predate the era → recurring-only
  → anchor + footer (Europe/London); (b) finite rows still fire in-era → expand the finite
  history explicitly + recurring footer tail (America/New_York; the T4.1 enabler).
* **Verified** (`zdump` oracle): `Test/FinalEffective` (`1990..2040`), `Test/Mixed`
  (`1976..1997`), `Test/MixedInEra` (`1970..2040`). Tests
  `final_era_ignores_pre_era_finite_rows_when_classifying_recurring_tail`,
  `single_era_mixed_historical_rules_are_checked_before_footer_tail`,
  `mixed_in_era_finite_and_recurring_final_era_is_supported`.
* **Slim/fat note:** for recurring zones zic-rs emits a *fat* explicit set vs `zic`'s *slim*
  one (e.g. `Test/Mixed` 122 vs 39); behaviour is identical (`zdump`) and the footer matches.
  Matching `zic`'s slimming heuristic is a separate byte-parity goal (below). See
  [reference-zic-semantics.md](reference-zic-semantics.md) §6.

## T3.2d — Native zishrink record-key parsing ✅ (done)

* Record keywords now match as `zic`-style **unambiguous prefixes** of `Rule`/`Zone`/`Link`
  (`src/source/parser.rs::record_keyword`), so the installed single-file
  `/usr/share/zoneinfo/tzdata.zi` (which uses `R`/`Z`/`L` plus already-supported prefixed
  month/weekday/year tokens) **parses directly**. Pinned empirically: `zic`'s zone-file keyword
  table is exactly `{Rule, Zone, Link}` — `Leap`/`Expires` are "unknown type" in a zone file —
  so a bare `L` is unambiguously `Link`. Tests: `record_keyword_matches_zic_style_unambiguous_prefixes`,
  `parses_zishrink_abbreviated_record_keys`, `zic_rs_compiles_abbreviated_and_canonical_london_identically`.

## T3.2a — Obsolete `FROM = minimum` ✅ (done)

* `minimum` is coerced to **1900** at parse time, exactly as reference `zic` (which warns
  "obsolete; treated as 1900"); year keywords match as unambiguous prefixes (`mi`/`ma`/`o`,
  bare `m` rejected). Compatibility breadth only — tzdata 2026b uses it zero times. Verified by
  `Test/MinRule` (`zdump` over 1899..1910 and 2019..2040). Warning-surfacing is a tracked TODO.

## T3.2b — Inline-save eras ✅ (done)

* A `Zone` era whose `RULES` is a clock value (e.g. `0:30`) compiles to one fixed type at
  `STDOFF + SAVE`, `is_dst` set, abbreviation from a **literal** or **`%z`** `FORMAT` (`%z` over
  the total offset). Real-data-forced (Hong Kong / Jakarta wartime). ttinfo **byte-identical** to
  reference `zic`; `zdump` match for `Test/InlineLit`/`InlineZ`/`InlineSolo`. **Fail closed** on
  inline-save `%s`/slash/negative. Single-era path: `compile::compile_inline_save`; multi-era:
  `EraKind::InlineSave` in `transitions.rs`.

## T4.1 — Second real IANA slice: `America/New_York` ✅ (done)

* `fixtures/iana-slices/america_new_york_2026b.zi` (+ `.abbrev.zi` sibling), byte-identical
  under reference `zic`. Forced and exercises the **genuinely mixed-in-era** final era (Rule US
  finite DST `1967..2006` + recurring `2007..max`) plus the WWII `W`/`P` war-time eras. 6 eras,
  footer `EST5EDT,M3.2.0,M11.1.0`; `zdump` match over `1883..2040`.
  (`america_new_york_2026b_matches_reference_zic_over_1883_2040`.)

## `support-report` — the frontier map ✅ (done)

* `zic-rs support-report --input <file> [--format text|json]` compiles **every** zone in a
  source file and buckets the outcome (canonical zones vs links accounted separately; every zone
  in exactly one bucket; `supported + Σ unsupported == parsed`; catch-all `other` keeps the raw
  diagnostic). Reports *compile* support, not behavioural correctness (the oracle's job). Core in
  `src/report.rs`; shared JSON escaper in `src/json.rs`. See
  [conformance-ladder.md](conformance-ladder.md) for the doctrine + per-slice receipts
  ([`fixtures/iana-slices/SLICES.toml`](../fixtures/iana-slices/SLICES.toml)).
* **Frontier finding → `%z` unlock ✅** (installed `tzdata.zi` 2026b): the biggest bucket was
  **176 zones** blocked by a no-rules era with a **`%z` FORMAT** (`0:30 - %z`). Implementing it
  (render the era's numeric `STDOFF`; `%s`/slash stay fail-closed) took compile-clean coverage
  **162 → 338 / 341**; law 7 (negative SAVE) → **339**; law 10 (non-POSIX `ON` + v3 footer) →
  **341 / 341** (594 / 598 identifiers). **No canonical zone fails closed.**
* **Honest caveat (compile-clean ≠ behaviour-verified):** a **comprehensive** `zdump` sweep over
  **all 341** canonical zones (`1900..2040`) reports **341 match / 0 mismatch / 0 fail-closed**
  (341 + 0 = 341 compile-clean), *after T5 #1–#5 + law 7 + law 10*: **every canonical zone
  behaviour-matches reference `zic` over `1900..2040`** — the canonical-zone behaviour frontier is
  closed. The match count rose **264 → 328 → 333 → 338 → 339 → 341**; the old 35-zone *sample*
  ("28 / 6") is retired. **T5 #1–#5 + law 7 + law 10 fixed.** See
  [reference-zic-semantics.md](reference-zic-semantics.md) §10.

## T5 — ActiveRuleState correctness pass (highest priority — correctness over breadth)

* ✅ **#1 standard-`LETTER` temporal state** — `%s` renders the *active* rule letter, and a
  `SAVE=0` rule can change it. Fixed generically: a rule activating at-or-before an era boundary
  (`ut <= s`) seeds the boundary state (`Pacific/Auckland` NZMT→NZST now matches; no NZ
  special-casing). Test `pacific_auckland_1946_changes_nzmt_to_nzst_and_matches_reference_zic`.
* ✅ **#2 final-recurring-era boundary-coincident anchor** — the shape-(a) short-circuit emitted its
  era-start anchor from the era's *standard* seed, skipping the `ut <= s` absorption. A rule that
  springs/falls exactly at the era start (`Europe/Lisbon` 1996-03-31 01:00u = Rule E spring →
  WEST, not WET) is now seeded into the anchor. Cleared **`Europe/Lisbon` + `America/St_Johns`**
  (the oracle disproved the earlier "Lisbon = Buenos_Aires" grouping — they are different bugs).
  Tests `europe_lisbon_final_era_boundary_coincident_dst_matches_reference_zic`,
  `america_st_johns_matches_reference_zic_after_anchor_fix`.
* ✅ **#3 boundary-coincident standard-clock activation absorption** — a rule activation naming the
  *same standard-clock instant* as an era boundary must seed the boundary, but the general path only
  absorbed it when the converted `ut <= s`. When the new era's stdoff is 1h smaller, `ut` overshoots
  `s` (boundary uses the old, larger stdoff) and a spurious standard hour was inserted
  (`Europe/Moscow` 1991: `3 R MSK/MSD` → `2 R EE%sT`, both "Mar lastSun 02:00s"). Fixed by carrying
  the previous era's UNTIL as a naive-local + clock-ref (`boundary_until`) and absorbing on **exact**
  clock-frame equality (no tolerance). Cleared **64** zones (74 → 10). Tests
  `era_boundary_absorbs_new_era_standard_clock_spring_forward` (hermetic),
  `europe_moscow_1991_matches_reference_zic`, `europe_volgograd_boundary_spring_forward_matches_reference_zic`,
  `asia_novosibirsk_1991_matches_reference_zic`, `russia_boundary_offset_drop_does_not_create_spurious_standard_hour`.
* ✅ **#4 prior-active rule-state seeding** — a named rule set has its own timeline; entering a later
  era that uses it does **not** reset the rule state to standard. A ruled era now seeds from the rule
  set's most-recent activation at-or-before the era start (scanning the rule's actual `FROM` range,
  not `start_year−1`), carrying `save` + `LETTER` together. `America/Phoenix` 1944 starts in War time
  (MWT) because Rule US set War in 1942; `Atlantic/Bermuda` 1930 inherits `LETTER=S` (AST) from 1918.
  Cleared **5** (Phoenix, Bermuda, Manila, Brussels, Macau). Tests
  `ruled_era_inherits_prior_active_rule_state` (hermetic `prior_active_carry.zi`),
  `america_phoenix_1944_inherits_war_time_matches_reference_zic`,
  `atlantic_bermuda_1930_inherits_standard_letter_matches_reference_zic`,
  `asia_manila_and_europe_brussels_inherit_prior_dst_match_reference_zic`.
* ✅ **#5 clock-reference normalization at era boundaries** — `Rule AT` and `Zone UNTIL` may use
  wall/standard/universal references; boundary decisions must compare **resolved instants**, not raw
  local-second fields. Two sub-sites: the era-end break now compares `ut >= until_ut` (not naive
  local — `Europe/Simferopol` universal spring after a wall UNTIL), and the boundary-coincidence
  test normalizes refs under the prevailing save (wall ≡ standard at save 0 — `Asia/Tashkent`/
  `Ashgabat`/`Anadyr`; also fixes `Europe/Warsaw`). Cleared the **last 5**. Tests
  `era_boundary_normalizes_wall_standard_clock_reference` (hermetic `boundary_clockref.zi`),
  `central_asia_offset_drop_boundaries_normalize_clock_reference`,
  `europe_mixed_reference_era_end_breaks_match_reference_zic`.
* ✅ **law 7 — inline negative SAVE (signed effective offset).** Lifted the defensive guard; the
  existing inline-save path renders `STDOFF + SAVE` with `is_dst = (save != 0)` — `Europe/Prague`'s
  `1 -1 GMT` → GMT, isdst=1, gmtoff 0, zdump-matches reference `zic`. Cleared **1** (Prague). Tests
  `inline_save_negative_renders_signed_effective_offset`,
  `negative_inline_save_is_signed_state_matches_reference_zic`.
* ✅ **law 10 — non-POSIX `ON` day form + content-driven v3 footer.** `Asia/Gaza`/`Asia/Hebron`'s
  perpetual `Sat<=30` rule: `zic` re-anchors a `weekday<=N` form onto a clean nth-weekday and folds
  the skipped days into the transition time (`Sat<=30 02:00` → `M3.4.4/50`, 4th Thursday + 50h),
  needing a **v3** footer. Implemented the exact `stringrule` conversion in `posix_footer::date_rule`
  + content-driven v3 (`recurring` returns the version) + extended the explicit horizon to the last
  finite one-shot rule year (Ramadan rows to 2086 are emitted explicitly; footer covers the perpetual
  tail). Cleared **2** (Gaza/Hebron); synthetic `Sun<=25` → `M10.3.3/98` is byte-identical to ref.
  Tests `recurring_sat_leq_30_uses_v3_extended_time_footer`, `recurring_sun_leq_25_uses_extended_v3_footer`,
  `neighboring_month_on_form_matches_reference_zic`.
* **🎯 Result: 341/341 — every canonical zone behaviour-matches reference `zic` over `1900..2040`;
  0 mismatch; 0 fail-closed.** The canonical-zone behaviour frontier is closed. (Remaining: the
  **T16–T21** ecology/reliability/trust shell — *not* a canonical-zone behaviour gap; **not** full-`zic`.
  T9–T15 closed; T16 next.)
* ▷ A `support-report --verify` mode to fold the comprehensive sweep into the tool (compile-clean →
  behaviour-match counts), and `--explain-buckets` mapping each bucket to a deep zic semantic.

## T8 — Structural TZif parity ✅ (inventory ✅ · T8-v3 ✅ · T8-abbrev ✅ · T8-slim ✅)
The **measurement** step of the operational ladder: quantify how far the *writer* is from reference
`zic` before CLI/leap/range modes. Shipped the **`structural-report`** command (`src/structural.rs`,
`tests/structural_report.rs`) — compiles every canonical zone both ways, decodes both, and classifies
the byte difference into a fixed taxonomy. Full write-up: [structural-parity.md](structural-parity.md).
**Findings (tzdata.zi 2026b vs tzcode 2026b, all 341 zones, 0 errors):**
* `isutcnt`/`isstdcnt`/`leapcnt`/`typecnt` — **full parity** (so `ttisstd`/`ttisut` is *not* a gap:
  both emit `0` in this slim-default build; the earlier T8.1 hypothesis is **retired by measurement**).
* version + footer match for **341/341** (after T8-v3, below); net **+~4765** explicit transitions
  (slim/fat).
* **T8-v3 ✅ DONE** — version byte, pinned to `zic.c` (tzcode 2026b): `version = compat < 2013 ? '2'
  : '3'`, and `stringrule` raises `compat = 2013` exactly when a `weekday>=N`/`weekday<=N` re-anchor
  needs a **non-zero day-shift** (`wdayoff != 0`) OR the folded time is **negative**; a folded time
  `>= 24h` *alone* is only `compat = 1994` → still v2 (`Africa/Cairo`'s `lastThu 24:00`). zic-rs keys
  the version on `*_shift != 0 || *_tod < 0` (the shift already computed by `date_rule`). Closed the
  last two version outliers (`America/Santiago`, `Pacific/Easter`) → **version+footer 341/341**, no
  other zone flipped, full sweep stays 341/0/0. Tests `recurring_dayshift_forces_v3_even_when_time_in_range`,
  `recurring_last_weekday_24h_stays_v2`. The earlier naïve `tod ≥ 24h` attempt was tried and
  **rejected** (regressed Cairo) — fixed by reading the source, not guessing.
* **T8-abbrev ✅ DONE** — abbreviation **suffix-sharing** designation packer, ported verbatim from
  `zic.c::addabbr` into `tzif::data_block::build_designations` (two-pass: build the shared table —
  suffix-reuse / splice / append — then resolve each offset; `charcnt` is order-independent so it
  matches `zic` regardless of type order). `HST`⊂`AHST`, `LMT`⊂`PLMT`. Closed `Asia/Ho_Chi_Minh` +
  `America/Adak` → **`charcnt` parity 341/341**; structural `abbreviation-table` + `mixed` classes
  empty; behaviour unchanged (full sweep **341/0/0**; both zones zdump-identical). +5 packer unit
  tests; **186 default tests**.
* **T8-slim ✅ DONE** — `--emit-style {default|zic-slim|zic-fat}` (`EmitStyle`, threaded through
  `compile_zone_styled`). `zic-slim` reproduces slim `zic` by truncating the footer-governed recurring
  tail at `TZstarttime` (per-transition `from_recurring` flag + `nonTZlimtime`, so far-future *finite*
  one-shot rows like Gaza's Ramadan dates are kept) and pruning unused types; the `Default` path is
  never entered by the truncation (CORE.1-safe). Result under `--emit-style zic-slim`:
  `slim/fat-timecnt` collapses **75 → 1** (only `Europe/Lisbon`, ref-fatter-by-1 no-op),
  `byte-identical` **130 → 141**; **behaviour 341/0/0 in both modes**. `zic-fat` aliases default.
  Tests `tests/emit_style.rs`; **190 default tests**.
* **Structural parity is now closed** (all three sub-campaigns done): default = behaviour-matched fat
  (CORE.1-gated); `zic-slim` = byte-close to reference (1 enumerated residual).
* **Doctrine:** behaviour parity (CORE.1) stays the contract; structural parity is *measured*;
  byte parity only where pinned / in `zic-slim`; the **default** emission is never changed.

## T3.2 — Remaining breadth (next, ordered by impact)
* `24:00`+ and negative `AT` times in compiled output; content-driven v3 where required.
* Negative SAVE (parsing already supported); configurable transition-limit (`ZIC009`,
  `MAX_TRANSITIONS` enforced; wire `--transition-limit`).
* Per-fixture byte-parity mode in `compare` (would also drive `zic`-style slim emission).
* Inline-save `%s`/slash/negative (only if a real zone forces them).

## T3.5 — Link/canonical correctness + `explain` evidence trace ✅ (done)

* `resolve_link_target` detects **cycles** precisely (reports the path, e.g. `B -> A -> B`)
  and **missing targets** distinctly; `explain` surfaces both as diagnostics instead of a
  generic "no such zone". Tests: alias/chain/cycle/missing in `tests/explain.rs`.
* `explain` is now an **evidence trace**: decoded source eras (STDOFF/RULES/FORMAT/UNTIL),
  the compiled local-time-types, transitions with UT instants decoded to readable timestamps
  + the type each switches to (capped for recurring zones), the footer, and a note surfacing
  any equal-`utoff` distinct-type pair (the kept-boundary trap). Reports link aliases too.

## T3.3 — Reference-semantics ledger (later) · T3.4 — Release provenance (partly shipped, see T3.4c)

Remaining design items: grow
[reference-zic-semantics.md](reference-zic-semantics.md) into a full ledger with `zic.c`
references; detect the tzdb release version + IANA URL for the provenance manifest; add
`docs/tzdb-governance.md` cross-links. (The provenance manifest itself shipped in T3.4c.)

## T3.4a–d — Consumer-integration lane (producer-side framing)

From the Rust-ecosystem review ([rust-ecosystem.md](rust-ecosystem.md),
[generated-data-contract.md](generated-data-contract.md)): zic-rs is the *producer* upstream
of consumers like tz-rs/tzdb/jiff.

* **T3.4a — positioning** ✅ (docs landed): `rust-ecosystem.md`, `generated-data-contract.md`,
  README producer-side paragraph.
* **T3.4b — alias/canonical manifest** ✅ (shipped): `compile --alias-map` →
  `alias-map.json` (`zic-rs-alias-map-v1`): per-identifier `{kind: zone|link, target?,
  sha256}` + `{identifiers, canonical_zones, links, duplicated_byte_links}` — answers
  jiff#258's alias-duplication accounting. In-house SHA-256 (`src/hash.rs`, NIST vectors).
* **T3.4c — generation provenance** ✅ (shipped; **evolved through T12.2–T12.5d → schema
  `zic-rs-compile-manifest-v8`**): `compile --manifest` → `zic-rs-manifest.json`. Originally (v1)
  `zic_rs_version` + tzdb source path/SHA-256/`source_kind` + a stubbed `generation_options` block +
  zones/links touched + an `oracle` block that is **`not-run`** for a bare `compile` (never infers a
  match from tests). Since then T12 made it a real build-identity artifact: **T12.2** split tzdb
  version into detected-vs-claimed (`version_status`) and replaced the stub with a real
  `build_profile`; **T12.3** added the ordered `source_inputs` block (per-file hashes +
  `aggregate_hash` + structural `kind`); **T12.4b** added the `link_profile` block (link counts +
  `alias_map_sha256` + selected/omitted link hashes); **T12.4c** added fail-closed `AliasMap::validate()`;
  **T12.4d** added the `source_profile.backward_evidence` axis; **T12.5b** added the
  `source_profile.backzone_evidence` axis (hash-anchored to the pinned 2026b reference admitted in
  T12.5a.2); **T12.5c** added the `source_profile.packratlist_evidence` axis (backzone *scope* —
  detected only from an admitted generation-policy input hash-matching the pinned 2026b `zone.tab`,
  **never** from compile `source_inputs`, since `PACKRATLIST` is a generation-policy selector, not a
  `zic` source); **T12.5d** added `source_profile.dataform_evidence` (encoding form — hash-matched
  against the pinned 2026b `main`/`vanguard`/`rearguard.zi` artifacts via `source_inputs` membership,
  + `recipe_hash`/`generated_from`; never content-inferred) and removed the last `build_profile`
  placeholders — all detected/claimed/status, hash-backed-or-claim-only, never inferred. The detailed
  schema + doctrine (incl. the all-IANA release-admission matrix, T12.5a.3) live in
  `docs/build-profile-parity.md`. *Still future:* the T12 close receipt;
  an IANA release URL; real oracle results via a compare-and-emit path.
* **T3.4d — optional ecosystem interop bench** ✅ (shipped): feature `ecosystem-tests`
  (`dep:tz-rs`, optional — absent from the default graph). `tests/ecosystem_consumers.rs`
  compiles fixtures with zic-rs and loads the TZif with [`tz-rs`](https://crates.io/crates/tz-rs),
  asserting offset/`is_dst`/abbreviation at ledger trap instants — including **footer-projected**
  futures (`Test/Eastern` 2040, `Europe/London` 2030). Behaviour-only, **not** the oracle; the
  correctness hierarchy and consumer matrix are in
  [consumer-testbench.md](consumer-testbench.md). Jiff is deferred (awkward one-off-file API).
  *Future:* `zic-rs bundle` (`output-tree`/`concat-tzif`/`rust-static`/`manifest-only`) and
  richer `--link-mode` (`manifest-only`/`hardlink`/`dedup-copy`) live here — not built yet.

## Credits follow-up — `docs/upstream-credits.md`

Generate `docs/upstream-credits.md` mechanically from the tzdb `NEWS` / release history so
named-contributor credit is tracked completely and systematically, without bloating
[ACKNOWLEDGEMENTS.md](../ACKNOWLEDGEMENTS.md) (which stays a readable, sourced, explicitly
non-exhaustive summary).

## T4.0 — First real IANA-zone slice: `Europe/London` ✅ (done)

* `fixtures/iana-slices/europe_london_2026b.zi` — a canonical (record-keys-only de-abbreviated)
  slice of `tzdata.zi` 2026b, with self-documenting provenance. Faithfulness proven: reference
  `zic` compiles it **byte-identically** to the verbatim abbreviated extract
  (`…_2026b.abbrev.zi`), asserted by
  `canonical_london_slice_is_byte_identical_to_abbreviated_extract_under_reference_zic`.
* **Verified:** 5 eras (LMT → GMT/BST, the 1968–71 fixed-BST and earlier `BDST` double-DST
  period), footer `GMT0BST,M3.5.0/1,M10.5.0`; matches reference `zic` under the `zdump`
  behaviour oracle over `1830..2045`
  (`europe_london_2026b_matches_reference_zic_over_1830_2045`). It became admissible via the
  T3.2c effective-in-era classification (its final EU era is effectively recurring-only).

## T4 — Further IANA slice expansion

* `America/New_York`, `Asia/Tokyo`, `Australia/Sydney`, … each extracted with provenance,
  gated by the oracle, and added to `docs/compatibility.md` only when it passes. Grow strictly
  by fixture class — never add syntax ahead of a fixture.

## Beyond v0.1 (explicitly out of scope for now)

Leap seconds (`-L`), ownership/mode (`-u`/`-m`), the legacy `posixrules` link (`-p` — reference `zic`
itself warns it is obsolete), `-r` truncation, and full far-past historical fidelity. Listed in
[unsupported-syntax.md](unsupported-syntax.md) so nothing is silently ignored. (Emission style — `zic`'s
`-b slim`/`-b fat` — is implemented via `--emit-style {zic-slim|zic-fat|default}`, T8-slim; the
`localtime` install policy — `-l`/`-t` — via `--localtime`/`--localtime-name`, T9.4.)
