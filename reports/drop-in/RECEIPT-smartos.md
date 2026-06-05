# Drop-in receipt — SmartOS (illumos) global zone (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for SmartOS's *native* base `/usr/sbin/zic`
> (`smartos_20260528_x86_64`, T16.5b.6) but no receipt showing **zic-rs itself** runs there. zic-rs is a
> Linux ELF; SmartOS's own `cargo` **builds it from the crates.io source** and the resulting native illumos
> binary runs the drop-in matrix. `binary_origin = ecosystem_source_build`. **Completes the illumos triad**
> (OmniOS · OpenIndiana · SmartOS) — the third and final illumos drop-in.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM. SmartOS is a **stateless RAM
  global zone** (no persistent `/usr`), so a build env required completing the **SmartOS setup wizard** to
  create a persistent **`zones` zpool** on a 30 GB virtio disk. The whole wizard (network DHCP · DNS · NTP ·
  zpool layout · **"install pkgsrc into the global zone? yes"** · root password · hostname) was driven
  **headlessly via the QEMU monitor** (`qmon.py` `screendump`/`sendkey` over the VGA console — the same
  method as NetBSD/DragonFly/OpenIndiana). After reboot the configured GZ has **pkgsrc/`pkgin`** (in
  `/opt/tools`/`/opt/local`) and a writable `/var` on the zpool.
- **Toolchain note:** `pkgin -y install rust` (the rust package downloads a 31-package dependency set
  including **llvm** — large + slow over NAT; pkgin caches downloads, so an interrupted run resumes) + a
  pkgsrc `gcc` for linking. Build done on the persistent `/var/tmp` (zones zpool).

## Required fields

| field | value |
|---|---|
| **platform** | **SmartOS** (illumos, SunOS 5.11, PI `joyent_20260528T000227Z`, i86pc/amd64) |
| **crate / version** | `zic-rs` / `0.1.0` (crates.io `.crate` sha256 `2859db2c…`) |
| **toolchain** | `rustc 1.95.0 (59807616e 2026-04-14)` · `cargo 1.95.0` (pkgsrc `rust`) + pkgsrc `gcc` linker |
| **binary_origin** | **`ecosystem_source_build`** (SmartOS cargo, `cargo install zic-rs --version 0.1.0 --locked`, from crates.io) |
| **runtime ABI** | `ELF 64-bit LSB executable AMD64` (illumos native) |
| **tzdata source** | the matrix 2026b `tzdata.zi` (served from the host) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to host + every Linux/BSD/illumos build |
| **verdict** | **`matched_deterministic`** |

(`binary_sha256`/`tzdata_sha256` not captured — the GZ lacks `sha256sum`/`shasum` on PATH; the authoritative
witness is the `bundle_hash` from zic-rs's own `size-report`.)

## What this proves

zic-rs builds from the published crate on **SmartOS** (illumos) using pkgsrc rust/cargo and produces the
**byte-identical** 598-file zoneinfo tree. **The illumos family is now done three ways** — OmniOS (cloud
qcow2), OpenIndiana (disk install), SmartOS (setup-wizard zpool + pkgsrc-in-GZ) — three distinct distros,
three distinct binaries/toolchain paths, identical output. The crate source is the portable unit; output
determinism is independent of OS/libc/toolchain/build provenance. *Method note:* the stateless RAM-GZ
"no persistent /usr" wall was overcome the SmartOS-native way — its own setup wizard installs pkgsrc into
the GZ on the zones zpool.

## Non-claims

```text
SmartOS source-build + run != SmartOS pkgsrc/image acceptance != upstream binary release
                           != default /usr/sbin/zic replacement != universal /usr/sbin/zic parity
                           != a native zone build (this is the global zone with pkgsrc)
only the OUTPUT (bundle_hash) is invariant; the binary is illumos-specific by design
```
