# Tier-3 ecosystem source-build matrix — zic-rs built from the crates.io source crate

> The Rust-native Tier-3 proof: each ecosystem's **own** Rust/packaging tooling builds zic-rs **from the
> published crates.io source crate** (`zic-rs 0.1.0`, `.crate` sha256 `2859db2c…`), and every
> independently-built binary passes the drop-in matrix with the **identical** `bundle_hash`.
> `binary_origin = ecosystem_source_build` for all rows. Source `tzdata.zi` = `0078657f…` (the matrix source).

| ecosystem | toolchain | libc | build machinery | package artifact | built binary sha256 | files | `bundle_hash` |
|---|---|---|---|---|---|---|---|
| **Alpine 3.23** | rustc/cargo **1.91.1** (`apk rust-1.91.1-r2`) | **musl** | `cargo install --locked` | — (binary) | `0204bdc8…` | **598** | `453641ff…` |
| **Arch** (rolling) | rustc/cargo **1.96.0** (`rust 1:1.96.0-1`) | **glibc 2.43** | **`makepkg` + PKGBUILD** | **`zic-rs-0.1.0-1-x86_64.pkg.tar.zst`** (`91a0aa8d…`) | `ed358fc0…` | **598** | `453641ff…` |
| **Debian 13 trixie** | rustc/cargo **1.85.0** (`cargo 1.85.0+dfsg3-1`) | **glibc 2.41** | `cargo install --locked` | — (binary) | `e444105b…` | **598** | `453641ff…` |
| **AlmaLinux 9.8** | rustc/cargo **1.92.0** (`cargo-1.92.0-1.el9`) | **glibc 2.34** | **`rpmbuild` + .spec** | **`zic-rs-0.1.0-1.el9.x86_64.rpm`** (`2b5bdef5…`) | `1c9c82fb…` | **598** | `453641ff…` |

Per-row detail: `RECEIPT-2026-06-04-tier3-{alpine-source-build,arch-makepkg,rpm-rpmbuild}.md` + the Debian row
(`reports/drop-in/tier3/debian-cargo.log`, recipe `debian-cargo-recipe.sh`). All `compile_exit=0`. All built
`--locked` against the crate's published `Cargo.lock`. Network: **online for the build** (fetch crate + cargo
deps); the *runtime* drop-in needs no network.

## The witness — output determinism is independent of build provenance

**Five independently-produced binaries** now compile the same `tzdata.zi` to the **byte-identical** 598-file
tree (`bundle_hash 453641ff2568d8b1…`):

```text
binary                build provenance                              libc        rustc
b499a96f…  host-built (cachyos)                                      glibc 2.43  ~1.74-era host
0204bdc8…  Alpine `cargo install` from crates.io                    musl        1.91.1
ed358fc0…  Arch `makepkg`/PKGBUILD from crates.io  → .pkg.tar.zst   glibc 2.43  1.96.0
e444105b…  Debian `cargo` from crates.io                            glibc 2.41  1.85.0
1c9c82fb…  AlmaLinux `rpmbuild`/.spec from crates.io → .rpm         glibc 2.34  1.92.0
```

Different compilers (rustc 1.74→1.96), different libcs (glibc 2.34/2.41/2.43 + musl), different packaging
(`cargo install` · `makepkg` · `rpmbuild`) — **identical output**. The crate source is the portable unit;
the binary hash is build-specific, but the *compiled zoneinfo* is invariant.

Two rows are **real distro packaging** (Arch `.pkg.tar.zst` via `makepkg`+PKGBUILD; AlmaLinux `.rpm` via
`rpmbuild`+`.spec`, each `pacman -U` / `rpm -i` installed) and two are the idiomatic `cargo install` from the
crate. Combined with Tier-2 (8 real VMs run the binary), the gauntlet now proves both **runtime** and
**source-build** determinism across the major Linux ecosystems.

## Non-claims (loud — apply to every row)

```text
ecosystem source-build recipe succeeded
  != upstream binary release            (zic-rs ships SOURCE on crates.io; ecosystems build it)
  != official distro package acceptance (no AUR / Fedora / EPEL / Debian / apk package submitted or merged)
  != default /usr/sbin/zic replacement
  != universal /usr/sbin/zic parity
online build-provenance != the no-network runtime drop-in
real packaging (makepkg/rpmbuild) != packaging the distro's own vendored-dep model (deps came from crates.io)
binary hashes differ across builds BY DESIGN — only the OUTPUT (bundle_hash) is invariant
```

Reference `zic` behaviour parity remains the host/container work (`RECEIPT-host.md` §A). These are
**source-build provenance** proofs: each ecosystem can build zic-rs from the crate source and the result is
deterministic.
