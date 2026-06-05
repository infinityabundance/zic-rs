# Drop-in receipt — OmniOS r151058 / illumos (zic-rs built from crates.io, runs natively)

> Fills the gap where the lab has a **vendor-oracle** receipt for OmniOS's *native* base `/usr/sbin/zic`
> (`omnios_illumos_x86_64`, T16.5b.2) but no receipt showing **zic-rs itself** runs there. zic-rs is a Linux
> ELF; OmniOS's own `cargo` (the `ooce/developer/rust` package) **builds it from the crates.io source** and
> the resulting native illumos binary runs the drop-in matrix. `binary_origin = ecosystem_source_build`.
> **First illumos/SunOS drop-in** (the Solaris-family kernel — distinct ABI from Linux and the BSDs).

- **Date:** 2026-06-04 · **Host:** Linux 7.0.9-1-cachyos x86_64 · QEMU/KVM, image `omnios.qcow2`
  (from `omnios-r151058.cloud.qcow2`, sha256 `eb36b84b…`), boot **UEFI(OVMF)+q35** + `-cpu host` +
  `virtio-rng` (RDSEED masked → entropy stall without it) + PXE-ROM stripped (`romfile=`). Transport:
  host server `10.0.2.2:8000` (served `tzdata.zi` + uploaded `uploads/dropin-omnios-evidence.tar.gz`).
- **Rust path note:** OmniOS lays cargo down at **`/opt/ooce/rust/bin`** (the `ooce/developer/rust` ips
  package), *not* `/opt/ooce/bin` — so the build needs that dir on PATH.

## Required fields

| field | value |
|---|---|
| **platform** | **OmniOS Community Edition v11 r151058** (illumos, SunOS 5.11, i86pc/amd64; CPE `cpe:/o:omniosce:omnios:11:151058:0`) |
| **crate / version** | `zic-rs` / `0.1.0` (crates.io `.crate` sha256 `2859db2c…`) |
| **toolchain** | `rustc 1.95.0 (59807616e 2026-04-14) (OmniOS/151058)` · `cargo 1.95.0 (OmniOS/151058)` (`pkg install ooce/developer/rust`) |
| **binary_origin** | **`ecosystem_source_build`** (OmniOS cargo, `cargo install zic-rs --version 0.1.0 --locked`, from crates.io) |
| **binary_sha256** | `b8c7fb6bcc60562750daee196692c54f44efdecfc917d507d7fc1e299262283a` |
| **runtime ABI** | `ELF 64-bit LSB executable AMD64` (illumos native — distinct from Linux/BSD ELF) |
| **tzdata source** | `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (the matrix source, 2026b-dirty) |
| **compile exit / files** | `0` / **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to host + every Linux/BSD build |
| **verdict** | **`matched_deterministic`** |

## What this proves

zic-rs builds from the published crate on **illumos** (the Solaris-family kernel — a third ABI lineage after
Linux and the BSDs) using OmniOS's own rustc/cargo, and produces the **byte-identical** 598-file zoneinfo
tree. The crate source is the portable unit; output determinism is independent of kernel ABI, libc, and build
provenance (illumos rustc 1.95.0 here vs glibc/musl/BSD toolchains elsewhere → identical bytes).

## Non-claims

```text
OmniOS source-build + run != OmniOS/illumos pkg(5) acceptance != upstream binary release
                          != default /usr/sbin/zic replacement != universal /usr/sbin/zic parity
                          != OpenIndiana/SmartOS (same family, distinct builds — separate rows)
only the OUTPUT (bundle_hash) is invariant; the binary is illumos-specific by design
```
