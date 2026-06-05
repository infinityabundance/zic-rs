# Tier-3 ecosystem source-build — Arch (PKGBUILD / makepkg, from the crates.io source crate)

> Arch's **own packaging machinery** (`makepkg` + a `PKGBUILD`) builds zic-rs **from the published crates.io
> source crate** into a real `.pkg.tar.zst`, `pacman -U` installs it, and the packaged binary passes the
> drop-in matrix. Stronger than `cargo install`: this is the distro's *package-build* shape, sha256-pinned to
> the crate source. `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · `podman --network=host archlinux:latest`
  (a source-build must fetch the crate + cargo deps). Recipe `reports/drop-in/arch-makepkg-recipe.sh`
  (inline PKGBUILD); log `reports/drop-in/tier3/arch-makepkg.log`.

## Required receipt fields

| field | value |
|---|---|
| **crate name / version** | `zic-rs` / `0.1.0` |
| **crate source identity** | crates.io `.crate` sha256 **`2859db2c57a72086dfe919974c8cab3de7e70160e87a2b8ac61ab25f6bc5e556`** — **pinned in the PKGBUILD `sha256sums`** (makepkg verifies it before building) |
| **Cargo.lock policy** | **`cargo build --release --locked`** (the crate's published lockfile) |
| **ecosystem** | **Arch Linux** (rolling), package family **pacman**, builder **`makepkg (pacman) 7.1.0`** |
| **toolchain identity** | `rustc 1.96.0 (Arch Linux rust 1:1.96.0-1)` · `cargo 1.96.0` · `pacman -Q rust` = **`rust 1:1.96.0-1`** · glibc **2.43** |
| **build recipe** | `PKGBUILD` (`source=` the crates.io `.crate`, sha256-pinned) → `makepkg -f` (as an unprivileged `builder` user) → `.pkg.tar.zst` → `pacman -U` |
| **network mode** | **online** (makepkg fetches the crate + cargo fetches the locked deps; the *runtime* drop-in needs none) |
| **binary_origin** | **`ecosystem_source_build`** (Arch `makepkg`/PKGBUILD from crates.io source) |
| **package artifact** | **`zic-rs-0.1.0-1-x86_64.pkg.tar.zst`** (769,976 B), sha256 **`91a0aa8d688005d36f0158d30c6fabe6d15ccc0d95a4a2e2986a70b8ae81fef2`** |
| **binary hash** | **`ed358fc0471a1d5af416d21a16af0c03e7b7f39084acd20024b36f35aac6740f`** — distinct again (Arch glibc PIE; ≠ host `b499a96f…`, ≠ Alpine-musl `0204bdc8…`) |
| **target / runtime ABI** | `x86_64` · **glibc 2.43** (dynamically linked) |
| **drop-in result** | `compile_exit=0` · **files = 598** · tzdata `0078657f…` |
| **bundle_hash** | **`453641ff2568d8b1…`** — byte-identical to the host + 8 VMs + Alpine source-build |

## Source-build provenance now spans two ecosystems × two libcs

```text
ecosystem source-build (Tier-3):
  Alpine   apk rust + cargo install   musl   binary 0204bdc8…  → 598 / 453641ff…
  Arch     makepkg + PKGBUILD          glibc  binary ed358fc0…  → 598 / 453641ff…   ◀ this receipt
```

**Three independently-built binaries** (host-built `b499a96f`, Alpine-musl `0204bdc8`, Arch-glibc `ed358fc0`)
— different toolchains (rustc 1.74-era host · 1.91.1 Alpine · 1.96.0 Arch), different libcs, different
packaging — all emit the **identical** 598-file tree (`bundle_hash 453641ff…`). The crate source is the
portable unit; output determinism is independent of build provenance.

## Non-claims (loud)

```text
makepkg/PKGBUILD source-build succeeded
  != official Arch package acceptance     (no AUR/community pkgbase submitted or merged)
  != upstream binary release              (zic-rs ships SOURCE on crates.io)
  != default /usr/sbin/zic replacement
  != universal /usr/sbin/zic parity
online-build provenance != the no-network runtime drop-in
binary hash differs from other builds BY DESIGN — only the OUTPUT (bundle_hash) is invariant
```

- The PKGBUILD is a faithful Arch package-build shape (sha256-pinned crate source, `makepkg`, `.pkg.tar.zst`,
  `pacman -U`), **not** a submitted/merged AUR or `[extra]` package — distro *acceptance* is a separate,
  unclaimed rung. Reference `zic` behaviour parity remains the host/container work.
