# Release ecology & downstream-contract — inventory and execution status (T16)

> **Live state (T16.1–T16.6 sealed; T17 CLOSED; T18.1 first cut sealed; T19 trust front door sealed):** T16.1 established the inventory below; T16.2–T16.6 have
> filled the execution column — ✅ `ReferenceBuildProfile` (T16.2) · ✅ `ReferenceLocatorKind`+`SignatureTrustModel`
> (T16.3) · ✅ auxiliary-table validator (T16.4) · ✅ external vendor-oracle **receipt admission + ingestion**
> with **17 ecology rows / 19 receipts (18 admitted + 1 typed non-admission)** (T16.5-core/T16.5b/T16.5b.1…17)
> · ✅ **operator tooling `release-diff` + `doctor`** (T16.6). The inventory table's *Execution* column is the
> source of truth for per-surface status. **This doc is the in-repo canonical home of the vendor-receipt
> matrix** — the *Vendor/platform oracle admission* row of the execution table below carries the full
> `T16.5b.1…17` enumeration; README / roadmap / reviewer-orientation **summarise and point here, they do not
> restate it.** (Per-receipt raw evidence + image provenance live in the external lab's `README.md` +
> `IMAGE-PROVENANCE.md`, outside the crate.)
>
> **T16.1 was reference/inventory-first — NO behaviour change**, exactly like T9.1 / T10.1 / T11.1 / T12.1 /
> T13.1 / T14.1 / T15.1. It **classifies the release/ecology surfaces that live *outside* T15** — the ones
> that decide *which* reference, *which* release, *which* platform, and *which* auxiliary evidence a claim
> is scoped to — and assigns each an **owner type · report target · test witness · non-claim boundary ·
> execution milestone**. It changes no code beyond an executable **inventory witness**
> (`tests/release_ecology_inventory.rs`) that pins the *current* non-claims so every later T16.x addition
> is a deliberate, visible flip (the T13.1/T14.1 "executable from day one" lesson). **Live gate at T16.1:
> 397 tests · CORE.1 341/0/0**.
>
> **The T16 north star (one sentence):** *"matches reference `zic`" means "matches **which** reference
> `zic`?"* — a compatibility claim is only as strong as the identity of the thing it was checked against.
> T15 made the *reports* trustworthy; T16 makes the wider *release / platform / downstream ecology*
> **admissible**, so "ecology overclaim" becomes impossible to blur.

## Authority boundary (why this is a separate layer from T15)

Two different standards govern two different things, and T16 must keep them apart:

* **RFC 9636 (TZif)** — the *format* a compiled artifact is in. T15's `tzif-validate` lives here. It says
  nothing about *where the data came from* or *which tool built the reference*.
* **RFC 6557 / BCP 175** — the *maintenance procedures* for the tz database and its code (releases,
  intake, distribution). Release ecology — intake provenance, patch stacks, signing, vendor builds — lives
  here. This is a **process / governance** authority surface, distinct from TZif format validity and from a
  single `zic` behaviour check.

So a T16 claim is never "valid TZif" (that is T15.4) and never "behaviour-matches `zic`" alone (that is
CORE.1). It is *"this artifact was built from **this admitted release**, against **this identified
reference**, on **this admitted platform**, with **these auxiliary tables**, and here is what is **not**
admitted."* Cross-links (not duplicated here): [`build-profile-parity.md`](build-profile-parity.md) (T12
build *inputs*), [`tzdb-governance.md`](tzdb-governance.md) (IANA authority chain),
[`platform-portability.md`](platform-portability.md) (portable-core / install-surface split),
[`generated-data-contract.md`](generated-data-contract.md) (consumer guarantees).

## The central concept — `ReferenceBuildProfile` ("matches *which* reference `zic`?")

Today (T15.5-remainder) `OracleIdentity` records the oracle's **program name · binary sha256 · `--version`
string · platform · command-line · env · `zoneinfo_resolution`**. That answers "*which binary*," but not
"*which build*." Reference `zic` is portable C compiled by many packagers with **compile-time flags that
change observable output and diagnostics** — so two `zic` binaries with the same version string can differ.
`ReferenceBuildProfile` is the T16 enrichment that captures the *build*, so a divergence is attributable to
configuration rather than mistaken for a zic-rs bug:

| Axis | Why it changes behaviour/diagnostics |
|------|--------------------------------------|
| `source_release` + `binary_sha256` | the exact tool (already captured in `OracleIdentity`) |
| build flags (`ZIC_BLOAT_DEFAULT` · `TZNAME_MAXIMUM` · `TZ_RUNTIME_LEAPS` · `ZIC_MAX_ABBR_LEN_WO_WARN` · `TZDEFRULESTRING`) | slim/fat default, abbr-warning threshold, leap handling, footer rule |
| `TimeTModel` (signed/unsigned · 32/64 · rust-i64) | far-past/far-future representable range (e.g. AIX 32-bit pre-1901) |
| `RuntimeLeapSupport` (enabled / posix-only / unknown) | whether `right/` semantics are even meaningful |
| `TzdirResolutionPolicy` (prepends / suppresses TZDIR) | whether `zdump` read the intended tree |
| `locale` (`LC_ALL`/UTF-8 availability) | name-byte warnings, sort order |
| `warning_thresholds` | which `-v` warnings fire (`ZIC018`/`ZIC026` thresholds) |
| `reference_platform` | the vendor axis (T13 matrix; executed at T16.5) |

`ReferenceBuildProfile` was **named here and built at T16.2** — emitted as `oracle_identity.reference_build_profile`
with a typed `BuildAxisEvidence` disposition per axis (`known` / `unknown_unmeasured` /
`unavailable_on_this_host` / `not_applicable` / `documentation_only` / `inferred_forbidden`). Today only the
genuinely-observable axes are `Known` (binary sha256 · platform · captured `--version` · data-path policy);
build flags and `time_t` model are **`inferred_forbidden`** (never host-guessed — *evidence, not vibes*), and
runtime-leap / warning thresholds are `unknown_unmeasured`. Measured build-flag / `time_t` / runtime-leap
capture is the **T16.5** external-lab job.

## The inventory (surface · owner type · report target · witness · non-claim boundary · execution milestone)

| Surface | Owner type (named) | Report target | Test witness | Non-claim **today** | Execution |
|---------|--------------------|---------------|--------------|---------------------|-----------|
| **Reference build identity** | `ReferenceBuildProfile` + `BuildAxisEvidence` (disposition per axis) | semantic-report `oracle_identity.reference_build_profile` | `tests/release_ecology_inventory.rs` (T16.2: present + dispositioned) | **✅ emitted (T16.2)** — but most axes honestly `unknown_unmeasured`/`inferred_forbidden`: build-flags + `time_t` are **never host-guessed**; only binary/platform/version/data-path are `Known`. Richer capture → T16.5 | **T16.2 ✅** + **T16.5** |
| **Reference locator** | `ReferenceLocatorKind` { `versioned_archive` (only this backs *sealed* claims) · `live_current_directory` · `local_cached_copy` · `distro_source_package` · `unknown` } | semantic-report `oracle_identity.reference_admission` | `tests/reference_admission.rs` (T16.3) | **✅ emitted (T16.3)** — `supports_sealed_claim()` enforced as code; the live PATH oracle is `live_current_directory` (exploration-grade, **not** sealed); only the T12.5a.2 versioned archive seals | **T16.3 ✅** |
| **Signature trust** | `SignatureTrustModel` { `fingerprint_anchored` · `web_of_trust_validated` · `platform_keyring` · `hash_only` · `unsigned` · `unknown` } (+ `pins_integrity()`) | semantic-report `oracle_identity.reference_admission` | `tests/reference_admission.rs` (T16.3) | **✅ emitted (T16.3)** — the 2026b archive is **`fingerprint_anchored`** (key `7E37…7E34`); `hash_only` = integrity-only, **never** "signature verified"; `fingerprint_anchored` ≠ web-of-trust | **T16.3 ✅** |
| **Release intake / patch stack** | `ReleaseIntakeProvenance` { `official_archive` · `mailing_list_patch` · `distro_patch` · `local_override` } + `PatchStackIdentity` + `ReleaseAdmissionCompleteness` | manifest `tzdb` block / receipt | inventory witness | only the **pristine official 2026b archive** is admitted; the installed `tzdata.zi` is **`2026b-dirty` (local), not pristine**; distro patch-stacks not admitted | **T16.6** |
| **Auxiliary tables** | `ZoneTableKind` + `ZoneTableStructuralVerdict` + `ZoneUniverse` + `CountryCodeAuthority` | `aux-table-validate` / `zic-rs-aux-table-validation-v1` | `tests/aux_table_validation.rs` (T16.4) | **✅ built (T16.4)** — **structural admissibility only**; `zone.tab` is `policy_input` not `compile_input`; **names the universe** (no name resolution); `zonenow` = now/future not all-history; coord syntax ≠ geodetic; **not** one-row-per-country (semantic-row dedup) | **T16.4 ✅** |
| **Install ecology** | `RedoInstallLayout` { `posix_only`·`right_only`·`posix_right`·`right_posix` } + `LocaltimeInstallPolicy` + `InstallContext` { `staged_destdir`·`production`·`test_only` } | manifest install profile | `tests/portability.rs` + T9 install tests | **no implicit system install** (output only under `--out`); no REDO-layout witness; no runtime tzfile-refresh / `TZ_CHANGE_INTERVAL` | **T16.4** |
| **Vendor/platform oracle admission** | `ReferencePlatformStatus` + `vendor-oracle-receipt-v1` (`OracleRunStatus`/`OracleExitDisposition`/`StderrEncoding`/`ReceiptHashScope`/`OracleInvocationIdentity`) | `vendor-oracle-sample` (schema) + `VendorOracleReceipt::{from_json, admit}` + `vendor-oracle-admit` CLI | `tests/vendor_oracle_receipt.rs` (T16.5-core/T16.5b) + admitted `freebsd_14` + `openbsd_79` + `netbsd_101` + `dragonfly_642` + `omnios_illumos` + `smartos_20260528` + `openindiana_hipster_20260430` + `alpine_3234` + `debian_13` + `nixos_2511` + `ubuntu_2404` + `almalinux_9` + `gentoo_20260531` + `opensuse_leap_16` + `yocto_poky_buildhost_native_zic` + `yocto_poky_target_runtime` + `sles_15sp7` + `archlinux` receipts (external lab) | **✅ T16.5-core + T16.5b + T16.5b.1…17** — typed contract + strict `admit()` (never admitted by assumption) + canonical emission + **no-dep fail-closed JSON ingestion** (`from_json`; parse-failure ≠ inadmissible); **core admits *receipts*, never runs VMs** (`does_not_ship_or_operate_vendor_qemu_labs_in_core_repo`). **17 ecology rows / 19 receipts (18 admitted + 1 typed non-admission) — BSD sweep + illumos triad + Linux: FreeBSD 14.3 (2022g) · OpenBSD 7.9 (old fork, 2023c) · NetBSD 10.1 (2022g) · DragonFly 6.4.2 (oldest fork) · OmniOS/SmartOS/OpenIndiana (2025a) · Alpine 3.23.4 (musl, tzcode 2026b) · Debian 13 (glibc-`zic` 2.41) · NixOS 25.11 (glibc-`zic` 2.40, content-addressed store) · Ubuntu 24.04.4 LTS (glibc-`zic` 2.39) · AlmaLinux 9.8 (glibc-`zic` 2.34, enterprise RPM) · Gentoo (tzcode-`zic` 2026a via `timezone-data`, source-built) · openSUSE Leap 16.0 (tzcode-`zic` 2025b via the `timezone` RPM) · Yocto/Poky (build-host `tzcode-native` 2026b admitted + target-runtime consumer non-admission) · SLES 15-SP7 (commercial SUSE, container; tzcode-`zic` via `timezone` RPM ≡ Leap, distinct binary) · Arch Linux (rolling; tzcode-`zic` via the `tzdata` pacman pkg, glibc 2.43 unused — 6th packaging model)** — each admitted row rejects every fixture (safe); 3–4/5 `class_location_match` + declared `divergence`s (`continuation_without_zone` on all; **`nul_byte`→"line too long"** on OpenBSD + DragonFly **and the older glibc builds Ubuntu-2.39 + AlmaLinux-2.34**); measured **vendor lag spectrum** (DragonFly oldest < OpenBSD 2023c < FreeBSD/NetBSD 2022g < illumos 2025a < openSUSE 2025b < Gentoo 2026a < Alpine 2026b=upstream) + **complete illumos convergence on three independent builds** + **C-library independence** (Alpine musl = same 4/5+1 as glibc) + **two `zic` implementation lineages, the glibc one version-stratified with a bracketed inflection** (glibc **2.34**/**2.39** old-fork-like [no NUL class, fat] < **2.40**/**2.41** modern [NUL class, slim], transition between 2.39 and 2.40) + **the `zic` lineage is a distro packaging choice independent of libc AND package format** (Gentoo runs tzcode-`zic` via `sys-libs/timezone-data` despite glibc 2.42; openSUSE runs tzcode-`zic` via the `timezone` RPM where AlmaLinux's RPM gives glibc-`zic` → **RPM ≠ glibc-`zic`**; 6 packaging models) + **bloat-default is its own packaging axis** (fat in old-glibc + Gentoo-tzcode; slim in Alpine/openSUSE-tzcode + modern-glibc) + **some artifacts ship NO on-device `zic`** (the Yocto/Poky target image = TZif consumer — precompiled data, compiler off-device) + **the embedded build-host *producer* ≠ target-runtime *consumer*** (Yocto/Poky: `tzcode-native` 2026b owns `zic` and compiles the target's TZif → admitted; the produced `core-image` ships only data → non-admission); the 2 non-admissions are decided by rule (`admit`=`not_admitted_platform_not_admissible`), proving the contract represents a negative ecology result as first-class; per-image provenance in `IMAGE-PROVENANCE.md` | **T16.5-core ✅ · T16.5b ✅ · T16.5b.1…17 ✅** |
| **Generated-transform provenance** | `SourceLineProvenance` { `original_source` · `generated_zi` · `transformed_by_ziguard\|zishrink` } + `DiagnosticLocationProvenance` | diagnostic location / manifest `source_profile` | inventory witness + T12.5d `recipe_hash` test | diagnostic locations are **direct-source today** (no original-line mapping for generated `.zi`); `tzdata.zi`/`leapseconds` are **generated `InstallTextArtifact`**, not pristine source | **T16.6 / T17** |

## Non-claims (T16.1)

* **No new report field or behaviour** — T16.1 is the *inventory*, not the engine (each surface's owner
  type is **named + assigned**, not built; built in its execution milestone, born typed there).
* **No new schema bump** — the inventory witness only *asserts the current non-claims* (the not-yet-emitted
  fields are genuinely absent) so later additions flip deliberately.
* zic-rs does **not** claim an admitted vendor oracle, an auxiliary-table validator, or generated-transform
  line mapping **yet** — each is a recorded gap with an owning milestone, never silently assumed.
  (`ReferenceBuildProfile` is emitted as of T16.2 with honest `unknown_unmeasured`/`inferred_forbidden`
  dispositions; `ReferenceLocatorKind` + `SignatureTrustModel` are emitted as of T16.3 — *evidence, not
  vibes*.) **The sealed-claim rule is enforced as code:** only a `versioned_archive` with integrity-pinned
  trust seals; the live-PATH oracle a report runs against is exploration-grade, never sealed.
* **The core repo does not ship or operate vendor QEMU labs** (`does_not_ship_or_operate_vendor_qemu_labs_in_core_repo`,
  → `NegativeCapability` at T16.5): *the core repo admits evidence; an external lab generates it.* T16.5
  admits + verifies externally produced vendor-oracle **receipts**; VM images / QEMU orchestration /
  OS-install scripts live **outside** the core repository and are never vendored.

## Acceptance (T16.1)

> T16.1 is accepted when zic-rs classifies the release/ecology surfaces outside T15 — `ReferenceBuildProfile`,
> `ReferenceLocatorKind`, `SignatureTrustModel`, auxiliary tables, install ecology, vendor/platform oracle
> admission, patch-stack/source provenance, and generated-transform provenance — with each surface assigned
> an owner type, report target, test witness, non-claim boundary, and later execution milestone, and with an
> executable witness pinning the current non-claims; no behaviour change, no schema bump.
