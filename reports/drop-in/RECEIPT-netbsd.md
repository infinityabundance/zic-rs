# Drop-in receipt — NetBSD 10.1 amd64 (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for NetBSD's *native* base `/usr/sbin/zic`
> (`netbsd_101_x86_64`, T16.5b.4) but no receipt showing **zic-rs itself** runs there. zic-rs is a Linux ELF;
> NetBSD's own `cargo` **builds it from the crates.io source** and the resulting native binary runs the
> drop-in matrix. `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM, NetBSD 10.1 live image
  (`NetBSD-10.1-amd64-live.img`) via a writable qcow2 overlay + a 16 GB build disk (`gpt` + wedge `dk0` →
  `/b`, since the live root has only ~360 MB free). Console driven over the VGA framebuffer via QEMU
  monitor `screendump`/`sendkey` (the NetBSD live image emits nothing to serial).
- **Rust install note:** `cdn.NetBSD.org` 302-redirects both `amd64`→`x86_64` **and** `10.1`→`10.0_2026Q1`,
  and NetBSD's base `ftp(1)`/`pkg_add` do **not** follow redirects (they hang at 0%). Rust was installed via
  `pkg_add` pointed at a **host-side relay** (`10.0.2.2:8001` → the no-redirect `x86_64/10.0_2026Q1/All`
  path), so the full closure (rust + curl + libunwind + **llvm** + python313 + …) was fetched through the
  host at LAN speed. The toolchain and `/usr/pkg` live on the 16 GB build disk; NetBSD base ships `gcc`, so
  `rustc` links natively. The build itself (`cargo install zic-rs`) pulled from crates.io directly.

## Required fields

| field | value |
|---|---|
| **platform** | **NetBSD 10.1 (GENERIC) amd64** |
| **crate / version** | `zic-rs` / `0.1.0` (crates.io `.crate` sha256 `2859db2c…`) |
| **toolchain** | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` · `cargo 1.91.1` (pkgsrc `rust-1.91.1nb1`, built from source) |
| **binary_origin** | **`ecosystem_source_build`** (NetBSD cargo, `cargo install zic-rs --version 0.1.0 --locked`, from crates.io) |
| **runtime ABI** | `ELF 64-bit LSB pie executable, x86-64 (SYSV), dynamically linked` (NetBSD native) |
| **tzdata source** | the matrix 2026b `tzdata.zi` (served from the host) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to host + every Linux/BSD/illumos build |
| **verdict** | **`matched_deterministic`** |

(`binary_sha256`/`tzdata_sha256` not captured: NetBSD base has no `sha256sum`/`shasum`; the authoritative
witness is the `bundle_hash` from zic-rs's own `size-report`.)

## What this proves

zic-rs builds from the published crate on **NetBSD** using its own rustc/cargo (built-from-source pkgsrc
toolchain) and produces the **byte-identical** 598-file zoneinfo tree. Fourth non-Linux platform; with
FreeBSD + OpenBSD + OmniOS, the BSD and illumos families are all covered by native source-builds — the crate
source is the portable unit, output determinism independent of OS/libc/toolchain/build provenance.

## Non-claims

```text
NetBSD source-build + run != NetBSD pkgsrc/pkg_add acceptance != upstream binary release
                          != default /usr/sbin/zic replacement != universal /usr/sbin/zic parity
the host relay only worked around a CDN redirect; it changed nothing about the build itself
only the OUTPUT (bundle_hash) is invariant; the binary is NetBSD-specific by design
```
