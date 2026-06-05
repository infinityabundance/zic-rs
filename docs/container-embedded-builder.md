# Container / embedded builder profile (T21)

> For the **image builder** who wants a *small, deterministic, correct* timezone payload **without dragging
> a distro or toolchain into the image** (the Docker multi-stage pattern: build with tools, ship only
> `/zoneinfo`). This is the deployment front door + the genuinely-new *bundle* tooling. It **consumes** the
> already-true facts in `docs/platform-portability.md` (the pure core + copy-mode baseline),
> `docs/effect-boundary-map.md` (no host reads), `docs/drop-in-compatibility-contract.md` (what is/ isn't
> claimed), and the existing `--zone`/`--out`/`--manifest`/`--alias-map`/`release-diff` surfaces — it does
> not duplicate them. Persona reading: the *embedded/appliance builder* packet in `docs/security-personas.md`.
>
> **Status:** T21.1 (this contract) ✅ · **T21.2 ✅** — `zic-rs size-report` + the deterministic
> `bundle_hash` are **shipped** (`src/size_report.rs`, `zic-rs-size-report-v1`, `tests/size_report.rs`).
> Named bundle profiles / `--link-policy` / a hard version-mismatch build-fail / the reader gauntlet remain
> **planned** (marked below); nothing here claims a command that does not exist.

## 1. Claim

zic-rs produces **minimal, deterministic, host-free timezone bundles** suitable for containers and embedded
images: the production compile path is **pure** (explicit source bytes → TZif bytes; no fs/process/env, no
reference `zic` required — only the *conformance* path needs it), output is **byte-deterministic and
independent of host `TZ`/`LC_ALL`** (`tests/reliability.rs`), and the **copy-mode** baseline (regular files +
copied links, no chmod/chown/symlink, explicit `--out`, no implicit system install) is the portable default
for restricted / non-Unix targets.

## 2. Bundle profiles (the selection vocabulary)

| Profile | Meaning | Today |
|---|---|---|
| `utc-only` | just `Etc/UTC` (+ aliases) — the minimal image | ✅ via `--zone Etc/UTC` |
| `single-zone` | one named zone + its needed links | ✅ via `--zone <name>` (repeatable) |
| `app-list` | an explicit zone list a service uses | ✅ via repeated `--zone` (a `--zone-list <file>` convenience is **T21.2-planned**) |
| `zone1970` / `zonenow` | the post-1970 / now-and-future selection | ▷ **planned** (needs the `.tab` selection wiring; cross-ref `aux-table-validate`) |
| `full-canonical` / `full-with-links` | every canonical zone (± links) | ✅ via compiling `tzdata.zi` (the CORE.1 set) |

A **named `--bundle-profile`** flag that encodes these (plus `--link-policy include-needed`) is **T21.2** —
today the same outcomes are reachable with `--zone` + `--out` + `--alias-map`/`--manifest`. The bundle
**artifact** is: the zoneinfo files + `manifest.json` + `alias-map.json` + the source hash + the admitted
tzdb release + the zones/links **included** *and* **omitted** (the omitted set is recorded, never silent).

## 3. Container recipe (pattern, not a pinned base image)

A documented multi-stage pattern — **build** stage runs `zic-rs compile --input <tzdata.zi> --out /tz`
(tools present); **final** stage is distroless/static with only the compiled TZif copied in and `ENV TZ=…`.
zic-rs ships the *pattern + the inputs that make it reproducible* (source hash, release, profile), not a
base image. The recipe **inputs** (what a builder pins): source `.zi` + its hash · admitted tzdb release ·
emit-style · zone selection · the resulting bundle hash (§7).

## 4. No-host-contamination guarantee

Input source is **explicit** — the library never silently reads `/usr/share/zoneinfo/tzdata.zi`; the
manifest records the source hash; a builder can reconcile a **claimed vs detected** tzdb version
(`--tzdb-version`, T12.2 — `version_status` flags a mismatch). This kills "works on my host tzdata." A
**hard build-fail on a version/hash mismatch** (vs just recording it) is **T21.2-planned**.

## 5. Cross-build / no-`zic` mode

The production `compile` path is **pure Rust, no reference `zic` required** (`docs/effect-boundary-map.md`);
only the *conformance* path (`compare`/`structural-report`/`semantic-report`) needs `zic`/`zdump`. So a
cross-build / scratch image can compile timezone data with **no host timezone toolchain at all**.

## 6. Copy-mode = the portable baseline (← T16.4 Redox-lesson fold)

For containers / embedded / restricted / non-Unix targets, the portable baseline is **copy links · regular
files · no chmod/chown · explicit `--out` · no implicit system install** — symlink/`-m`/`-u` are Unix-only
and `cfg`-gated (fail closed off-Unix; `docs/platform-portability.md`). A valid minimal bundle needs none of
the Unix-only install features.

## 7. Deterministic bundle hash ✅ (T21.2)

`bundle_hash` = a deterministic SHA-256 over the **sorted** per-file `relpath\0content-hash` lines of the
output tree (symlinks contribute `relpath\0symlink:<target>`, so link structure is captured), reusing
`src/hash.rs`. **Order-independent** (sorted before hashing) → same tree gives the same hash regardless of
readdir order; any byte change changes it (`tests/size_report.rs::bundle_hash_is_deterministic_and_sensitive`).
Companioning it with `source_hash · compiler_version · build_profile` and surfacing as OCI image labels
(`org.opencontainers.image.tzdb.version`, …) is a builder-side convention.

## 8. `zic-rs size-report --out <dir>` ✅ (T21.2)

A **read-only** report over a produced output tree (`src/size_report.rs`, schema `zic-rs-size-report-v1`):
`tzif_files` · `symlink_links` · `other_files` · total + TZif bytes · largest TZif · the v1/v2/v3/v4
version histogram · `footer_present` · `bundle_hash`. Carries the provenance block + the non-claim; a
missing tree is a config error (exit 1), never a panic. **Honest classification:** it reports the tree *on
disk* and does **not** distinguish a zone from a copy-mode link (a copied link is a byte-identical TZif —
that needs `alias-map.json`, a future cross-reference); a symlink is a link, a TZif-parsing regular file is
`tzif_files`, everything else is `other_files`.

## 9. Update-impact report (reuses `release-diff`)

`release-diff --old <prev> --new <new>` (with a `--zone-list` filter, T21.2) is the **emergency-rebuild
signal** for a bundle: which bundled zones/links changed, past vs future, footer/version — the same typed
`ReleaseChangeKind` axis, oracle absence visible. No new engine.

## 10. Producer ≠ consumer (the Yocto/Poky finding, made a contract)

The vendor lab proved (`T16.5b.15`) that "embedded timezone tooling" splits in two and they must **never be
collapsed**: a **build-host producer** owns `zic` and *compiles* the TZif; a **target-runtime consumer**
ships only precompiled data and may have **no on-device `zic` at all**. zic-rs is the **producer** in this
pattern; a produced image is a **consumer**. A bundle's manifest records which side it is.

## 11. Builder may / may NOT conclude (the embedded/appliance persona)

- **May conclude:** the compile core is pure + host-free; bundles are deterministic and `TZ`/`LC_ALL`-independent; copy-mode is the portable baseline; the included/omitted zone sets + source/release are recorded.
- **May NOT conclude:** that the produced files are **approved for any runtime** without a reader smoke-test (the reader-compatibility gauntlet — glibc/Python/Go/CCTZ/Timelib — is **T21/T22**, not yet run); that the image is **distro-packaged** (that is T21-packaging / RRL-4, not reached); that a bundle is a **universal `zic` replacement** (it is the admitted CORE.1 surface only); that install features exist off-Unix (they fail closed).

## 12. Non-claims

- **No runtime-platform approval** — a bundle compiling is not a guarantee a given runtime accepts it; the reader gauntlet is future (T21/T22, cross-ref `tzif-reader-ledger`, T18.3).
- **No distro-packaging claim** (T21-packaging / `docs/replacement-readiness-ladder.md` RRL-4).
- **No universal replacement** — bundles are the admitted release + selected zones only.
- **No bundle is crash-atomic for the whole tree** (`RISK.INSTALL.1`; per-file durable on Unix).
- `size-report` + `bundle_hash` are **shipped** (T21.2); **named bundle profiles**, `--link-policy`, the hard version-mismatch build-fail, and the reader-compatibility gauntlet remain **planned** — no command is claimed before it ships.
