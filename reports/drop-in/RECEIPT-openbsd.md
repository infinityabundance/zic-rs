# Drop-in receipt — OpenBSD 7.9 (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for OpenBSD's *native* base `/usr/sbin/zic`
> (`openbsd_79_x86_64`, T16.5b.3) but no receipt showing **zic-rs itself** runs there. zic-rs is a Linux ELF;
> OpenBSD's own `cargo` **builds it from the crates.io source** and the resulting native binary runs the
> drop-in matrix. `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM, image `openbsd-7.9-amd64.qcow2`,
  `-nic user` (em0), networking `ifconfig em0 inet autoconf`. Transport: host server `10.0.2.2:8000`
  (served `tzdata.zi` + uploaded evidence `uploads/dropin-openbsd-evidence.tar.gz`).

## Required fields

| field | value |
|---|---|
| **platform** | **OpenBSD 7.9 amd64** |
| **crate / version** | `zic-rs` / `0.1.0` (crates.io `.crate` sha256 `2859db2c…`) |
| **toolchain** | `rustc 1.94.1` (OpenBSD `pkg_add rust`) |
| **binary_origin** | **`ecosystem_source_build`** (OpenBSD cargo, `cargo install zic-rs --locked`, from crates.io) |
| **runtime ABI** | `ELF 64-bit LSB shared object, x86-64` (OpenBSD native) |
| **tzdata source** | `0078657f…` (the matrix source) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to host + every Linux/BSD build |
| **verdict** | **`matched_deterministic`** |

(`binary_sha256` was not captured: OpenBSD base ships `sha256`, not `sha256sum`/`shasum` — cosmetic; the
authoritative witness is the bundle_hash emitted by zic-rs's own `size-report`.)

## What this proves

zic-rs builds from the published crate on **OpenBSD** (a conservative non-Linux userland) and produces the
**byte-identical** 598-file zoneinfo tree as Linux and FreeBSD. Second non-Linux platform after FreeBSD;
the source crate is the portable unit.

## Non-claims

```text
OpenBSD source-build + run != OpenBSD ports/pkg acceptance != upstream binary release
                           != default /usr/sbin/zic replacement != universal /usr/sbin/zic parity
only the OUTPUT (bundle_hash) is invariant; the binary is OpenBSD-specific by design
```
