# Tier-3 ecosystem source-build — Alpine (from the crates.io source crate)

> **The Rust-native Tier-3 model:** the portable distribution unit is the **crates.io source crate**, not a
> shipped binary. This receipt proves Alpine's **own** Rust toolchain can **build zic-rs from the published
> crate source** and that the resulting distro-native binary passes the drop-in matrix.
> `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · `podman --network=host alpine:latest`
  (a source-build must reach the network to fetch the crate + its crates.io dependencies). Recipe:
  `reports/drop-in/alpine-abuild-recipe.sh`; log `reports/drop-in/tier3/alpine-abuild.log`.

## Required receipt fields

| field | value |
|---|---|
| **crate name** | `zic-rs` |
| **crate version** | `0.1.0` |
| **crate source identity** | crates.io `.crate` sha256 **`2859db2c57a72086dfe919974c8cab3de7e70160e87a2b8ac61ab25f6bc5e556`** (238,368 B) — the published source, not a local tree |
| **Cargo.lock policy** | **`--locked`** (builds against the lockfile published *inside* the crate → reproducible dependency set) |
| **ecosystem** | **Alpine Linux v3.23** (musl), package family **apk** |
| **toolchain identity** | `rustc 1.91.1 (Alpine Linux Rust 1.91.1-r2)` · `cargo 1.91.1` · apk **`rust-1.91.1-r2`** / **`cargo-1.91.1-r2`** — Alpine's *own* toolchain, not a vendored one |
| **build recipe** | `apk add rust cargo` → `cargo install zic-rs --version 0.1.0 --locked --root /opt/zic` |
| **network mode** | **online** (required: `apk` + `cargo` fetch the crate and its crates.io deps; the *runtime* drop-in needs no network) |
| **binary_origin** | **`ecosystem_source_build`** (`built_from_crates_io_source_by_ecosystem_recipe`) |
| **binary hash** | **`0204bdc8ef853dbcdaabb0737324457b520fb5a02319cded265abb5d23eddf21`** — a **distinct** binary from the host-cross-built musl one (`a9617888…`): different toolchain, different build, **identical output** |
| **target triple / runtime ABI** | `x86_64-alpine-linux-musl` · **musl** (`/lib/ld-musl-x86_64.so.1`) |
| **drop-in result** | `compile_exit=0` · **files = 598** |
| **tzdata source** | host `tzdata.zi` sha256 `0078657f…` (the matrix source) |
| **bundle_hash** | **`453641ff2568d8b1…`** — byte-identical to the host + all 8 Tier-2 VMs + the containers |

## What this proves (and the new axis)

The drop-in matrix `bundle_hash` is now identical not only across **runtime** environments (host · containers
· 8 real VMs · glibc 2.34–2.43 · musl) but across **build provenance**: a binary produced by **Alpine's own
`cargo` from the crates.io source** yields the same 598-file output as the host-built and host-cross-built
binaries. **The output is deterministic regardless of who compiled the crate or how** — the strongest form of
"the source crate is the portable unit."

```text
Tier-2 (runtime):   zic-rs RUNS in the ecology            ✅ (8 VMs)
Tier-3 (source):    the ecology BUILDS zic-rs from source ◀ this receipt (Alpine)
```

## Non-claims (loud)

```text
crates.io source-build recipe succeeded
  != upstream binary release            (zic-rs ships SOURCE on crates.io; distros build it)
  != official distro package acceptance (no apk aport / APKBUILD merged)
  != default /usr/sbin/zic replacement
  != universal /usr/sbin/zic parity
online-build provenance != the no-network runtime drop-in (the build fetches; the runtime does not)
ecosystem-built binary hash differs from the host binary BY DESIGN — only the OUTPUT (bundle_hash) is invariant
```

- The build used Alpine's stock `cargo install` (the idiomatic "build from crate source"), not a merged
  **APKBUILD/aport** — that (an `abuild`-from-`APKBUILD` aport) is the next sub-rung, and distro *acceptance*
  of such an aport is a separate, unclaimed step. Reference `zic` behaviour parity remains the host/container
  work; this receipt is a **source-build provenance** proof.
