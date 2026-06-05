# Tier-3 ecosystem source-build — RPM family (rpmbuild / .spec, from the crates.io source crate)

> The RPM family's **own packaging machinery** (`rpmbuild` + a `.spec`) builds zic-rs **from the published
> crates.io source crate** into a real `.rpm`, `rpm -i` installs it, and the packaged binary passes the
> drop-in matrix. Real distro packaging (the rpm counterpart to the Arch makepkg rung).
> `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · `podman --network=host almalinux:9`.
  Recipe `reports/drop-in/rpm-rpmbuild-recipe.sh` (inline `.spec`); log `reports/drop-in/tier3/rpm-rpmbuild.log`.

## Required receipt fields

| field | value |
|---|---|
| **crate name / version** | `zic-rs` / `0.1.0` |
| **crate source identity** | crates.io `.crate` sha256 **`2859db2c…`** — downloaded to `~/rpmbuild/SOURCES`, verified in-recipe |
| **Cargo.lock policy** | **`cargo build --release --locked`** (the crate's published lockfile) |
| **ecosystem** | **AlmaLinux 9.8** (enterprise RPM), package family **rpm/dnf**, builder **`rpmbuild` 4.16.1.3** |
| **toolchain identity** | `rustc 1.92.0 (Red Hat 1.92.0-1.el9)` · `cargo 1.92.0` · `rpm -q cargo` = **`cargo-1.92.0-1.el9`** · glibc **2.34** |
| **build recipe** | `.spec` (`Source0=` the crates.io `.crate`; `%global debug_package %{nil}`) → `rpmbuild -bb` → `.rpm` → `rpm -i` |
| **network mode** | **online** (rpmbuild fetches the crate / cargo fetches the locked deps; the *runtime* drop-in needs none) |
| **binary_origin** | **`ecosystem_source_build`** (RPM `rpmbuild`/spec from crates.io source) |
| **package artifact** | **`zic-rs-0.1.0-1.el9.x86_64.rpm`** (747,449 B), sha256 **`2b5bdef594c8d226a8b8ca94bf42b8470e8b5dc5b0710dff79e27f8514662fd4`** |
| **binary hash** | **`1c9c82fb17a79e95f763d325a98265da399b5bd4aa05b47404d4ecb7a7f3fe0e`** — distinct (AlmaLinux glibc 2.34) |
| **target / runtime ABI** | `x86_64` · **glibc 2.34** (dynamically linked) |
| **drop-in result** | `compile_exit=0` · **files = 598** · tzdata `0078657f…` |
| **bundle_hash** | **`453641ff2568d8b1…`** — byte-identical to the host + 8 VMs + Alpine/Arch/Debian source-builds |

## Non-claims (loud)

```text
rpmbuild/.spec source-build succeeded
  != official RPM/distro package acceptance   (no Fedora/RHEL/EPEL package submitted or merged)
  != upstream binary release                  (zic-rs ships SOURCE on crates.io)
  != default /usr/sbin/zic replacement
  != universal /usr/sbin/zic parity
online-build provenance != the no-network runtime drop-in
the .spec bundles deps via cargo (crates.io), NOT the Fedora "rust2rpm / vendored-as-rpm" dependency model
binary hash differs from other builds BY DESIGN — only the OUTPUT (bundle_hash) is invariant
```

- This is a faithful rpm package-build shape (sha256-pinned crate source, `rpmbuild`, `.rpm`, `rpm -i`),
  **not** a merged Fedora/EPEL package nor the `rust2rpm` per-dependency-as-rpm model — distro *acceptance*
  is a separate, unclaimed rung.
