# Reviewer Orientation & Analysis Context

> **Read this first.** It is the *frame* for outside maintainers, packagers, auditors, reviewers, and
> future contributors — the doctrine, ladder, claim boundaries, and current sealed state — **before** you
> inspect details. Without it, `zic-rs` can be misread as "an overcomplicated Rust rewrite of `zic`." With
> it, it reads correctly as what it is: *a reference-admitted Rust TZif compiler candidate whose claims are
> typed, witnessed, report-backed, and bounded.*

`zic-rs` is **not** presented as compatible by resemblance, by language choice, or by broad assertion.
Every compatibility claim is tied to an **admitted reference**, a **typed contract surface**, an **oracle
mode**, a **machine-checkable report**, and an **explicit non-claim boundary**.

Two binding rules govern everything:

> **Reference first. Evidence second. Behaviour third. Claims last.**
>
> **If a human can read it as a claim, a machine must be able to locate the field that proves or bounds it.**

The recurring shape of every change is: **doctrine → owner type → report field → executable witness →
receipt.** (The single canonical positioning sentence: *zic-rs is not a Rust rewrite claiming safety by
language choice; it is a reference-admitted TZif compiler candidate whose claims are typed, oracle-backed,
and machine-checkable.*)

## How to verify (30 seconds, before reading further)

```sh
# 1. The gate (every claim is test-pinned):
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test   # all green (count: STATUS.md)
bash /tmp/t9sweep.sh                                                           # CORE.1 → 341/0/0

# 2. The machine-readable claim surfaces:
zic-rs support-report    --input /usr/share/zoneinfo/tzdata.zi --format json
zic-rs structural-report --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic --format json
zic-rs semantic-report   --input /usr/share/zoneinfo/tzdata.zi --format json
zic-rs tzif-validate     --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic --format json
zic-rs aux-table-validate --zone-tab /usr/share/zoneinfo/zone.tab \
    --zone1970-tab /usr/share/zoneinfo/zone1970.tab \
    --zonenow-tab /usr/share/zoneinfo/zonenow.tab \
    --iso3166-tab /usr/share/zoneinfo/iso3166.tab --format json
zic-rs vendor-oracle-sample   # vendor-oracle-receipt-v1 schema (T16.5; core admits receipts, never runs VMs)
zic-rs release-diff --old OLD/tzdata.zi --new NEW/tzdata.zi --reference-zdump zdump --format json  # T16.6a
zic-rs doctor            # T16.6b: read-only host probe (zic/zdump/tzdata); always exit 0
zic-rs compile --manifest --input /usr/share/zoneinfo/tzdata.zi --out /tmp/out --all-supported
```

Then read these fields — each is a typed surface, not prose:

| Look at | In | It tells you |
|---------|----|--------------|
| `oracle_mode` | every report | whether an oracle ran, or `unavailable` + `skipped_with_reason` (never silent) |
| `negative_capabilities` | support-report provenance | what is **not** claimed — each with its `enforced_by` guard |
| `source_variant_reference_pin_gate` + the manifest `tzdb` block | support-report / manifest | which release is **admitted** (only `2026b`) and detected-vs-claimed |
| `ZIC0nn` diagnostic codes | `docs/zic-warning-parity.md` + `reports/t13/t14-close-receipt.md` | the diagnostic contract (class/severity/verbosity/span/platform) |
| `witnesses[]` + `witness_horizon` + `oracle_identity` | semantic-report | selected `offset/is_dst/abbreviation` behaviour vs `zdump`, scoped to a declared set |
| `tzif_structural_verdict` (+ footer/reader-compat/leap-expiry/version verdicts) | `tzif-validate` (T15.4) | RFC 9636 byte-format integrity — a **separate** axis from semantic witnesses; validates reference `zic` output too |
| `artifact_category` | every claim-bearing row | which evidence category the artifact belongs to (no uncategorised claims) |

If a field reads like a claim, locate the report field that proves or bounds it — that is the whole design.

## Current trust ladder (sealed state)

Each stage is a different trust question, sealed with a close receipt:

| Stage | Question it answers | State |
|-------|---------------------|-------|
| **T12** | what source / build profile / source-profile / source-variant **evidence** supports the emitted artifact? | ✅ closed — `reports/t12-close-receipt.md` (manifest schema **v8**) |
| **T13** | what **diagnostics** are emitted, by class · severity · verbosity · source-origin · source-location · span-precision · reference-platform? | ✅ closed — `reports/t13-close-receipt.md` (`ZIC001`–`ZIC020`) |
| **T14** | how does the compiler behave under **malformed / hostile / pathology-bearing** input? | ✅ closed — `reports/t14-close-receipt.md` (`ZIC021`–`ZIC025`; found+removed a panic on untrusted input) |
| **T15** | can an outside reviewer **verify the conformance claim from machine-readable reports**? | ✅ closed — `reports/t15-close-receipt.md` (the public conformance engine; `ZIC026` added; 15 guard-enforced non-claims; no global verdict) |
| **T16** | which **release / reference build / locator / trust / table / vendor-receipt** does a claim belong to? | ◐ in progress — T16.1–T16.6 sealed (inventory · `ReferenceBuildProfile` · locator+trust · aux-table validator · vendor-oracle receipt admission + no-dep receipt **ingestion** · **release-diff + doctor** · **17 ecology rows / 19 receipts (18 admitted + 1 typed non-admission) — full BSD sweep (FreeBSD · OpenBSD · NetBSD · DragonFly) + illumos triad (OmniOS · SmartOS · OpenIndiana) + Linux (Alpine/musl-tzcode · Debian/NixOS/Ubuntu/AlmaLinux glibc · Gentoo/tzcode · openSUSE/tzcode-RPM) + Yocto/Poky (build-host tzcode-native 2026b admitted + target-runtime non-admission) + SLES 15-SP7 (commercial SUSE, container ≡ Leap) + Arch (rolling, tzcode-`zic` via the `tzdata` pkg)**; each admitted row rejects every fixture (safe), 3–4/5 class+location match + declared divergences; measured vendor lag spectrum, old-fork input-compat gap, complete illumos convergence, musl-vs-glibc libc-independence, **two `zic` implementation lineages (IANA tzcode vs glibc's own) — the glibc lineage version-stratified with a bracketed inflection (glibc 2.34/2.39 old-fork-like 3/5+2 fat < 2.40/2.41 modern 4/5+1 slim → transition between 2.39 and 2.40)**, that **the lineage is a distro packaging choice independent of libc AND package format** (openSUSE proves **RPM ≠ glibc-`zic`**), that **bloat-default is its own packaging axis**, that **some artifacts ship no on-device `zic`** (the Yocto/Poky target image consumer), and that **the embedded build-host producer ≠ target-runtime consumer** (Poky `tzcode-native` owns `zic`, image ships only data); the 2 non-admissions prove the contract handles negative results first-class; QEMU outside the repo). **T16.6 ✅** — operator tooling: `release-diff` (typed `ReleaseChangeKind`; structural always + behavioural via two `zdump` year-windows, oracle absence visible; `zic-rs-release-diff-v1`) + `doctor` (read-only host probe, typed `ToolStatus`, always exit 0; `zic-rs-doctor-v2` since T17.3 — typed `ToolVersionStatus`/`HashReadStatus`); **453 tests at the T16.6 seal**; T16.7/T16.8 reviewer personas remain) |
| **T17** | can the Rust implementation itself be made hard to embarrass? | ✅ closed — `reports/t17-close-receipt.md` (T17.1a/b bounds-guard + `ResourceLimits` caps · T17.2 CONTRACT.TYPING · T17.3 doctor/release-diff taxonomy → doctor **v2** · T17.4 install/materialization · T17.5 `CountArithmeticVerdict` · T17.6 schema/CLI stability [now **11 schema surfaces**] · T17.FUZZ scaffold [`pending_capture` at close, **later run: T23.cargo-fuzz.1/.2**] · T17.7 verify-here) |
| **T18** | where does each claim's external knowledge come from? | ◐ in progress — knowledge index + source/archive ledgers + claim-source-map + copyright policy (**13 sources, S1–S13**; **archival capture pass COMPLETE** — all 10 external sources captured, operator pass 2026-06-02); breadth→40 + T18.3 adjacent-compiler/TZif-reader ledgers remain |
| **T19** | how does a reviewer/packager/security person decide it is safe to depend on? | ✅ sealed — the trust front door: `TRUST.md` + `effect-boundary-map` + `not-yet-ready` + `replacement-readiness-ladder` + `drop-in-compatibility-contract` + `misuse-resistance-ledger` + `audit-readiness` + `maintenance-policy` |
| **T20** | what does the rewrite remove, and what may each audience conclude / NOT conclude? | ✅ sealed — `security-rewrite-evaluation.md` + `security-personas.md` (10 personas; the *may-NOT-conclude* column is the loudest) |
| **T21** | minimal deterministic timezone bundles for containers/embedded? | ◐ first cut — `container-embedded-builder.md` + **`zic-rs size-report`** (footprint + deterministic `bundle_hash`; `zic-rs-size-report-v1`) |
| **T22** | is the resource cost measured and bounded? | ◐ first cut — `perf-ledger.md` + `reports/perf/` (real receipt: 107 KB → 598 TZif files ≈12 ms / ≈22 MiB, far inside the caps; *measured + bounded, not a speed claim*) |
| **T23** | what does each audit tool witness — and what is actually run? | ◐ first cut — `audits/` (the exact 17-tool list, receipt-bearing). **all 17 tools run/confirmed (2026-06-05)**: cargo-audit clean · cargo-geiger · cargo-auditable · miri (tzif core, 0 UB) · panic-analysis · dsfb-gray · **kani (10 bounded proofs)** · cargo-fuzz (found+fixed F1–F3) · cargo-vet.8 (42/1/23) · **creusot PROVED · crux-mir 5/5 Valid · loom proved · flux/hax ran · cargo-crev/cargo-scan · cargo-valgrind→ASan** — *not all green, by design* |

**The live behaviour claim** (never inflated): *zic-rs behaviour-matches reference `zic`/`zdump` for all
341 canonical zones in `tzdata.zi` 2026b over `1900..2040` — 341 match · 0 mismatch · 0 fail-closed
(CORE.1).* That is the contract; structural / byte / diagnostic / operational parity are **separate axes**,
never collapsed.

## Project scope (read this before assuming what `zic-rs` *is*)

`zic-rs` is a **`zic` compiler candidate** — it compiles IANA tzdb source → TZif and verifies itself
against reference `zic`/`zdump`. It is explicitly **not**:

* a **runtime `localtime`/`mktime` library** (it does not interpret a named zone at call time — it
  produces a *compiled artifact from a pinned release*; named-zone *intent* may change with future tzdb
  releases — RFC 9557 — but a compiled artifact's semantics are fixed at the admitted release);
* a **`tzselect` replacement** (no interactive zone selection);
* a **system tzdb packager by default** (no implicit system install; output only under an explicit `--out`);
* a **runtime tzfile-refresh / `TZ_CHANGE_INTERVAL`** mechanism;
* an authority on **future civil time** (tzdb records best-known rules; governments change them — see
  non-goals).

The `NegativeCapability` set (16 entries, each `enforced_by` a guard) is emitted today in every report's
provenance block (T15.2/T15.5/T16.5); a typed `ProjectScope` rollup of these scope lines is tracked → **T16**.
Stated here as the front-door guardrail so a "tzdb toolchain replacement" reader does not overread a
compiler candidate.

## Non-goals (what is explicitly **not** claimed)

* compatibility with all IANA releases without **release admission**;
* byte-exact **stderr** parity;
* **unadmitted** vendor/platform parity;
* full adversarial-filesystem **TOCTOU** resistance;
* semantic-output proof **from diagnostics alone**;
* TZif structural validity **from semantic witnesses alone**;
* source-variant **behaviour** from source-variant **evidence**;
* **Rust safety as a substitute** for compatibility evidence.

These are not prose: most are emitted as machine-visible `negative_capabilities` rows, each naming the
guard/test/receipt that enforces it (see below).

## Evidence categories (no claim-bearing artifact is uncategorised)

Every byte/artifact belongs to a declared category, so category errors (e.g. treating a generation-policy
input as a compile input — the `zone.tab` lesson, T12.5c) become typed, not prose. The owner type is
`manifest::ArtifactCategory`: `compile_input` · `policy_input` · `reference_input` · `generated_artifact`
· `output_artifact` · `diagnostic_artifact` · `semantic_witness_artifact` · `structural_validation_artifact`
· `policy_prose` · `release_note_evidence`. **The rule: a claim-bearing artifact does not enter a public
report without an artifact/evidence category.**

## Report layers must not be blurred

These are distinct surfaces, each with its own report/section:

* **manifest provenance** — what was built, from which inputs (`compile --manifest`; not a TZif sidecar,
  not required to interpret TZif bytes);
* **diagnostic evidence** — what the tool *noticed* and classified (`ZIC0nn`);
* **semantic behaviour** — selected offset / `is_dst` / abbreviation under a declared oracle
  (`semantic-report`, T15.3);
* **TZif structural correctness** — byte-format integrity under RFC 9636 (the `tzif-validate` structural
  validator, T15.4 — five separate typed verdicts; validates zic-rs **and** reference `zic` output);
* **operational / materialization** behaviour — output-tree safety, install policy;
* **release / vendor / platform admission**.

A semantic witness does **not** prove RFC 9636 structural validity. A structural validator does **not**
prove civil-time behaviour. A diagnostic does **not** prove output semantics. A manifest is **not** TZif
semantics.

> **Structural validator scope — RFC 9636 structural validator v1 (T15.4).** The `tzif-validate` validator
> is a **first pass**, explicitly bounded so "a validator exists" is not overread. It covers
> *count/bounds · type-index < typecnt · indicator counts ∈ {0, typecnt} · strictly-ascending transitions ·
> typecnt ≥ 1 · version semantics · footer-shape parseability*, and validates reference `zic` output too. It
> does **not** yet cover *full dual-block 32/64-bit semantic equivalence · footer future-**projection** match
> (a semantic axis, paired with the semantic witnesses) · exhaustive v4 leap-expiry subcases*. Those deeper
> traps are **tracked** (plan T15.4-enrichment / T16), not silently assumed.

## Oracle discipline

**Oracle absence is visible, never silent.** Reports use the typed `manifest::OracleMode`
(`NotRun` / `ReferenceZic` / `ReferenceZdump` / `StructuralDecode` / `Unavailable(reason)`); when a
reference tool is missing, the report renders `{ "mode": "unavailable", "skipped_with_reason": … }` rather
than quietly omitting oracle-backed evidence. Witnesses carry a declared `witness_horizon` and
`fixture_set` so a small seed set is never overread as universal parity.

**Oracle identity is more than version + hash** (the decades-of-platform-variation lesson, and the
sharpest single guardrail). A reference `zic`/`zdump` binary's behaviour depends on its **build flags**
(`ZIC_BLOAT_DEFAULT`, `TZNAME_MAXIMUM`, `TZ_RUNTIME_LEAPS`, `ZIC_MAX_ABBR_LEN_WO_WARN`, `TZDEFRULESTRING`,
`OPENAT_TZDIR`/`SUPPRESS_TZDIR`, …), its **`time_t` model** (signed/unsigned · 32/64), its **runtime leap
support**, and its **locale / TZDIR data-path policy**. So "matches reference `zic`" must resolve to
"matches *which* reference `zic`": `oracle_identity` now carries a typed **`ReferenceBuildProfile`** (T16.2)
— each axis a `BuildAxisEvidence` disposition (`known` / `unknown_unmeasured` / `inferred_forbidden` / …),
with build flags and `time_t` model **`inferred_forbidden`** (never host-guessed; *evidence, not vibes*) and
richer *measured* capture done by the **T16.5 external vendor-oracle lab** (QEMU/containers outside the core
repo), which produced **17 ecology rows / 19 receipts** (canonical matrix: the plan's `T16.5b.1…17` table —
this doc summarises, it does not restate). Illustrative code/data skews: **FreeBSD** (T16.5b.1) `zic`/`zdump`
**tzcode 2022g** while its `tzdata.zi` is **2026b**; **OmniOS/illumos** (T16.5b.2) `zic`/`zdump` **tzcode 2025a**
(owned by core pkg `SUNWcs`) while its data is **`zoneinfo@2026.1`** — real **code/data skews**, both
`ZIC_BLOAT_DEFAULT = slim`, `time_t` signed-64 — so a distro-built reference is never silently mistaken for
divergence caused only by compile configuration.

## Negative capabilities (first-class, enforced)

`manifest::NegativeCapability` renders `{ capability, enforced_by }` — each non-claim names the guard/
test/receipt enforcing it (e.g. `does_not_infer_source_variant_from_output_shape` → the T12.5
`source_variants_not_inferred_*` tests; `does_not_claim_full_toctou_resistance` → the T14.6 ledger). A
non-claim without an enforcing reference would be decorative; these are not.

## Diagnostic contract

Diagnostics are structured compatibility artifacts (`ZIC001`–`ZIC025`), not stderr fragments. A diagnostic
claim is admitted by **class · severity · verbosity · source-origin · source-location · span-precision ·
reference-platform**, with **wording last**. Codes are **append-only**; meanings are never reused; the
stable surface is `(class, location, severity, verbosity, span-precision)`, not exact English. Each code
has machine-checked `layer()` / `span_precision()` / `default_severity()` accessors + a totality test.

## Vendor / platform diagnostics

Vendor/platform behaviour is **admitted per platform, never inferred** from upstream IANA. The matrix
distinguishes upstream-IANA · local-system · glibc-distro · BSD · macOS · AIX · Solaris/illumos ·
distro/locally-patched, with row states `admitted` / `unavailable_on_this_host` / `documentation_only` /
`pending_fixture_run` / `known_divergence` / `available_but_inadmissible_unpinned`. T13 **defines** the
matrix; the **T16.5** QEMU/container-backed oracle lab (external, outside the core repo) executes it against
real vendor `zic` binaries (it does not reopen T13). It has executed for real across **17 ecology rows / 19
receipts (18 admitted + 1 typed non-admission)** — the full BSD sweep, the illumos triad, the major Linux
lineages (musl/glibc/source/content-addressed/RPM/apk/pacman), and the embedded build-system; the **canonical
per-receipt matrix is `docs/tzdb-release-ecology.md`** (the *Vendor/platform oracle admission* execution row),
summarised here.
The durable findings: **two `zic` implementation lineages** — IANA **tzcode `zic`** (BSD/illumos/Alpine/Gentoo/
openSUSE/Arch) vs **glibc's own `zic`** (Debian/Ubuntu/AlmaLinux, owned by `libc-bin`/`glibc-common`; NixOS via
its store) — **the glibc lineage version-stratified with a bracketed inflection** (glibc 2.34/2.39 old-fork-like
3/5 + fat < 2.40/2.41 modern 4/5 + slim → transition between 2.39 and 2.40); the lineage is a **distro packaging
choice independent of libc *and* package format** (openSUSE proves **RPM ≠ glibc-`zic`**); **bloat-default is its
own packaging axis**; **some artifacts ship no on-device `zic`** (the Yocto/Poky target image = TZif consumer →
the 1 typed non-admission); and the embedded **build-host producer ≠ target-runtime consumer**. Every admitted
row **rejects every fixture (safe)** with a real **class + location** verdict (3–4/5 `class_location_match` + the
universal `continuation_without_zone` divergence — zic-rs's finer `ZIC014` vs the vendor's generic "unknown line
type"; old-fork + older-glibc tiers add `nul_byte` → "line too long"). A **measured** vendor lag spectrum, not an
assumed one. **T16.6** then shipped the operator-facing `release-diff` + `doctor` over this evidence.

## Hostile-input & output-tree scope (honest bounds)

T14 tightened malformed-input behaviour and converted an untrusted-input **panic** ("two rules for same
instant") into a controlled coded rejection (`ZIC023`). **But** hostile-output-tree claims are scoped:
covered pre-planted leaf attacks (file/symlink/dir) fail closed and are never written through; the
concurrent **parent-component symlink-swap race** is honestly **not claimed** (`RequiresOpenatStyleHardening`)
— it needs fd-relative `openat`/`O_NOFOLLOW` materialization, deferred to T17/T20.

## Release discipline

A tzdb release is admitted by **ceremony, not inference**: archive identity · signature/provenance ·
admitted file hashes · generated-artifact recipe hashes · Makefile/build-policy review · release-delta
review · source-variant feature profile · oracle identity. **2026b is the only admitted release.** A claim
for one release is never silently generalized to another (`docs/build-profile-parity.md` T12.5a.3 matrix).

## Schema & report stability (reports are themselves APIs)

Once external readers rely on a report, its schema is a compatibility surface. Therefore: explicit schema
version (`zic-rs-support-report-v4` · `zic-rs-structural-report-v3` · `zic-rs-compile-manifest-v8` ·
`zic-rs-semantic-report-v1`); finite vocabularies backed by owner types; **additive/minor vs semantic/major
bump** policy; **shape-witness tests** that flip deliberately on each addition (`tests/conformance_report_shape.rs`);
legacy renderings confined to the serialization boundary with a **drift test** + removal plan (e.g.
`OracleMode::manifest_str()`).

## Preferred review method

Do not ask only "does the feature exist?" Ask:

1. What exact claim is made?
2. What reference/oracle admits it?
3. What owner type represents the finite vocabulary?
4. What report field exposes it?
5. What test/receipt pins it?
6. What is explicitly **not** claimed?
7. What happens if the oracle is unavailable?
8. What platform, release, and fixture set does it apply to?

## Reproduce (the short command path)

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test   # all green (count: STATUS.md)
bash /tmp/t9sweep.sh                                                           # CORE.1 → 341/0/0
zic-rs support-report     --input /usr/share/zoneinfo/tzdata.zi --format json  # capability + provenance + negative_capabilities + oracle_mode
zic-rs structural-report  --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic --format json
zic-rs semantic-report    --input /usr/share/zoneinfo/tzdata.zi --format json  # zdump-backed semantic witnesses (oracle_mode visible)
zic-rs compile --manifest --input … --out … --all-supported                   # build provenance manifest
```

## Current direction

**T15 — the public conformance engine — is CLOSED** (`reports/t15-close-receipt.md`): the internal proof
ladder is now externally runnable and machine-readable. Shipped: typed `OracleMode` + `negative_capabilities`
(T15.2), `OracleResult` unified onto the owner enum (T15.2a), **semantic-witness verdicts + `ArtifactCategory`**
(T15.3), the **RFC 9636 TZif structural validator** (`tzif-validate`, T15.4 — five separate typed verdicts,
validates reference `zic` output too), the one-line **`ConformanceStatus`**/`ConformanceLevel` + the
**report-as-artifact** provenance set (`CompilerIdentity`/`WorkspaceProvenance`/`ReportProvenance`/
`declared_scope_hash`) + the typed claim-shape axes (`ReferencePinGate`/`ClaimPortability`/
`EvidenceAuthorityKind`/`ClaimBoundary`/`valid_disambiguation`) + richer **oracle identity** + golden &
failure-mode fixtures + `ZIC026` (T15.5 + remainder), and the close receipt (T15.close — 15 guard-enforced
non-claims, no global verdict). T15 did not become a prose dump — every field proves or bounds a claim, and
a public report is itself a claim surface, not an unexamined trust root. **Next: T16 (release ecology).**

> Authoritative live state: `research/SESSION-CONTEXT.md` (overview + ledger) and the milestone plan.
> The binding "what works now": the repo `docs/*`, the close receipts in `reports/`, and the test suite.
