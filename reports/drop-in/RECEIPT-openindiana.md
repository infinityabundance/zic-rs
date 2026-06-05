# Drop-in receipt — OpenIndiana Hipster 2026.04 / illumos (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for OpenIndiana's *native* base `/usr/sbin/zic`
> (`openindiana_hipster_20260430_x86_64`, T16.5b.7) but no receipt showing **zic-rs itself** runs there.
> zic-rs is a Linux ELF; OpenIndiana's own `cargo` **builds it from the crates.io source** and the resulting
> native illumos binary runs the drop-in matrix. `binary_origin = ecosystem_source_build`. **Completes the
> illumos triad** (OmniOS · SmartOS-family · OpenIndiana) on the compiler side — third illumos drop-in.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM. OpenIndiana ships **only a text
  live ISO** whose `/usr` is a **read-only lofi** image (so the live ISO cannot install packages — see
  STATUS), so this required a **full disk install**: the text installer (`OI-hipster-text-20260430.iso`) was
  driven **headlessly via the QEMU monitor** (`qmon.py` `screendump`/`sendkey` over the VGA console — the
  same method that cracked NetBSD/DragonFly) into a fresh 24 GB virtio disk (whole-disk EFI/ZFS rpool, root
  password set, DHCP), then the **installed** system was booted (writable `/usr`, working IPS).
- **Toolchain note:** `pkg install developer/lang/rustc` (the package is `developer/lang/rustc`, not
  `developer/rust`) + **`developer/gcc-14`** — rustc 1.95.0 links via `/usr/gcc/14/bin/gcc`, which rustc's
  own package does **not** pull, so gcc-14 must be installed explicitly (the first build failed exactly there
  with `linker /usr/gcc/14/bin/gcc not found`, fixed by adding gcc-14). Build done on the ZFS `/var/tmp`.

## Required fields

| field | value |
|---|---|
| **platform** | **OpenIndiana Hipster 2026.04** (illumos, SunOS 5.11, illumos-4648b9b8c3, i86pc/amd64) |
| **crate / version** | `zic-rs` / `0.1.0` (crates.io `.crate` sha256 `2859db2c…`) |
| **toolchain** | `rustc 1.95.0 (59807616e 2026-04-14)` · `cargo 1.95.0` (`developer/lang/rustc`) + `developer/gcc-14` linker |
| **binary_origin** | **`ecosystem_source_build`** (OpenIndiana cargo, `cargo install zic-rs --version 0.1.0 --locked`, from crates.io) |
| **binary_sha256** | `1db96a82848ce43a306c4845fe72661013b6dba16851c7b489acb8a80810093b` |
| **runtime ABI** | `ELF 64-bit LSB executable AMD64` (illumos native) |
| **tzdata source** | `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (the matrix 2026b source) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to host + every Linux/BSD/illumos build |
| **verdict** | **`matched_deterministic`** |

## What this proves

zic-rs builds from the published crate on **OpenIndiana** (illumos) using its own rustc/cargo and produces
the **byte-identical** 598-file zoneinfo tree. With **OmniOS**, the illumos family now has two independent
native source-builds (distinct binaries: `1db96a82…` here vs OmniOS `b8c7fb6b…`), converging on identical
output. The crate source is the portable unit; output determinism is independent of OS/libc/toolchain/build
provenance. *Method note:* the live-ISO read-only-`/usr` wall (recorded in STATUS) was overcome the only way
it can be — a real disk install — driven entirely headless via the QEMU monitor.

## Non-claims

```text
OpenIndiana source-build + run != OpenIndiana IPS/pkg acceptance != upstream binary release
                               != default /usr/sbin/zic replacement != universal /usr/sbin/zic parity
                               != the live ISO building it (it can't — read-only lofi /usr; a disk install was required)
only the OUTPUT (bundle_hash) is invariant; the binary is illumos-specific by design
```
