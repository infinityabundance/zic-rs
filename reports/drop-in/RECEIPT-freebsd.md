# Drop-in receipt — FreeBSD 14.3 (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for FreeBSD's *native* `/usr/sbin/zic`
> (`freebsd_14_x86_64`, T16.5b.1) but no receipt showing **zic-rs itself** runs there. zic-rs is a Linux
> ELF and cannot be copied to FreeBSD — but it is on crates.io and Rust targets FreeBSD, so FreeBSD's own
> `cargo` **builds it from source** and the resulting native binary runs the drop-in matrix.
> `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM, image
  `FreeBSD-14.3-CLOUDINIT-ufs.qcow2` behind a 20 GB overlay (`growfs` expands root). Transport: host server at
  `10.0.2.2:8000` (served `tzdata.zi` + uploaded evidence). Driver `qemu/dropin-freebsd-drive.py`; evidence
  `uploads/dropin-freebsd-evidence.tar.gz` (sha256 `2dbc4a35b778822b…`); serial `logs/freebsd_dropin_vm/serial.log`.

## Required fields

| field | value |
|---|---|
| **platform** | **FreeBSD 14.3-RELEASE-p14 amd64** (`FreeBSD 14.3-RELEASE-p14 GENERIC`) |
| **crate name / version** | `zic-rs` / `0.1.0` |
| **crate source identity** | crates.io `.crate` sha256 `2859db2c57a72086dfe919974c8cab3de7e70160e87a2b8ac61ab25f6bc5e556` |
| **Cargo.lock policy** | `cargo install zic-rs --version 0.1.0 --locked` |
| **toolchain** | `rustc 1.94.0 (built from a source tarball)` (FreeBSD `pkg install rust`) |
| **binary_origin** | **`ecosystem_source_build`** (FreeBSD's own cargo, from crates.io source) |
| **binary sha256** | **`fe821f73622af024651c3438475a5cd18dda510cdf66dd2413975af1a88e11e4`** — a native **FreeBSD ELF**, distinct from every Linux build |
| **runtime ABI** | `ELF 64-bit LSB pie executable, x86-64, version 1 (FreeBSD), dynamically linked` |
| **tzdata source** | `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (the matrix source) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — **byte-identical** to the host + every Linux VM + every Tier-3 source-build |
| **verdict** | **`matched_deterministic`** |

## What this proves

zic-rs builds cleanly from the published crate on **FreeBSD** (a non-Linux kernel + libc) and produces the
**byte-identical** 598-file zoneinfo tree as Linux glibc, Linux musl, and the host. The drop-in determinism
witness now crosses the **Linux ↔ FreeBSD** boundary: same source crate, different OS/ABI, identical output.
This is the FreeBSD **drop-in** counterpart to its existing native-`zic` **vendor-oracle** receipt.

## Non-claims

```text
FreeBSD source-build + run  != FreeBSD ports/pkg acceptance (no port submitted/merged)
                            != upstream binary release (zic-rs ships SOURCE on crates.io)
                            != default /usr/sbin/zic replacement on FreeBSD
                            != universal /usr/sbin/zic parity
the binary hash is FreeBSD-specific BY DESIGN; only the OUTPUT (bundle_hash) is invariant
```
