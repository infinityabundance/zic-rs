# Drop-in receipt — DragonFly 6.4.2-RELEASE (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for DragonFly's *native* base `/usr/sbin/zic`
> (`dragonfly_642_x86_64`, T16.5b.5 — the oldest `zic` fork, which itself can't even ingest modern zishrink
> `tzdata.zi`) but no receipt showing **zic-rs itself** runs there. zic-rs is a Linux ELF; DragonFly's own
> `cargo` **builds it from the crates.io source** and the resulting native binary runs the drop-in matrix —
> and unlike DragonFly's old native `zic`, zic-rs compiles the modern 2026b `tzdata.zi` cleanly.
> `binary_origin = ecosystem_source_build`.

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM, DragonFly 6.4.2 live image
  (`dfly-x86_64-6.4.2_REL.img`) via a writable qcow2 overlay. Console driven over the VGA framebuffer via
  QEMU monitor `screendump`/`sendkey` (the live image emits nothing to serial). Network via `dhclient em0`;
  transport: host server `10.0.2.2:8000`.
- **Space note:** the live "Amnesiac" root has <1 GB free and `/usr/local` lives on it, so a 7 GB tmpfs was
  nullfs-mounted over `/usr/local` (preserving `pkg`) and a 5 GB tmpfs used for the build (`CARGO_HOME`/
  `TMPDIR`) — all RAM-backed (tmpfs is sparse). DragonFly's `pkg` repo + DNS work directly (no relay needed),
  and base ships `gcc`/`cc` so `rustc` links natively.

## Required fields

| field | value |
|---|---|
| **platform** | **DragonFly 6.4.2-RELEASE x86_64** (`cpe:/o:dragonflybsd:dragonfly:6.4`) |
| **crate / version** | `zic-rs` / `0.1.0` (crates.io `.crate` sha256 `2859db2c…`) |
| **toolchain** | `rustc 1.85.1 (4eb161250 2025-03-15)` · `cargo 1.85.1` (DragonFly dports `rust-1.85.1`, Avalon repo) |
| **binary_origin** | **`ecosystem_source_build`** (DragonFly cargo, `cargo install zic-rs --version 0.1.0 --locked`, from crates.io) |
| **runtime ABI** | `ELF 64-bit LSB shared object, x86-64 (SYSV), dynamically linked` (DragonFly native) |
| **tzdata source** | the matrix 2026b `tzdata.zi` (served from the host) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to host + every Linux/BSD/illumos build |
| **verdict** | **`matched_deterministic`** |

(`binary_sha256`/`tzdata_sha256` not captured — DragonFly base lacks `sha256sum`/`shasum`; the authoritative
witness is the `bundle_hash` from zic-rs's own `size-report`.)

## What this proves

zic-rs builds from the published crate on **DragonFly BSD** and produces the **byte-identical** 598-file
zoneinfo tree. This **completes the BSD family** (FreeBSD · OpenBSD · NetBSD · DragonFly) as native
source-builds. Pointed contrast: DragonFly's *own* `zic` is the oldest fork in the vendor lab and **cannot**
ingest the modern zishrink 2026b source — yet zic-rs, built right there from the crate, compiles it to the
same bytes as everywhere else. The crate source is the portable unit; output determinism is independent of
OS/libc/toolchain/build provenance.

## Non-claims

```text
DragonFly source-build + run != DragonFly dports/pkg acceptance != upstream binary release
                             != default /usr/sbin/zic replacement != universal /usr/sbin/zic parity
                             != fixing DragonFly's old native zic (separate tool)
only the OUTPUT (bundle_hash) is invariant; the binary is DragonFly-specific by design
```
