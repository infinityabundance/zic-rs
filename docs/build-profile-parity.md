# Build-profile parity — inventory (T12.1)

> **Status: reference-only inventory (T12.1), like T9.1 / T10.1 / T11.1.** This pins the tzdb
> *build-profile model* and records exactly what zic-rs's provenance manifest captures today (and the
> gaps), **before** any implementation. **No behaviour change here.** Cross-links
> [`tzdb-governance.md`](tzdb-governance.md), [`generated-data-contract.md`](generated-data-contract.md),
> [`leap-right-v4-microcases.md`](leap-right-v4-microcases.md) — does not duplicate them.

## Trust ladder (what each operational milestone makes auditable, in 30 seconds)

| Stage   | Trust question it answers |
|---------|---------------------------|
| **T9**  | Did the CLI / filesystem / install surface behave *safely* (exit status, no partial install, install policy within `--out`)? |
| **T10** | Did range/emission (`-b`/`-R`/`-r`) become *explicit policy*, not parser trivia? |
| **T11** | Did leap / `right/` / TZif-v4 behaviour stay *bounded and reference-first* (opt-in, grammar-walled)? |
| **T12.2** | Can the build *describe its own identity* — detected-vs-claimed tzdb version, emit/range/link/output-tree profile? |
| **T12.3** | Can the manifest pin the *exact ordered source inputs* (per-file hashes + order-sensitive aggregate)? |
| **T12.4b** | Can the manifest describe the *alias/link surface* (selected / omitted / failed counts + hashes)? |
| **T12.4c** | Can the exported alias map *fail closed* if it is internally inconsistent? |
| **T12.4d** | Can `backward` membership be *reconciled as evidence* (detected/claimed/status) without inference? |
| **T12.5a** | Do we *understand the upstream source-variant build process* (DATAFORM/BACKWARD/PACKRATDATA/PACKRATLIST) before touching it? — a **decision/inventory stage, not behaviour support** |
| **T12.5a.1/.2** | Did we *admit the real upstream bytes before implementing* — fetch pristine tzdb 2026b, **verify its signature**, SHA-256-pin the reference set, and lift the gate **only** for that pinned version? |
| **T12.5b** | Can `backzone` (`PACKRATDATA`) *membership* be reconciled as hash-backed evidence (detected/claimed/status), anchored to the pinned 2026b reference, never inferred? |
| **T12.5c** | Can `backzone` *scope* (`PACKRATLIST` subset) be reconciled as evidence **from an admitted generation-policy input** — never from compile `source_inputs` (which would be a category error, since `PACKRATLIST` is a generation-policy selector, not a `zic` source)? |
| **T12.5d** | Can the `DATAFORM` *encoding* (`main`/`vanguard`/`rearguard`) be reconciled as hash-backed evidence against the pinned 2026b generated `.zi` artifacts (+ a `recipe_hash` binding their generation) — **never** inferred from negative `SAVE`, output shape, names, `PACKRATLIST`, or `backzone`? |
| **T12.6** | Can an operator *see the trust state by running the tool* — manifest schema + source-variant reference-pin gate (now `lifted_for_2026b`) surfaced in `support-report` / `structural-report`? |

> Read top-to-bottom, this is a maintainer-grade migration story: each stage turns an ambiguity that
> legacy C `zic` tolerates into a typed, tested, documented **evidence boundary** — not new features.
> **T12.5a is deliberately an inventory, not an implementation:** before modelling backzone/rearguard/
> vanguard, we pin *how IANA defines them* and decide what zic-rs may claim vs must refuse.

## Why build-profile manifests matter to packagers (the external adoption story)

A distro packager, language-runtime maintainer, or container builder evaluating zic-rs does not just
ask "are the bytes right?" — they ask **"what produced this tree, and can I trust the metadata about
it?"** The build-profile manifest exists to answer that, concretely:

- **zic-rs output is not only bytes — it is a *policy product*.** The same tzdb release compiled with a
  different emit style, range, leap source, or source set is a *different artifact*. The manifest
  records that policy as structured fields (never a vague `profile = "tzdata 2026b"` label), so two
  "2026b" trees are never falsely equated.
- **A packager can see exactly what source set + policy produced the tree** — ordered input files with
  hashes (`source_inputs`), the build profile (`build_profile`), and the detected-vs-claimed tzdb
  version (`tzdb.version_status`) that refuses to silently stamp a release (e.g. the installed
  `tzdata.zi` here sniffs as `2026b-dirty`, *not* pristine `2026b`).
- **Link aliases are observable output identity — but alias presence does not prove backward source
  participation.** The manifest keeps these separate: `link_profile` records *what aliases the build
  exposed*; `source_profile.backward_evidence` records *what source evidence (if any) explains
  `backward` membership* — reconciled as detected/claimed/status, never guessed.
- **`--alias-map` is validated** (`AliasMap::validate()`, T12.4c), so exported alias metadata **cannot
  silently drift** from the materialised outputs (every alias names a real compiled zone with a
  matching hash, or the artifact fails closed).

The point: the manifest turns "trust me" into "here is the evidence chain" — which is what makes a
conservative, reference-first replacement *evaluable* in a staged build environment before adoption.

## Doctrine

> **A tzdb *release* is not a single output. A tzdb release + a *build profile* is an output
> identity.** Two trees stamped "2026b" can differ (backzone in/out, rearguard vs vanguard encoding,
> posix vs right, slim vs fat). The manifest must record the *profile*, precisely enough that two
> "2026b" artifacts are never falsely treated as the same thing — and never as a vague label like
> `profile = "tzdata 2026b"`.

## The build-profile axes (pinned from the tzdb/`zic` build model)

1. **Data source set** — which IANA source files are compiled:
   - **main region files** — `africa antarctica asia australasia europe northamerica southamerica
     etcetera` (+ `factory`): the canonical zones.
   - **`backward`** — compatibility **renames/aliases** (e.g. `US/Eastern → America/New_York`).
     *Included* in the default IANA `tzdata.zi`.
   - **`backzone`** — pre-1970 data for zones that agree post-1970; **excluded by default** (opt-in via
     `PACKRATDATA=backzone`). Changes the *identifier set* and pre-1970 history, not post-1970
     behaviour.
2. **Data-format variant** — the *same data*, different *encoding*:
   - **vanguard** — newest features (negative `SAVE`, `%z`, `24:00`/day-shift `ON`).
   - **rearguard** — avoids them (old-reader-friendly workarounds).
   - **main** — the middle ground shipped as `tzdata.zi`.
   (zic-rs already compiles the vanguard-style features it needs — negative SAVE [T7 law 7], `%z`,
   re-anchored `ON` [law 10]; rearguard is a distinct *input encoding* of the same zones.)
3. **Leap profile / output tree** (T11):
   - **default** / **`posix/`** — **no** leap table.
   - **`right/`** — **with** the leap table (`-L leapseconds`). An explicit profile, never the default.
4. **Single-file vs per-zone source** — `tzdata.zi` (one zishrink file, what is usually installed) vs
   the individual region source files. zic-rs reads either (a file or a directory of files) via
   `--input`.
5. **Emission / range knobs** (already in zic-rs): `-b slim|fat` (T10.2), `-r`/`-R` (T10.3/4).

## Output identity (the precise field set — never a vague label)

```
tzdb_release    = "2026b"                 (from `# version …`, via report::sniff_tzdb_version)
source_hash     = sha256(of each input)
source_set      = [africa, …, backward]   (which files; backzone in/out)
rearguard       = true | false
vanguard        = true | false
backzone        = included | excluded
leap_source     = none | <leapseconds path/hash>     (→ output_tree posix vs right)
output_tree     = posix | right
emit_style      = default | zic-slim | zic-fat
range           = none | @lo/@hi
redundant_until = none | @hi
link_policy     = copy | symlink
```

> **Implementation mapping (as of T12.5d).** This is the T12.1 *north-star* concept; the live schema
> (`zic-rs-compile-manifest-v8`) realises it across structured blocks: `tzdb` (`tzdb_release` =
> `detected_version`, `source_hash` = `source_sha256`), `source_inputs` (the `source_set` concept,
> now an **ordered** per-file list + `aggregate_hash` + structural `kind` — see T12.3),
> `build_profile` (`leap_source`/`output_tree`/`emit_style`/`range`/`redundant_until`/`link_mode`; with
> the `DATAFORM` axes now in `source_profile`, **nothing remains** as a `build_profile` `"unknown"`
> placeholder), `link_profile` (`link_policy` + counts + alias/link hashes — T12.4b), and
> `source_profile` (`backward_evidence` — T12.4d; `backzone_evidence` — T12.5b; `packratlist_evidence`
> — T12.5c; `dataform_evidence` — T12.5d — each a detected/claimed/status axis). See "what the manifest
> captures (post-T12.5d)" below for the authoritative current shape.

That precision is what stops two "2026b" outputs from being treated as identical artifacts.

## zic-rs today — what the manifest captures (post-T12.5d)

`src/manifest.rs` (`CompileManifest`, schema **`zic-rs-compile-manifest-v8`**) now records the build
**identity**, not just a stub. Five structured blocks carry it:

- **`tzdb` (`TzdbProvenance`)** — version provenance: `source_path` (a human display field, **not**
  identity — machine-local), an order-independent `source_sha256` (content over the source bytes in
  sorted order), a **`detected_version`** (from `report::sniff_tzdb_version`, which reads the
  `# version 2026b` header) reconciled against an optional **`claimed_version`** (the `--tzdb-version`
  flag). `version_status()` reports `match` / `detected_differs_from_claim` / `detected_only` /
  `claimed_only` / `unknown` — the manifest **never silently stamps a release** it did not detect.
- **`source_inputs` (`SourceInputs`, T12.3)** — the deterministic *input identity*: a structural
  **`kind`** (`tzdata_zi` / `multi_file` / `single_file` / `unknown` — input *form*, never source-set
  *membership*), the **`files`** list in **input order** (each with a portable `logical_name`
  (basename, never an absolute path), per-file `sha256`, `bytes`, and `order_index`), and an
  order-sensitive **`aggregate_hash`**. **Source order is part of the build identity**: reordering the
  same files changes `aggregate_hash` (while the canonicalized `tzdb.source_sha256` stays equal).
- **`build_profile` (`BuildProfile`)** — the real **`emit_style`** (semantic — `zic_slim`, not
  `flag_b`), **`range`**, **`redundant_until`**, **`link_mode`**, **`output_tree`** (`posix` vs `right`,
  driven by whether leaps were applied), and a **`leap_source` (`LeapSourceInfo`)** block (mode/sha256/
  entry-count/expires/rolling-entries). As of T12.5d it carries **no source-variant placeholders at
  all** — `backward`/`backzone`/`PACKRATLIST`/`DATAFORM` are all `source_profile` evidence axes, so
  `build_profile` describes purely *how this run emitted*, never source-set membership or encoding.
- **`link_profile` (`LinkProfile`, T12.4b)** — link/alias identity: **`link_policy`** (`copy`/`symlink`),
  the counts **`zones_compiled_count`** / **`links_selected_count`** / **`links_materialized_count`** /
  **`links_omitted_count`** / **`links_failed_count`** (selected = eligible & made; omitted = excluded
  by selection *policy*; failed = missing/cyclic chain *error* — the three never blur), and stable
  hashes **`alias_map_sha256`** (over the deterministic `alias-map.json` serialization — sorted by
  identifier, fixed field order, LF, no timestamps) + **`selected_links_sha256`** / **`omitted_links_sha256`**
  (order-independent set identities). **Links are output identifiers, never source-set membership
  evidence** — this block never sets `backward`/`backzone`.
- **`source_profile` (`SourceProfile`, T12.4d + T12.5b/c/d)** — source-evidence axes, each a reconciled
  **evidence axis, never a boolean** (`detected`/`claimed`/`status`/`evidence_sha256`), **never inferred**
  from the alias/link surface:
  - **`backward_evidence`** (T12.4d): `detected` is hash-backed (an admitted `--backward-source` whose
    bytes are/aren't in `source_inputs`); `claimed` is the bare `--backward` flag.
  - **`backzone_evidence`** (T12.5b): `detected` is hash-anchored to the **pinned 2026b reference
    `backzone`** (`REF_2026B_BACKZONE_SHA256` = `63fb39ad…`, admitted in T12.5a.2) — `present` iff that
    hash is among `source_inputs`, else `unknown` (**presence is hash-backed; absence never asserted** —
    a concatenated `.zi` may merge backzone); `claimed` is the bare `--backzone` flag. Source
    *membership* (`PACKRATDATA`), version-scoped.
  - **`packratlist_evidence`** (T12.5c) — backzone *scope*: `detected: subset_from_policy_input` only
    when an **admitted generation-policy input** (`--packratlist-source`) hash-matches the pinned 2026b
    **`zone.tab`** (`REF_2026B_ZONE_TAB_SHA256` = `4d8e389e…`) **and** backzone is present; else
    `unknown`. **`PACKRATLIST` is a generation-policy input, not a `zic` compile source — detection is
    NEVER keyed off `source_inputs`** (that would be a category error; `zone.tab` isn't compilable).
    `full`/`none` are claim-only (`--packratlist {full|subset|none}`).
  - **`dataform_evidence`** (T12.5d) — the *encoding* form: `detected: main|vanguard|rearguard` when a
    `source_input`'s hash matches a **pinned 2026b generated artifact** (`REF_2026B_{MAIN,VANGUARD,
    REARGUARD}_ZI_SHA256`), else `unknown`. **Unlike `PACKRATLIST`, the `.zi` artifacts *are*
    compilable `zic` sources, so `source_inputs` membership is category-correct here** (it mirrors
    `backzone`, not `packratlist`). Two extra provenance fields specific to a *generated* artifact:
    **`recipe_hash`** (binds archive · `Makefile` · `ziguard.awk` · command · toolchain, hashed as raw
    bytes — see `dataform_recipe_hash`) and **`generated_from`** (`"tzdb-2026b"`). `claimed` is the bare
    `--dataform` flag. **Never inferred** from negative `SAVE` or any syntax (mainline 2026b already
    uses negative SAVE), output shape, zone names, filenames, `PACKRATLIST`, or `backzone`. `ziguard.awk`
    is **not** treated as a general converter — the `.zi` are recorded as generated reference artifacts
    with a recipe, not as something zic-rs can reproduce or transform.
  This completes the source-variant arc: every membership/encoding axis is now a hash-backed/claim-only
  evidence axis here, and `build_profile` holds no `"unknown"` placeholders. See the dedicated sections
  below.

**Source identity vs profile identity — kept separate (T12.3 doctrine).** `source_inputs` answers
*"which exact files, in what order, with what bytes"*; `build_profile` answers *"what knobs/profile"*.
The two never blend: a file *named* `backzone` does **not** flip the `backzone` axis (that stays
`"unknown"` until a pinned, hash-backed detector exists), and `kind: "tzdata_zi"` is a structural
statement — it does **not** assert a pristine upstream release (the `2026b-dirty` case proves why).
There is deliberately **no `synthetic_fixture` kind**: synthetic-vs-pristine is not deterministically
detectable from input form, so claiming it would be exactly the filename-inference the doctrine
forbids — that question is answered by the orthogonal `detected_version` / `version_status` (a
synthetic `.zi` has `detected_version: null` → `version_status: "unknown"`, never claimed upstream).

**Closed in T12.2/T12.3:** `tzdb.version` is no longer hard-coded; the stale
`leapseconds:"unsupported"` / `rearguard:"unsupported"` stubs are gone; the real
`emit_style`/`range`/`redundant_until`/`link_mode`/`output_tree` + leap-source hash are recorded
(T12.2); and the exact ordered source-input set + per-file hashes + `aggregate_hash` are recorded
(T12.3). The manifest **describes this invocation** — a capability claim (`"supported"`) never appears
as a field *value*, and absolute paths are never treated as portable identity.

**Still open (later substeps):**

- **`backzone`/`backward`/`rearguard`/`vanguard` membership detection** (currently honest `"unknown"`)
  → T12.4/T12.5; no profile label will exist without hash-backed source evidence.
- **`compare` is single-zone**, not release-to-release; profile diffs are a separate concern (T16.2
  `release-diff`).

## Substep ladder (T12)

- **T12.1 — build-profile inventory ✅** (this doc; reference-only, no behaviour change).
- **T12.2 — manifest build-profile fields ✅** — schema bumped to `zic-rs-compile-manifest-v2`;
  `TzdbProvenance` now carries `detected_version` (via `sniff_tzdb_version`) reconciled with a
  `claimed_version` (`--tzdb-version`) through `version_status()`; a `BuildProfile { source_set,
  output_tree, leap_source, emit_style, range, redundant_until, link_mode }` records the real run
  (semantic field values — `emit_style = zic_slim`, never `flag_b`); the stale `leapseconds`/
  `rearguard` `"unsupported"` stubs are gone (structured `leap_source` + honest `"unknown"`). Additive,
  provenance-only — **no compiler semantics changed**; CORE.1 unchanged (341/0/0). Tests in
  `tests/manifest.rs` (`manifest_records_build_profile_of_this_run`,
  `manifest_reconciles_detected_and_claimed_version`, schema-v2). **258 tests.**
- **T12.3 — multi-file source-set ✅** — schema bumped to `zic-rs-compile-manifest-v3`; new
  `SourceInputs { kind, files: [SourceFile { logical_name, sha256, bytes, order_index }],
  aggregate_hash }` records the deterministic **input identity in input order** (never re-sorted).
  `kind` is structural *form* only (`tzdata_zi`/`multi_file`/`single_file`/`unknown`); `aggregate_hash`
  is order-sensitive (reorder ⇒ different identity), complementing the order-independent
  `tzdb.source_sha256`; `logical_name` is a portable basename, absolute paths never enter identity.
  **No source-set membership is inferred** (a file named `backzone` does not flip the `backzone`
  axis); the redundant `build_profile.source_set` was retired (superseded). Additive, provenance-only
  — **no compiler semantics changed**; CORE.1 unchanged (341/0/0). Tests in `tests/manifest.rs`
  (`manifest_records_single_file_input`, `manifest_records_multi_file_inputs_in_order`,
  `manifest_aggregate_hash_changes_when_file_order_changes`,
  `manifest_aggregate_hash_changes_when_bytes_change`, `manifest_identity_is_path_independent`,
  `manifest_distinguishes_tzdata_zi_from_multi_file`,
  `manifest_synthetic_fixture_not_claimed_as_upstream_release`,
  `manifest_does_not_infer_backzone_from_filename`,
  `manifest_single_non_zi_input_is_single_file_not_guessed`). **267 tests.**
- **T12.4 — link / alias profile (identifier-set, reference-first; split a–d):**
  - **T12.4a — reference inventory ✅** (this section below; reference-only, no behaviour change):
    pinned `zic.c` link semantics + how `backward` contributes links + the current manifest link
    state and gaps.
  - **T12.4b — manifest link profile ✅** — new `LinkProfile` block (schema bumped to
    `zic-rs-compile-manifest-v4`): `link_policy`, the `zones_compiled`/`links_selected`/
    `links_materialized`/`links_omitted`/`links_failed` counts (selected/omitted/failed kept
    **distinct** — omission is policy, failure is error), and the stable `alias_map_sha256` +
    `selected`/`omitted_links_sha256` hashes that bind the build to its alias map. `build_compile_manifest`
    gained a `&Database` param (links classified exactly as `plan::run` does). Additive,
    provenance-only — **no compiler semantics changed**; CORE.1 341/0/0. Tests in `tests/manifest.rs`
    (`manifest_records_link_counts`, `manifest_records_alias_map_hash`,
    `alias_map_hash_is_stable_for_same_link_set`, `alias_map_hash_changes_when_link_set_changes`,
    `selected_and_omitted_links_are_distinct`, `copy_vs_symlink_link_policy_recorded`,
    `link_name_selection_compiles_target_and_materializes_link`) + `fixtures/minimal/links_two.zi`.
    **274 tests.** Documented the **link-name-selection trap**: selecting a link name compiles its
    *target* and materialises the link (not an empty build).
  - **T12.4c — `--alias-map` validation ✅** — new **`AliasMap::validate()`**, called by `build()` so
    every produced alias map is **fail-closed consistent**: every `Link` entry's `target` is present as
    a `Zone` entry (no dangling alias — *"missing target fails"*), its `target_sha256` is non-empty and
    **equals** that zone's hash, no entry is a **self-link**, and the summary counts agree with the
    entries. Link-cycle/self-link coverage is **preserved** (caught upstream by `resolve_link_target`;
    cyclic links are skipped in `plan::run` and counted **`failed`** — never `selected`/`omitted` — in
    the link profile). Additive, provenance-only — **no compiler semantics changed**; CORE.1 341/0/0.
    Tests in `tests/manifest.rs` (`alias_map_validate_accepts_clean_map`,
    `alias_map_validate_rejects_dangling_link`, `alias_map_validate_rejects_hash_mismatch`,
    `alias_map_validate_rejects_self_link`, `alias_map_validate_rejects_count_mismatch`,
    `cyclic_links_counted_failed_not_selected_or_omitted`) + `fixtures/minimal/link_cycle.zi`. **280 tests.**
  - **T12.4d — `backward` evidence ✅** — new `source_profile.backward_evidence` block (schema bumped
    to `zic-rs-compile-manifest-v5`), recording `backward` as a **detected/claimed/status evidence
    axis — never a boolean**. *Detection* is hash-backed only: an admitted backward source
    (`--backward-source <path>`) whose bytes ARE in the build → `present`, NOT in the build → `absent`
    (bounded to that artifact); no admitted source → `unknown`. *Claim* is the bare `--backward
    included|excluded`. `status()` reconciles them (mirrors T12.2's `version_status`). **Never inferred**
    from alias count, alias names, filenames, link target names, or selected/omitted/failed counts.
    Provenance-only — **no compiler/link/alias-map behaviour changed**; CORE.1 341/0/0. A deliberate
    **`source_profile` extension seam** holds `backzone`/`rearguard`/`vanguard` axes later without
    reshaping. +9 tests; **290 tests.** (See the dedicated section below.)
- **T12.5 — `backzone` / `PACKRATLIST` / `DATAFORM` source-variant evidence (reference-first):**
  - **T12.5a — source-variant policy inventory ✅** — the membership-vs-encoding taxonomy + non-inference
    findings (this doc's "Source Variant Policy Inventory"); no schema bump.
  - **T12.5a.1 — reference-pin requirement ✅** — the hard gate (`reports/t12_5a1-…md`): no variant
    *evidence* until the upstream files are admitted + SHA-256-pinned.
  - **T12.5a.2 — reference admission ✅** — fetched pristine tzdb 2026b, **verified its OpenPGP signature**
    (tz key `7E37…7E34`), SHA-256-pinned the reference set; gate **OPEN → `lifted_for_2026b`**.
  - **T12.5a.3 — all-IANA release-admission matrix ✅** — the versioned admission doctrine (see the
    matrix section); support is per-release + per-feature-profile, never globally inferred.
  - **T12.5b — `backzone` (`PACKRATDATA`) membership evidence ✅** — `source_profile.backzone_evidence`
    (schema `…-v6`), hash-anchored to the pinned 2026b `backzone`. (See the dedicated section below.)
  - **T12.5c — `PACKRATLIST` backzone-scope evidence ✅** — `source_profile.packratlist_evidence`
    (schema `…-v7`); detection only from an admitted **generation-policy input** matching the pinned
    2026b `zone.tab`, **never** from compile `source_inputs` (the category boundary). (See below.)
  - **T12.5d — `DATAFORM` (main/vanguard/rearguard) encoding evidence ✅** — `source_profile.dataform_evidence`
    (schema `…-v8`): detected by hash-match against the pinned `main.zi`/`vanguard.zi`/`rearguard.zi`
    via `source_inputs` membership (category-correct — the `.zi` *are* compile sources), claim-only
    otherwise, never content-inferred; adds `recipe_hash` + `generated_from` for the generated artifacts.
    Removed the last `build_profile` `rearguard`/`vanguard` `"unknown"` stubs — the arc is closed. (See below.)
- **T12.6 — profile in reports ✅:** `support-report` / `structural-report` surface the static
  provenance/gate state (see the T12.6 section).

> **Doctrine:** additive provenance + checkable facts — **zero behaviour change** to the default
> compile; profile names are **structured fields, never vague labels**; reference-first per substep.
> **Source identity ≠ profile identity ≠ membership**: links are recorded as *what this build
> produced*; `backward`/`backzone`/… membership is never inferred from alias counts or filenames.

## Link / alias identity — reference inventory (T12.4a)

**What a link *is* (pinned from `zic.c` 2026b).** A `Link TARGET LINKNAME` line makes `LINKNAME` an
alias of canonical zone `TARGET`. `make_links` (zic.c §985): links are sorted by name; **a later
`Link` with the same `LINKNAME` supersedes an earlier one** (last-wins dedup); links whose target is
itself a not-yet-made link are deferred and retried, so **link-to-link chains resolve** (worst case a
full pass per chain link); a **link cycle** (`"Link … is part of a link cycle"`) and a **self-link**
(`link X targets itself`) are hard errors. `dolink(target, linkname, staysymlink)` materialises one
link — `-s`/symlink mode emits a (relative-if-possible) symlink, otherwise a hard link / **byte copy**
(`dolink` falls back to copying the target's bytes when linking is unavailable). So a link is either a
*reference* (symlink) or a *duplicate* (copy) of the target's TZif bytes — the jiff#258 distinction.

**How `backward` contributes.** In the IANA distribution the file literally named **`backward`** is a
**pure `Link`-line file**: it maps **legacy / renamed identifiers** (e.g. `US/Eastern → America/New_York`,
`Asia/Calcutta → Asia/Kolkata`) onto current canonical zones. It defines **no zones** of its own. In
the single concatenated **`tzdata.zi`**, those `Link` lines are merged inline with everything else, so
**you cannot tell from `tzdata.zi` alone whether `backward` was included** — the links are simply
present (or not). In the per-file layout, `backward` is one input file among many. **Key consequence
(the T12.4d hard rule): the *presence of many aliases* does not prove `backward` was the source** —
links can come from many source arrangements — so `backward` membership must come from hash-backed
evidence or an explicit claim, never from alias count or a filename. `backward` is a **source-set /
profile axis** (included / excluded / unknown), **not** a correctness defect — the neutral wording is
*"backward included/excluded/unknown"*, never "legacy aliases enabled/disabled".

**What zic-rs records today.** Two surfaces already track links:

- **`CompileReport.links_written: Vec<LinkReport { link_name, target, mode }>`** (`src/lib.rs`) — every
  link materialised this run, with its `LinkMode` (`Copy`/`Symlink`). The compile path already enforces
  the safety rules: precise **link-cycle** vs **missing-target** diagnostics (`resolve_link_target`),
  atomic no-clobber, and traversal-safe names (`ZIC008`).
- **`alias-map.json`** (`manifest::AliasMap`, schema `zic-rs-alias-map-v1`, `--alias-map`) — per
  identifier `zone`/`link`, link target + target hash + `materialised` (`copy`/`symlink`), and the
  summary `identifiers` / `canonical_zones` / `links` / `duplicated_byte_links` (the jiff#258 figure).
- The **compile manifest** (`zic-rs-compile-manifest-v8`) records `links_materialized` (names only) in
  its `compile` block, `link_mode` in `build_profile`, the full `link_profile` block (T12.4b: counts +
  `alias_map_sha256` + selected/omitted link hashes), and the `source_profile.backward_evidence` axis
  (T12.4d).

**Four distinct identifier concepts (kept separate — a subtle trap).** *Requested* (what the
user asked for, e.g. `--zone UTC`) ≠ *selected zones* (canonical zone records compiled) ≠ *selected
links* (link identifiers materialised) ≠ *resolved targets* (where requested links point). zic-rs
resolves a requested **link name** to its canonical zone, compiles the **target**, and materialises
the **link** — so `--zone UTC` produces a compiled `Etc/UTC` plus a written `UTC` link, never an empty
build. The link counts below count these correctly rather than conflating them.

**Gaps closed by T12.4b; remaining for T12.4c–d.** **T12.4b ✅** added the `link_profile` block —
link/zone/alias **counts** as identity, the **selected vs omitted vs failed** split, and the
**`alias_map_sha256`** + `selected`/`omitted_links_sha256` hashes binding the manifest to its alias
map. **T12.4c ✅** added `AliasMap::validate()` (called by `build()`): every link entry maps to a
present zone with a matching hash, no self-link, counts agree — *missing target fails*; cycle/self-link
coverage preserved (cyclic links counted `failed`). **T12.4c ✅** as above. **T12.4d ✅** added the
`backward` evidence axis (below). All additive provenance, no compiler-semantics change.

## `backward` evidence axis (T12.4d) — the why before the fields

**The trap this exists to prevent.** The alias surface is **output identity, not source provenance**.
A build can expose legacy-looking aliases (`US/Eastern`, `Asia/Calcutta`, …) **without proving** that
the tzdb `backward` source file participated in the build — links can come from many source
arrangements, and in the concatenated `tzdata.zi` the `backward` links are inline-merged and invisible
as a set (see the T12.4a inventory). Conversely, the *absence* of such aliases does **not** prove
`backward` was excluded. So `backward` must **never** be a boolean, and must **never** be inferred from
the alias/link surface. T12.4d therefore records it as an **evidence axis** with `detected`, `claimed`,
and reconciled `status` — the same moral law as T12.2's tzdb-version reconciliation: *a detected fact
and a claimed fact are not the same kind of thing, and a manifest becomes dangerous the moment it turns
a claim into a fact.*

**The fields** (`source_profile.backward_evidence`):

```
detected         = present | absent | unknown
claimed          = included | excluded | none
status           = <reconciliation, see below>
evidence_sha256  = <hash of the admitted backward source, or null>
```

`status` values: `unknown_no_evidence` · `claimed_present_unverified` · `claimed_absent_unverified` ·
`detected_present` · `detected_absent` · `detected_matches_claim` · `detected_contradicts_claim`.

**Evidence-admission law (enforced in `BackwardEvidence::reconcile`).** `backward` status is admitted
**only** from (1) **hash-backed source evidence** or (2) an **explicit claim**. It is **never** inferred
from: alias count · alias names · output filenames · source filenames alone · link target names ·
selected/omitted/failed link counts · the presence of legacy-looking identifiers.

- **Detection** (hash-backed): the caller admits a backward source with **`--backward-source <path>`**;
  the manifest hashes that file and checks whether its **bytes participate in the build** (i.e. its
  SHA-256 is among `source_inputs`). Present → `detected: present`; not present → `detected: absent`.
  No admitted source → `detected: unknown`. The filename is irrelevant — only the content hash and its
  membership matter.
- **Claim** (bare assertion): **`--backward included|excluded`** is recorded as `claimed` and **never
  promoted to detection** (an unverified claim is labelled `claimed_*_unverified`).

**Two honesty boundaries worth stating explicitly:**

1. **Absence is bounded to the admitted artifact, against a complete input universe.** `detected:
   absent` means *"the file you admitted as the backward source is not among this build's inputs"* —
   it is **not** a universal "no backward data exists anywhere" claim. The subtlety (a place provenance
   systems lie by accident): *presence is easy, absence is hard* — you may only assert absence if you
   know the universe you searched is complete. zic-rs sidesteps this by only ever checking against
   **`source_inputs`, which is by definition the build's exhaustive input set**, and by emitting `absent`
   **only** when a backward source was explicitly admitted and hash-verified missing from it. The two
   "we didn't find it" states are therefore kept distinct:
   - **no admitted evidence → `unknown`** (the honest *"not detected because nothing was admitted"* — we
     never silently upgrade this to `absent`);
   - **admitted source, hash-verified not in the complete input set → `absent`** (a real,
     complete-source-set absence determination — but still bounded to *that artifact*: backward data
     could in principle be present under a *different* input the caller did not point at).
   This is the same omitted-link-universe discipline as T12.4b: you can only claim absence of what you
   can exhaustively search for.
2. **Detection proves participation, not semantic identity.** `--backward-source` verifies the admitted
   *artifact's bytes* are in the build; it does **not** assert the file is genuinely the IANA `backward`
   file. Matching against a pinned reference `backward` hash (true semantic identity) is a noted future
   enhancement, not a v1 claim.

**Extension seam.** The block lives under `source_profile` specifically so `backzone`/`rearguard`/
`vanguard` can later gain the *same* detected/claimed/status shape without reshaping the manifest — a
seam, not a premature abstract "variant framework". `backzone` is especially sensitive (the IANA
`backzone` file carries data outside normal tzdb scope and its links can supersede `backward`), so it
gets its own reference-first substep (T12.5), never folded in here.

**What T12.4d does *not* claim:** it does **not** mean zic-rs "supports `backward` fully", "proves
backward compatibility", or "handles backzone/rearguard/vanguard". The exact claim is: *T12.4d records
`backward` as a detected/claimed/status evidence axis, admits `included`/`excluded` only from
hash-backed source evidence or an explicit claim, and preserves `unknown` when evidence is
insufficient* — proven by positive tests (present/absent/match/conflict/claim-only) and negative tests
(never inferred from alias count, alias names, source/output filenames, or link-profile counts).

## Source Variant Policy Inventory (T12.5a) — reference-first; no implementation

> **Status: reference-first inventory, like T9.1 / T10.1 / T11.1 / T12.1 / T12.4a.** This pins the
> upstream IANA/tzdb *source-variant* build axes and decides what the manifest may claim — **before**
> any backzone/rearguard/vanguard implementation. **No compiler / link / alias-map behaviour change;
> no schema bump.** (The only code change in this substep is a correctness fix: `backward` was briefly
> double-listed as a `build_profile` `"unknown"` stub *and* a `source_profile.backward_evidence` axis;
> the stub is removed — `source_profile` is authoritative.)

### ⚠ Provenance — pinned in T12.5a.2 (was documented-only in T12.5a)

When this inventory was **first written (T12.5a)**, the canonical tzdb *distribution* artifacts that
define these axes — the **`Makefile`** (`DATAFORM`/`BACKWARD`/`PACKRATDATA`/`PACKRATLIST`/`REDO`/
`ZFLAGS`), **`theory.html`**, and the data files **`backzone`**/`vanguard.zi`/`rearguard.zi` — were
**not present in this environment**, so the inventory was documented from the build model,
*not byte-pinned*. **T12.5a.2 then admitted them:** the pristine IANA **tzdb 2026b** complete
distribution was fetched, its detached signature **verified** (GOODSIG/VALIDSIG by the published tz key
fingerprint `7E37…7E34`), and every file SHA-256-pinned — see
[`reports/t12_5a2-reference-admission.md`](../reports/t12_5a2-reference-admission.md). The Makefile
facts below are now **byte-confirmed**: `DATAFORM=main` / `BACKWARD=backward` / `PACKRATDATA=` /
`PACKRATLIST=` (empty) by default, and the subset-vs-all `PACKRATLIST` distinction. (Still locally
true: `zic` itself has no variant concept — confirmed by `zic.c` containing no
`DATAFORM`/`PACKRAT`/`rearguard`/`vanguard`/`backzone` tokens; variants are a distribution/Makefile
concern.) **Reference-pin gate (T12.5a.1 → T12.5a.2):** the requirement receipt
[`reports/t12_5a1-reference-pin-requirement.md`](../reports/t12_5a1-reference-pin-requirement.md) is now
**`lifted_for_2026b`** — T12.5b–d are unblocked **for that pinned reference only** (version-scoped; a
later release re-opens the gate). **Admission ≠ implementation:** no variant behaviour is implemented
or claimed yet (the reports still show `source_variant_behavior_implemented: false`).

### The two distinct kinds of axis (the core taxonomy)

Source variants are **not one knob**, and conflating them is the trap:

1. **Source-set *membership*** — *which identifiers/zones/links exist in the input set.* Knobs:
   **`BACKWARD`** (legacy/renamed-name `Link`s) and **`PACKRATDATA`/`PACKRATLIST`** (the `backzone`
   pre-1970 / out-of-scope data, all or a selected subset). Membership changes the *identifier
   universe*; **`backzone` can *supersede* mainline zones/links**, not merely add — so it is more
   dangerous than `backward` (the tzdb theory file describes `backzone` as outside the database's
   normal scope and less reliable).
2. **Encoding / format *variant*** — *the same data in a different source encoding.* Knob:
   **`DATAFORM = main | vanguard | rearguard`** (staged-adoption formats: `vanguard` uses the newest
   `zic` features earliest, `main` is the default and waits so downstream can upgrade `zic`,
   `rearguard` waits longest — rearguard consumers may also want `ZFLAGS = -b fat`). The three are
   *essentially the same data*; the difference is which source-text features are used.

**Critical non-inference consequence:** you **cannot** infer `DATAFORM` from output content — e.g.
"uses negative `SAVE` ⇒ vanguard" is **wrong**, because mainline 2026b already uses negative `SAVE`.
Encoding-variant detection must be **hash-backed** (matching a known `vanguard.zi`/`rearguard.zi`) or an
**explicit claim** — never a content heuristic. Likewise membership (`backward`/`backzone`) is never
inferred from alias counts, alias names, filenames, link counts, or output byte shape (the T12.4d law,
extended to all source variants).

### The upstream axes (from the documented tzdb build model — re-verify when files available)

| Axis | Upstream knob (`Makefile`) | What it controls | Kind | T12.5a treatment |
|------|----------------------------|------------------|------|------------------|
| `backward` | `BACKWARD=backward` (default) / empty | legacy/renamed-name compatibility `Link`s | membership | **already an evidence axis (T12.4d)** |
| `backzone` (all) | `PACKRATDATA=backzone` | pre-1970 / out-of-scope "packrat" zones+links; can supersede mainline | membership | inventory only → reserved |
| `backzone` (subset) | `PACKRATLIST=zone.tab` (or empty=all) | *which* packrat entries are admitted (selected vs all) | membership | inventory only → reserved |
| `dataform` | `DATAFORM=main\|vanguard\|rearguard` | source *encoding* (staged feature adoption) | encoding | inventory only → reserved |
| right/posix | `REDO=` (posix/right leap set) | leap/`right` build set | output tree | partly **T11** (`-L`/`right`); relation noted |
| fat/slim | `ZFLAGS=-b fat` etc. | TZif emission shape (reader-compat) | emission | **T10** (`-b`/`--emit-style`); relation noted |

**Default IANA profile (version-aware — defaults can change across releases, so record, never assume):**
`DATAFORM=main` · `BACKWARD=backward` (included) · `PACKRATDATA=` (none) · `PACKRATLIST=` (none).

### `backzone` is not a boolean (the key point)

Because `PACKRATLIST` can select a *subset*, `backzone` membership has more states than included/excluded:
`not_used` · `used_subset_via_packratlist` · `used_full` · `claimed_not_hash_backed` · `unknown`. And —
mirroring the T12.4d absence discipline — `absent` may only be asserted against a **complete, admitted
source universe**; otherwise it is `unknown`. A manifest must also *say what it means*: "this build
included out-of-scope/packrat historical data," not merely `backzone=true`.

### Target manifest model (sketched, NOT implemented this substep)

`source_profile` is the extension seam. The intended (future) shape, **not built yet**:

```
source_profile:
  backward_evidence: { detected, claimed, status, evidence_sha256 }   # ✅ T12.4d (done)
  # reserved future siblings (T12.5 implementation substeps):
  backzone_evidence: { detected, claimed, status, evidence_sha256, packratlist_policy }
  source_variant_policy: { dataform: {detected,claimed,status}, packratdata, packratlist }
```

No fields are added now (per the "inventory before implementation" rule); this records the decided
direction so the schema does not get sketched prematurely.

### What is already represented vs reserved vs not-claimed

| Concern | Status |
|---------|--------|
| `backward` membership evidence | **Represented** — `source_profile.backward_evidence` (T12.4d), hash-backed/claim-only. |
| emission shape (fat/slim) | **Represented** — `build_profile.emit_style` (T8/T10). |
| leap / `right` output tree | **Represented** — `build_profile.output_tree` + `leap_source` (T11/T12.2). |
| `backzone` (`PACKRATDATA`) membership evidence | **Represented** — `source_profile.backzone_evidence` (T12.5b), hash-anchored to the pinned 2026b `backzone`. |
| `backzone` *scope* (`PACKRATLIST` subset) evidence | **Represented** — `source_profile.packratlist_evidence` (T12.5c), from an admitted generation-policy input only; never from compile `source_inputs`. |
| `DATAFORM` (main/vanguard/rearguard) encoding evidence | **Represented** — `source_profile.dataform_evidence` (T12.5d), hash-backed against the pinned 2026b `.zi` artifacts + `recipe_hash`; never content-inferred. |
| `build_profile` source-variant placeholders | **None** — as of T12.5d every source-variant axis is a `source_profile` evidence axis; `build_profile` describes only how the run emitted. |

### Ladder note (numbering discipline)

A review suggested moving variant *implementation* to a new **T13** campaign. We are **not**
renumbering: the contiguous ladder rule (T0→T21, execution == numeric order) is a standing hard
constraint, and **T13 is already "Warning & diagnostic parity."** So variant-source implementation
stays under **T12** as future substeps (**T12.5b** backzone evidence · **T12.5c** PACKRATDATA/PACKRATLIST
policy · **T12.5d** DATAFORM reconciliation), with **T12.6** profile-in-reports — all reference-first.
T12.5a (this substep) is the inventory/decision bridge.

## T12.6 — profile / gate visible in reports

**Done.** The static **provenance/capability statement** is now surfaced (text + JSON) by both
`support-report` and `structural-report` (their schemas bumped `…-v1` → `…-v2`; additive section). A
single shared source in `src/manifest.rs` (`provenance_block_json`/`provenance_block_text`, +
`SOURCE_VARIANT_GATE_STATUS` / `SOURCE_VARIANT_BEHAVIOR_IMPLEMENTED` / `SOURCE_VARIANT_BLOCKED_SUBSTEPS`
/ `SOURCE_VARIANT_UNPINNED_FILES` constants) emits:

- `manifest_schema` — the `zic-rs-compile-manifest-v8` the tool produces;
- `source_variant_reference_pin_gate` — **`lifted_for_2026b`** since T12.5a.2 (was `open` at T12.5a.1);
- `blocked_substeps` / `unpinned_required_files` — now **empty** (gate lifted, 2026b reference pinned);
- `source_variant_behavior_implemented: false` + the non-claim note (admission ≠ implementation).

**Honesty boundary (deliberate):** the reports surface the **static** capability/gate state, **not** a
per-run `build_profile`/`link_profile`/`backward_evidence` — a `support-report`/`structural-report` run
is *not* a configured output compile, so it has no honest per-run profile of its own. Those remain the
`compile --manifest` artifact, which the block **points to** (`per_run_profile: "see compile
--manifest"`) rather than fabricating. T12.6 itself changed no compiler behaviour or manifest schema
(the manifest has since moved v5 → v6 → v7 → v8 via T12.5b/c/d); the gate constants are single-sourced so the reports
and `reports/t12_5a1-reference-pin-requirement.md` / `…t12_5a2…` cannot drift. Tests: the
`support_report` / `structural_report` JSON-shape tests assert the provenance block (gate
`lifted_for_2026b` since T12.5a.2, `source_variant_behavior_implemented: false`, manifest schema v8).

## T12.5b — `backzone` / `PACKRATDATA` evidence axis (version-scoped to admitted 2026b)

**Done.** `source_profile.backzone_evidence` records `backzone` (`PACKRATDATA`) as a **source-membership
evidence axis, never a boolean** — mirroring `backward_evidence`, but **hash-anchored to the pinned
reference release**. Schema bumped **`zic-rs-compile-manifest-v5` → `-v6`** (a real new block).

- **Detection (hash-backed, version-scoped):** `BackzoneEvidence::reconcile` checks whether
  `REF_2026B_BACKZONE_SHA256` (`63fb39ad…`, the 2026b `backzone` admitted + signature-verified in
  T12.5a.2) appears among `source_inputs`. **`present`** iff that exact file participated; otherwise
  **`unknown`** — *presence is hash-backed; absence is never asserted* (a concatenated `.zi` can merge
  backzone, so non-presence ≠ absent). This is the matrix discipline in code: a *different* release has
  a different `backzone` hash and needs its own admission.
- **Claim:** the bare `--backzone {included|excluded}` flag (provenance-only — never affects
  compilation/linking), recorded separately and **never promoted** to detection. `status()` reconciles
  them (`detected_present` / `detected_matches_claim` / `detected_contradicts_claim` /
  `claimed_present_unverified` / `claimed_absent_unverified` / `unknown_no_evidence`).
- **Never inferred** from aliases, zone names, filenames, link counts, output byte shape, pre-1970
  differences, or `DATAFORM` (`reconcile` only reads `source_inputs` + the reference hash + the claim).
- **Scope:** whether `backzone` participated *at all*. The **subset-vs-all (`PACKRATLIST`)** distinction
  is **T12.5c**; **`DATAFORM`** is **T12.5d**. `source_variant_behavior_implemented` stays **`false`** —
  this is evidence profiling, not behaviour (no output/compiler/link/alias-map change; CORE.1 341/0/0).
- **Args generalised:** `BackwardArgs` → **`SourceVariantArgs`** (`Default`-friendly; T12.5c/d add fields
  with zero call-site churn). Real-run: ordinary compile → `unknown_no_evidence`; compile with the pinned
  `backzone` as an input + `--backzone included` → `detected_matches_claim` (`evidence_sha256` = `63fb39ad…`).

## T12.5c — `PACKRATLIST` backzone-scope evidence axis (version-scoped to admitted 2026b)

**Done.** `source_profile.packratlist_evidence` records the **scope** of an included `backzone` —
*all* pre-1970 data vs a *subset* selected by the upstream `PACKRATLIST` knob — as a
detected/claimed/status evidence axis. Schema bumped **`zic-rs-compile-manifest-v6` → `-v7`**.

> **The category boundary (the finding that shaped the design).** `PACKRATLIST` is a
> **generation-policy selector, not a `zic` compile source.** Upstream, the `Makefile`/`ziguard.awk`
> apply `PACKRATLIST` (a `zone.tab`-style table of which pre-1970 zones to keep) *before* `zic` runs —
> its effect is **baked into the produced source**. And `zone.tab` itself **is not a compilable `zic`
> input** (it is a tab-separated geography table, not `Rule`/`Zone`/`Link` — `zic-rs --input zone.tab`
> fails, as does reference `zic`). Therefore **zic-rs does not search ordinary `source_inputs` for
> `zone.tab`** — doing so would be a *category error* (conflating compile evidence with
> generation-policy evidence). Subset detection is admitted **only** through an explicit
> *generation-policy* evidence path: a `--packratlist-source <path>` whose SHA-256 matches the **pinned
> 2026b `zone.tab`** (`REF_2026B_ZONE_TAB_SHA256` = `4d8e389e…`, admitted in T12.5a.2) **and** with
> `backzone` present. (Catching this before the seal — rather than encoding the wrong model — is the
> key win of this substep: the precise line between *compile evidence* and *generation-policy
> evidence*.)

- **Detection (hash-backed, policy-input only):** `PackratlistEvidence::reconcile(claim,
  admitted_policy_input_sha256, REF_2026B_ZONE_TAB_SHA256, backzone_present)` →
  `detected: subset_from_policy_input` **iff** the admitted policy-input hash equals the pinned 2026b
  `zone.tab` **and** `backzone` is present; otherwise `unknown`. The ordinary compile `source_inputs`
  are **never** consulted for this axis.
- **Claim:** `--packratlist {full|subset|none}` (provenance-only), recorded separately and never
  promoted to detection. `status()` reconciles them
  (`detected_subset_from_policy_input` / `detected_matches_claim` / `detected_contradicts_claim` /
  `claimed_full_not_hash_backed` / `claimed_subset_not_hash_backed` / `claimed_none_not_hash_backed` /
  `unknown_no_evidence`). `full`/`none` are claim-only — **absence proves nothing** (a subset that
  happens to retain a zone is indistinguishable from `full` by output shape).
- **Never inferred** from `zone.tab` *presence* in a build dir, output zone counts, aliases, link
  counts, filenames, pre-1970 differences, or "looks like global-tz" output shape.
- **Non-claim (explicit):** *T12.5c does not implement `PACKRATLIST` build behavior. It records
  `PACKRATLIST` subset-policy evidence.* `source_variant_behavior_implemented` stays **`false`** (no
  output/compiler/link/alias-map change; CORE.1 341/0/0).
- **Cleanup:** removed the now-contradictory `build_profile.backzone "unknown"` stub (backzone is a
  `source_profile` axis since T12.5b — the same de-duplication done for `backward` in T12.5a). At this
  point only `rearguard`/`vanguard` (`DATAFORM`) remained as `build_profile` placeholders; **T12.5d
  then removed those too**, closing the arc (`build_profile` now carries no source-variant placeholder).
- **Release-scoped global-tz note (folded doctrine):** the equivalence
  `PACKRATDATA=backzone PACKRATLIST=zone.tab` ≡ a global-`tz`-style build is **release-scoped — it must
  be *tested*, never claimed by inference**; source hashes are distinguished by *class* (standalone
  file · concatenated input · generated artifact · extracted member), not assumed one-file-per-role.

Real-run: ordinary compile → `unknown_no_evidence`; `--packratlist subset` (no source) →
`claimed_subset_not_hash_backed`; admitted `--packratlist-source` = pinned 2026b `zone.tab` + backzone
present → `detected_subset_from_policy_input`.

## T12.5d — `DATAFORM` encoding evidence (version-scoped to admitted 2026b) — closes the T12.5 arc

**Done.** `source_profile.dataform_evidence` records the *encoding form* — `main` / `vanguard` /
`rearguard` — as a detected/claimed/status evidence axis. Schema bumped **`zic-rs-compile-manifest-v7`
→ `-v8`**.

> **The clean category model (the doctrine anchor).** `backzone` = source-*membership* evidence ·
> `zone.tab` = generation-*policy* evidence · `vanguard.zi`/`main.zi`/`rearguard.zi` =
> **generated-artifact** evidence · `DATAFORM` = the *encoding policy* those artifacts realise. The
> decisive distinction from T12.5c: **the three `.zi` artifacts *are* compilable `zic` sources**
> (unlike `zone.tab`, a non-compilable policy table), so DATAFORM detection from `source_inputs`
> membership is **category-correct** — it mirrors `backzone`, not `packratlist`. If you compiled
> `vanguard.zi`, its bytes are a `source_input`, and that hash is the only honest signal of the form.

- **Detection (hash-backed, version-scoped):** `DataformEvidence::reconcile` scans `source_inputs` for a
  file whose SHA-256 equals one of the pinned 2026b artifacts — `REF_2026B_MAIN_ZI_SHA256` (`e0225823…`),
  `REF_2026B_VANGUARD_ZI_SHA256` (`49e16da4…`), `REF_2026B_REARGUARD_ZI_SHA256` (`91c4f362…`), all
  admitted in T12.5a.2. A match → that form; otherwise `unknown` (a zishrunk `tzdata.zi`, a concatenated
  build, or another release is not hash-recoverable).
- **`recipe_hash` + `generated_from` (generated-artifact provenance):** the `.zi` are *derived*, so a
  detected form carries more than "hash matched" — it carries **how it was generated**. `recipe_hash` =
  `sha256(archive_sha256 · Makefile sha256 · ziguard.awk sha256 · generation command · toolchain)`,
  **hashed as raw bytes, never line-ending-normalized** (a transformed copy is a different artifact).
  `generated_from` = `"tzdb-2026b"`. (Inputs pinned in `reports/t12_5a2-…md`, incl. the now-pinned
  `ziguard.awk` = `e4600a23…`.)
- **Claim:** the bare `--dataform {main|vanguard|rearguard}` flag (provenance-only), recorded separately
  and never promoted. `status()` reconciles them (`detected_matches_claim` / `detected_contradicts_claim`
  / `detected_only` / `claim_only` / `unknown_no_evidence`). There is intentionally **no
  `--dataform-source`**: the `.zi` *are* compile inputs, so admitting one you did not compile would
  assert provenance for bytes the build never used.
- **Never content-inferred:** not from negative `SAVE` (mainline 2026b already uses it, so
  "negative SAVE ⇒ vanguard" is provably wrong), nor any other syntax, output shape, zone names,
  filenames, `PACKRATLIST`, or `backzone`. `reconcile` reads only file hashes. **`ziguard.awk` is not a
  general converter** (it targets *current* tzdata, is neither idempotent nor reversible) — the `.zi`
  are recorded as generated reference artifacts with a recipe, **never** as something zic-rs can
  reproduce or transform.
- **Arc closure:** with DATAFORM now an evidence axis, the last `build_profile` `rearguard`/`vanguard`
  `"unknown"` stubs were removed (as `backward`/`backzone` were before). `build_profile` now describes
  purely *how this run emitted*; **all** source-variant axes live in `source_profile`.
  `source_variant_behavior_implemented` stays **`false`** — this is evidence profiling, not behaviour
  (no output/compiler/link/alias-map change; CORE.1 341/0/0).

Real-run: ordinary compile → `unknown_no_evidence` (`recipe_hash: null`); compile the pinned `main.zi`
+ `--dataform main` → `detected_matches_claim` (`evidence_sha256 = e0225823…`, `recipe_hash` set,
`generated_from "tzdb-2026b"`); compile the pinned `vanguard.zi` → `detected_only` (form `vanguard`).

## All-IANA Release Admission Matrix (T12.5a.3)

> **Doctrine (the scoped claim).** zic-rs does **not** claim global tzdb support by assumption. It
> **admits each IANA release** through archive provenance, signature/hash verification, a required-file
> inventory, generated-artifact recipes, and a feature-profile classification. **Support is
> release-admitted, not inferred** — claimed *per release* and *per feature profile*, never globally.
> 2026b is the first (and currently only) admitted release; future releases enter through the **same
> admission ceremony** (T12.5a.1 requirement → T12.5a.2 admission), without changing the evidence model.

**Three support layers** (kept distinct; do not conflate):

1. **Archive admission** — fetch + **verify signature** + SHA-256-pin the release (the T12.5a.2 ceremony).
2. **Source-grammar / build-policy classification** — a per-release *feature profile* (which knobs &
   files exist): `Makefile`/`BACKWARD`/`backzone`/`PACKRATDATA`/`PACKRATLIST`/`DATAFORM`/
   `vanguard|main|rearguard.zi`/`zone.tab`/`zone1970.tab`/`leapseconds`.
3. **Compile / output parity** — semantic + structural, where claimed (CORE.1-style, *per release*).

**T12.5 implements layers 1–2 only.** Layer-3 certification is later and per-release.

**Era-aware absence (the design rule).** A release that *predates* an axis records it
**`not_applicable`** — a version-classified absence, **not** `unknown` and **not** a failure.
(`unknown` = "the axis exists for this release era but no evidence was admitted"; `not_applicable` =
"this release predates the axis", e.g. a pre-`DATAFORM`-era release.)

**`ReleaseFeatureProfile` (documented seam, not built now).** Per-release feature presence, so the
manifest never hard-codes 2026b assumptions as universal. The version-scoped `backzone` detector
(anchored to `REF_2026B_BACKZONE_SHA256`) is the first concrete instance of this discipline.

**The matrix** (seeded with one admitted row; boundaries for older releases are **verified before
pinning**, never guessed):

| Release | Archive admission | Signature | Feature profile | Variant axes | Compile parity |
|---------|-------------------|-----------|-----------------|--------------|----------------|
| **2026b** | **pinned** (`tzdb-2026b.tar.lz`, `ffad46a0…`) | **verified** (tz key `7E37…7E34`) | current / full (Makefile byte-confirmed) | `backward`✅ · `backzone`✅(T12.5b) · `PACKRATLIST`✅(T12.5c) · `DATAFORM`✅(T12.5d) | CORE.1 341/341 (1900..2040) |
| 2026c+ | pending | pending | pending | pending | pending |
| 2018d–2026a | future | future | `DATAFORM`-era audit | future | future |
| pre-`DATAFORM` era | future | future | legacy-era audit | some axes `not_applicable` | future |

**Acceptance line (honest, non-overclaiming):** *zic-rs supports admitted IANA tzdb releases via a
versioned release-admission matrix; support is per-release and per-feature-profile, recorded (archive
provenance · signature/hash · required-file inventory · generated-artifact recipe · feature profile),
not globally inferred. 2026b is the first admitted release.*

**Light by design:** 2026b stays the only admitted release; this is doctrine + a one-row matrix +
era-aware vocabulary, **not** an implementation explosion. Future releases add rows via the same
ceremony; the evidence model (hash-backed, claim-only, never-inferred, version-scoped) does not change.

## T12 — CLOSED

The build-profile / manifest-identity arc is **closed**. The closing doctrine — the **evidence-category
taxonomy** (`compile_input`/`policy_input`/`reference_input`/`generated_artifact`/`output_artifact`/
`diagnostic_artifact`/`policy_prose`/`release_note_evidence`), the **release-delta-review requirement**
(+ `release_delta_review_hash`), the **current-release-bias guard**, the **negative-capabilities** list,
the RFC 9636 standard-boundary statement, the reproducibility/`recipe_hash` doctrine, and the reserved
surfaces assigned to their owning milestones (TZif structural validator → T14/T15; safety/rust-platform
policy → T17/T20; packager-integration → T16/T19; one-line machine status + `negative_capabilities` JSON
→ T15) — is recorded in **[`reports/t12-close-receipt.md`](../reports/t12-close-receipt.md)**. It is
documentation only: no behaviour, manifest-schema, or report-schema change.
**Next milestone: T13 — warning & diagnostic parity** (by error layer; class + location before exact wording).

> **Final doctrine line:** *Every byte and every claim must belong to the right evidence category —
> admitted for 2026b, not all releases · evidence, not behaviour · manifest provenance, not TZif
> semantics · hash-backed, not guessed · generated by a pinned recipe, not magic · a warning class, not
> exact wording yet · pristine-upstream, not distro-patched.*
