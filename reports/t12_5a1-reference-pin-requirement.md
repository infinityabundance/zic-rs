# T12.5a.1 — Source-Variant Reference Pin Requirement (receipt / gate)

> **⟢ GATE LIFTED — see [`t12_5a2-reference-admission.md`](t12_5a2-reference-admission.md).** As of
> **T12.5a.2** the pristine IANA tzdb **2026b** reference set has been fetched, signature-verified, and
> SHA-256-pinned, so this gate is now **`lifted_for_2026b`**: T12.5b–d are unblocked **for that pinned
> reference only**. This receipt remains as the *requirement* record (what must be pinned, and why);
> the *admission* record (the actual hashes + provenance + the lift) is the T12.5a.2 receipt. Per the
> version-scoped rule below, a future release re-opens the gate until its own admission.
>
> **Type:** project-invariant receipt. **Not** implementation, **not** a schema bump, **not** a
> behaviour change. This converts the T12.5a honesty caveat ("the tzdb variant source files are not in
> this environment, so the inventory is documented-not-byte-pinned") into a **hard gate**: the
> backzone / rearguard / vanguard *implementation* substeps (T12.5b–d) **may not begin** until the
> exact upstream reference files below are present in the environment and **byte-pinned**.
>
> Cross-links: `docs/build-profile-parity.md` → "Source Variant Policy Inventory (T12.5a)".

## The gate (invariant)

> **No `backzone` / `rearguard` / `vanguard` / `PACKRATDATA` / `PACKRATLIST` / `DATAFORM` *behaviour*
> or *evidence-detection* may be implemented until every required reference file below is admitted into
> the environment and its `sha256` is recorded in this receipt.** Until then the manifest keeps these
> axes as honest `"unknown"` (membership) — never inferred from aliases, filenames, link counts, or
> output byte shape (enforced by construction: the reconcilers do not receive that data, and by the
> `source_variants_not_inferred_*` tests in `tests/manifest.rs`).

**Why this gate exists (the discipline):** zic-rs is reference-first. The variant axes are defined by
the tzdb *distribution* (its `Makefile` knobs + the data files), **not** by `zic.c` (the compiler has
no variant concept — confirmed: `zic.c` 2026b contains no `DATAFORM`/`PACKRAT`/`rearguard`/`vanguard`/
`backzone` tokens). Those distribution files are absent from this environment. Implementing detection
from memory — or worse, from content/filename heuristics — would violate "diagnose, don't hack" and
the non-inference law. So we stop at the boundary and record the requirement instead of guessing.

## Required upstream reference files (to pin before T12.5b–d)

Source of truth: the IANA tzdb release matching the build under audit (the installed `tzdata.zi` here
sniffs as `2026b-dirty`, so a pristine **2026b** tzdb source tarball is the intended reference). Record
each file's `sha256` (and the tarball it came from) when admitted.

| File | Role | Consumed by | sha256 (fill when pinned) |
|------|------|-------------|---------------------------|
| `Makefile` | Defines the variant knobs + their defaults: `DATAFORM=main\|vanguard\|rearguard`, `BACKWARD`, `PACKRATDATA`, `PACKRATLIST`, `REDO`, `ZFLAGS`. The authoritative statement of the default profile and the staged-adoption semantics. | **all** of T12.5b–d (defaults + semantics) | `UNPINNED` |
| `theory.html` | Prose authority on what `backzone` *means* (out-of-scope / less-reliable data, may supersede), and the vanguard/main/rearguard "essentially the same data, staged feature use" framing. | T12.5b (backzone wording), T12.5d (DATAFORM wording) | `UNPINNED` |
| `backward` | Pinned in **T12.4a** as a pure `Link`-line file (legacy/renamed → canonical). Re-pin its release hash so backward *detection* can match content, not filename. | T12.4d hardening → T12.5 (membership) | `UNPINNED` |
| `backzone` | The pre-1970 / out-of-scope "packrat" data file. Its zones+links can **supersede** mainline — the dangerous axis. Hash needed so `PACKRATDATA=backzone` participation is detected by bytes, never by name. | **T12.5b** (`PACKRATDATA`) | `UNPINNED` |
| `zone.tab` (or the `PACKRATLIST` target) | The selector that admits a *subset* of `backzone` (vs all). Needed to distinguish `used_subset_via_packratlist` from `used_full`. | **T12.5c** (`PACKRATLIST`) | `UNPINNED` |
| `vanguard.zi` | The newest source *encoding* of the release. Hash needed because `DATAFORM` is **not** content-inferable (mainline 2026b already uses negative `SAVE`), so detection must match a known encoding's bytes or an explicit claim. | **T12.5d** (`DATAFORM`) | `UNPINNED` |
| `rearguard.zi` | The old-reader-friendly source *encoding* (rearguard consumers may also want `ZFLAGS=-b fat`). Same hash-backed-detection requirement as vanguard. | **T12.5d** (`DATAFORM`) | `UNPINNED` |
| mainline `*.zi` / `tzdata.zi` | The default-encoding source. The third `DATAFORM` value; the baseline the other two are compared against. | **T12.5d** (`DATAFORM`) | `tzdata.zi` present locally as `2026b-dirty` — **not pristine**; re-pin a clean 2026b before relying on it |

## How to lift the gate (procedure)

1. Obtain the pristine tzdb **2026b** source release (tarball + signature), record its provenance
   (URL / vendored path) and the tarball `sha256` here.
2. For each file above, compute `sha256` (in-house `crate::hash::sha256_hex`) and fill the table; note
   any divergence from the installed `2026b-dirty` source.
3. Pin the `Makefile`'s **default** values for `DATAFORM`/`BACKWARD`/`PACKRATDATA`/`PACKRATLIST` and the
   exact prose for backzone/vanguard/rearguard from `theory.html` — replacing the
   "documented-not-byte-pinned" caveat in `docs/build-profile-parity.md` with byte-pinned facts.
4. Only then begin **T12.5b** (`backzone`/`PACKRATDATA` evidence), reusing the `source_profile`
   extension seam (detected/claimed/status, hash-backed/claim-only).

## Status

**`lifted_for_2026b`** (T12.5a.2). The required files were fetched from IANA, the archive signature
was **verified** (GOODSIG/VALIDSIG by the published tz key fingerprint `7E37…7E34`), and every file
above is **SHA-256-pinned** — full hashes + provenance in
[`t12_5a2-reference-admission.md`](t12_5a2-reference-admission.md). T12.5b–d are therefore **unblocked
for the pinned 2026b reference only**. **Admission ≠ implementation:** no variant behaviour is
implemented or claimed yet (T12.5b begins that); the reports still report
`source_variant_behavior_implemented: false`. A later tzdb release re-opens the gate until its own
admission receipt (version-scoped supply-chain discipline).
