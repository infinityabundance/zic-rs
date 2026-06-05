# Alpine (musl) container package/build-recipe receipt — T23.drop-in-gauntlet.3.alpine

The second Tier-B container recipe, in a **different libc/package world** (musl), executed **ABI-honestly**:
the binary under test is a **musl-built static** `zic-rs`, not a glibc binary mounted into musl. Together with
the Arch (glibc) recipe this is the first **two-ecosystem, two-libc** package-build pair.

- **Date:** 2026-06-03 · **Engine:** `podman run --rm --network=none` · **Image:** `python:3.12-alpine`
  (**Alpine Linux v3.23**, musl userland) · **Recipe:** `reports/drop-in/alpine-recipe.sh`.
- **`binary_origin`:** host-cross-built musl (`rustup target add x86_64-unknown-linux-musl`; `cargo build
  --release --target x86_64-unknown-linux-musl`) → **statically linked** (`ld-musl-x86_64`), sha256
  `a9617888163dc776…`, 2 313 152 B. **`runtime_abi`:** musl.

## The ABI boundary (handled explicitly)

A glibc PIE binary does **not** run in a musl userland; treating that as a compiler-parity failure would
conflate ABI with package-recipe compatibility. So the recipe **demonstrates** the boundary and refuses the
conflation:

| binary | in musl userland | meaning |
|---|---|---|
| glibc `zic-rs` (host PIE) | **exit 127 (exec-fail)** | **expected ABI mismatch — NOT a zic-rs parity failure** |
| **musl `zic-rs` (static)** | runs (`ld-musl`), `--version` exit 0 | the actual test |

## Result (musl binary, in-container)

| check | outcome |
|---|---|
| userland | `Alpine Linux v3.23` (musl) |
| musl `zic-rs` runs | `zic-rs 0.1.0`, `--version` exit 0; `ldd` → `/lib/ld-musl-x86_64.so.1` |
| PKGBUILD-style compile `--all-supported tzdata.zi` | exit **0** |
| **output file-set** | **598** |
| **`bundle_hash`** | **`453641ff2568d8b1…` — IDENTICAL to the glibc host + Arch builds** → **cross-*libc* + cross-environment output determinism** (musl-built binary in a musl userland produces byte-identical zoneinfo to the glibc builds) |
| invalid flag | exit **2** (consistent with the glibc CLI taxonomy) |
| missing source | exit **1** (consistent) |

**Status:** `matched_with_declared_divergence (RAN, musl)`.

## Honest scope / non-claims

- **Reference `zic` was not run in-container** (`python:3.12-alpine` ships no `zic`; `--network=none` ⇒ no
  `apk`). The reference *behaviour* comparison is the host/Arch work (`drop-in-gauntlet.1`/`.2.arch`); this
  recipe proves **musl runtime + package-build shape + cross-libc deterministic output**, kept as separate
  axes (runtime ABI ≠ package-recipe ≠ `zic`-behaviour ≠ output-tree-equivalence ≠ reader-equivalence).
- **The musl binary is host-cross-built, not Alpine-native-built** (`binary_origin = host musl-static`). An
  Alpine-native `abuild`/source build is a further step (needs the Alpine Rust toolchain).

```text
container recipe ran (musl)  != Alpine apk acceptance
                             != official APKBUILD / abuild-from-source
                             != universal /usr/sbin/zic replacement
musl binary host-cross-built != Alpine-native build reproducibility
```
