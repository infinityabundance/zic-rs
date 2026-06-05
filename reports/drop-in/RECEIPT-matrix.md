# Linux drop-in **QEMU VM** matrix — Tier-2 rows (cloud-init `runcmd` transport)

Real booted Linux VMs (not containers, not the host) running zic-rs and producing the matrix `bundle_hash`.
Boot: `qemu-system-x86_64 -M q35 -m 2560 -smp 2 -enable-kvm -cpu host -nographic -snapshot` (the `-snapshot`
keeps every base image **immutable** — all VM writes are ephemeral). Transport: a **NoCloud seed** (`CIDATA`,
`/tmp/seed-dropin.iso`) whose cloud-init `runcmd` mounts the **data ISO** (`qa-data-arch.iso`: glibc-PIE +
musl-static zic-rs + host `tzdata.zi` + recipe) and runs the recipe to `/dev/console` — **fully automated,
no serial-login timing**. The recipe prefers the host **glibc PIE** (genuine glibc story) and falls back to
**musl-static** only if the VM's glibc cannot load it (the ABI-honest case). Driver
`qemu/cloudinit-dropin-vm-drive.py`, recipe `qemu/arch-dropin-vm-recipe.sh`.

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64. Shared payload hashes: glibc PIE
  `b499a96f7928687583431046472503360897cf4e14f6bef108a634f439182613` · musl
  `a9617888163dc7767f80c2f6a1b2937ae5a39dab23b0919db3fbd054228bf12d` · `tzdata.zi`
  `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (`2026b-dirty`) · data ISO
  `f925c5c17fc99ef8757a72801b154eb8daa844b3aa8fad2478b4a06602d715a5` · seed ISO label `CIDATA`.

## Verified Tier-2 VM rows

| ecology (VM) | kernel | glibc | binary under test | loader verdict | files | `bundle_hash` | serial log |
|---|---|---|---|---|---|---|---|
| **Debian 13** (trixie, genericcloud) | `6.12.90+deb13.1-cloud` | **2.41** (Debian) | **glibc PIE** (runs) | loads natively (FHS) | **598** | `453641ff2568d8b1…` | `logs/debian_gc_dropin_vm/serial.log` |
| **Ubuntu 24.04.4 LTS** (Noble cloud) | `6.8.0-117-generic` | **2.39** (Ubuntu) | **glibc PIE** (runs) | loads natively | **598** | `453641ff2568d8b1…` | `logs/ubuntu_dropin_vm/serial.log` |
| **AlmaLinux 9.8** (GenericCloud, enterprise RPM) | `5.14.0-687.5.3.el9_8` | **2.34** | **musl-static** (fallback) | glibc-PIE **rejected** (`GLIBC_2.39 not found` — 2.34 < 2.39; ABI-honest, not a parity fail) | **598** | `453641ff2568d8b1…` | `logs/alma_dropin_vm/serial.log` |
| **Gentoo** (minimal live ISO, source distro) | `6.18.33-p1` | **2.42** (Gentoo) | **glibc PIE** (runs) | loads natively (FHS) | **598** | `453641ff2568d8b1…` | `logs/gentoo_dropin_vm/serial.log` |
| **openSUSE Leap 16.0** (Minimal-VM-Cloud, SUSE/RPM) | `6.12.0-160000.5-default` | **2.40** | **glibc PIE** (runs) | loads natively (FHS) | **598** | `453641ff2568d8b1…` | `logs/leap_dropin_vm/serial.log` (result-disk capture) |
| **Fedora 43** (Cloud, fast RPM / RHEL-upstream) | `6.17.1-300.fc43` | **2.42** | **glibc PIE** (runs) | loads natively (FHS) | **598** | `453641ff2568d8b1…` | `logs/fedora_dropin_vm/serial.log` |
| **NixOS 25.11** (Xantusia, minimal live ISO, content-addressed) | `7.0.10` | **2.40** | **musl-static** (fallback) | glibc-PIE **cannot execute: required file not found** — NixOS is **non-FHS**, there is no `/lib64/ld-linux-x86-64.so.2` (a *different* failure mode than Alma's version mismatch) | **598** | `453641ff2568d8b1…` | `logs/nixos_dropin_vm/serial.log` |

All four: `compile=0`. Each VM ships its own `zic` — glibc-`zic` via `libc-bin`/`glibc-common`/Nix-store
(Ubuntu/Alma/NixOS, T16.5b.9/.11/.12/.10) and tzcode-`zic` via `timezone-data` (Gentoo, T16.5b.13); a future
in-VM reference-behaviour comparison is available.

**The two distinct binary-load outcomes (both correctly handled by the recipe's glibc-PIE→musl fallback) —
the ABI/FHS-honesty distinction made concrete:**

```text
glibc-PIE RUNS natively          : Arch 2.43 · Gentoo 2.42 · Ubuntu 2.39        (glibc >= 2.39, FHS present)
glibc-PIE rejected, musl used    :
   - version too old             : AlmaLinux glibc 2.34  ("GLIBC_2.39 not found")
   - no FHS dynamic loader       : NixOS glibc 2.40       ("cannot execute: required file not found")
```

The musl-static binary runs in **every** environment (no interpreter, no version floor) and produces the
identical output — a concrete argument for shipping a static artifact for maximal portability. Neither
fallback is a parity failure; the *output* (`bundle_hash`) is identical regardless.

## Combined VM coverage (with the standalone Arch + Alpine receipts)

| VM | libc | kernel | binary_origin under test | `zic` lineage | tier |
|---|---|---|---|---|---|
| Arch 2026.06 (`RECEIPT-…-arch-qemu.md`) | glibc **2.43** | `7.0.10-arch1-1` | host glibc PIE (native) | tzcode (`tzdata`) | Tier 2 |
| Gentoo (here) | glibc **2.42** | `6.18.33-p1` | host glibc PIE (native) | tzcode (`timezone-data`) | Tier 2 |
| Debian 13 trixie (here) | glibc **2.41** | `6.12.90+deb13.1-cloud` | host glibc PIE (native) | glibc (`libc-bin`) | Tier 2 |
| openSUSE Leap 16.0 (here) | glibc **2.40** | `6.12.0-160000.5` | host glibc PIE (native) | tzcode (`timezone` RPM) | Tier 2 |
| NixOS 25.11 (here) | glibc **2.40** | `7.0.10` | musl-static (no-FHS-loader) | glibc (Nix store) | Tier 2 |
| Ubuntu 24.04 (here) | glibc **2.39** | `6.8.0-117-generic` | host glibc PIE (native) | glibc (`libc-bin`) | Tier 2 |
| AlmaLinux 9.8 (here) | glibc **2.34** | `5.14.0-…el9_8` | musl-static (PIE version-rejected) | glibc (`glibc-common`) | Tier 2 |
| Fedora 43 (here) | glibc **2.42** | `6.17.1-300.fc43` | host glibc PIE (native) | glibc (`glibc-common`) | Tier 2 |
| Alpine 3.23 (`RECEIPT-…-alpine-qemu.md`) | **musl** | `6.18-virt` | host-cross musl-static | tzcode (`tzdata-utils`) | Tier 2 |

**The witness:** the staged-tree `bundle_hash` is **byte-identical** (`453641ff2568d8b1…`) across **NINE
real booted VMs** spanning **glibc 2.34 · 2.39 · 2.40 ×2 · 2.41 · 2.42 ×2 · 2.43 · musl**, nine kernels
(`5.14`/`6.8`/`6.12`×2/`6.17`/`6.18-virt`/`6.18.33`/`7.0.10`×2), **both** `zic` lineages (glibc-`zic` +
tzcode-`zic`), and the **deb · rpm · apk · pacman · source · content-addressed · SUSE** package families. The
glibc-`zic` RPM family is **bracketed end-to-end**: Fedora 2.42 (fast/RHEL-upstream) ↔ AlmaLinux 2.34 (frozen
el9). Cross-libc + cross-kernel + cross-distro-family determinism, proven in real VMs rather than containers.

**Capture methods (per image, recorded honestly):** serial-console direct read (Arch · Gentoo · NixOS via
direct-kernel boot; Ubuntu · Debian · Alma via cloud-init `runcmd` → `/dev/console`); **result-disk capture**
(openSUSE Leap — its kernel has no `console=ttyS0`, so cloud-init `runcmd` runs `recipe-disk.sh` which `dd`s
the `RBEGIN…REND` block to a second virtio disk read back from the host after poweroff; driver
`qemu/diskcapture-dropin-vm-drive.py`). The result-disk method is the robust fallback for any serial-hostile
image, exactly the "capture-server-style" approach for VMs that fight serial/cloud-init.

## Status of the remaining curated ecologies (honest)

| ecology | image on disk | status |
|---|---|---|
| openSUSE **Leap 16** ✅ | `Leap-16…Cloud.qcow2` (8 GB overlay on real disk + NIC) | **DONE — Tier-2 VM** (table above). The earlier failures were the **no-NIC + tiny-disk** combo (first-boot disk-grow filled the 1.3 GB disk → self-powerdown). Fixed with an **8 GB overlay on real disk + a NIC** → boots fully (`Welcome to openSUSE Leap 16.0 … (ttyS0)`). Its kernel lacks `console=ttyS0`, so captured via the **result-disk** method. SUSE/RPM family is now **Tier-2 VM** (the SLES-BCI container, `RECEIPT-…-sles-bci.md`, remains the commercial-SLES Tier-1 row). |
| **Debian 13** ✅ | `debian-13-genericcloud-amd64.qcow2` (cloud-init; fetched + 339,017,728 B) | **DONE — Tier-2 VM** (table above): glibc 2.41, glibc-PIE runs. (The on-disk `debian-13-nocloud` is the interactive-firstboot variant; the genericcloud image is the cloud-init one.) |
| **Gentoo** ✅ | minimal live ISO | **DONE — Tier-2 VM** (table above): direct-kernel boot, glibc 2.42, glibc-PIE runs. |
| **NixOS 25.11** ✅ | minimal live ISO | **DONE — Tier-2 VM** (table above): direct-kernel boot, non-FHS → musl fallback. |
| **BSD** (FreeBSD/OpenBSD/NetBSD/DragonFly), **illumos** (OmniOS/OI/SmartOS) | qcow2/img/iso present | **drop-in-runs N/A by construction** — zic-rs is a **Linux ELF**; it cannot execute on a BSD/illumos kernel without a native build target. These remain **vendor-oracle receipts** (their native `zic`, T16.5b.1–7), not zic-rs-runs rows. A typed non-applicability, not a failure. |

## Non-claims (loud)

```text
Linux VM recipe ran    != official distro package acceptance
                       != native package build (binaries are host-built, not built in-VM)  -> Tier 3
                       != distro policy / universal /usr/sbin/zic replacement
glibc-PIE rejected (Alma 2.34) = GLIBC version mismatch (needs 2.39), ABI-honest, NOT a parity failure
data-ISO + cloud-init seed     != network/package-manager install path
BSD/illumos                    = zic-rs (Linux ELF) does not run there; vendor-oracle receipts stand, drop-in N/A
```

- Reference `zic` was **not** run in-VM (offline data-ISO transport); reference-*behaviour* parity is the
  host/container work (`RECEIPT-host.md` §A). These rows prove **zic-rs runs in a real booted Linux VM
  (kernel/glibc/loader identity recorded) + 598-file compile + cross-libc/cross-kernel deterministic
  `bundle_hash`**. The next rung (Tier 3) is **distro-native package builds from source inside the VMs**
  (deb/rpm/PKGBUILD/abuild) → distro-native `binary_origin`.
