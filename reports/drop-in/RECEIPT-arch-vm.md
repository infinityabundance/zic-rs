# Arch **QEMU VM** drop-in recipe receipt — T23.drop-in-gauntlet.arch-qemu (canonical Arch row)

> **VM-canonical doctrine:** a real booted OS image is the preferred drop-in witness; the container row is
> retained as lower-tier evidence. This **supersedes** the Arch *container* row
> (`RECEIPT-arch-container.md`, Tier 1) as the **canonical Arch replacement-readiness row
> (Tier 2 — VM-recipe-ran)**. The host glibc PIE runs on the VM's **own** (rolling, newest) glibc — the
> genuine glibc-in-Arch story, not a binary mounted into a shared-kernel container.

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64 · `qemu-system-x86_64` `-M q35 -m 2560
  -smp 2 -enable-kvm -cpu host`, **direct-kernel serial boot** (archiso `vmlinuz-linux`/`initramfs-linux.img`
  extracted from the ISO; `console=ttyS0` + `archisolabel`) → reliable headless serial. A Python
  serial-console driver logs in `root`, mounts the data ISO, runs the recipe, captures `RBEGIN…REND`.

## Required receipt fields

| field | value |
|---|---|
| **VM image identity** | `archlinux-2026.06.01-x86_64.iso`, sha256 **`ec7a9c89aed7a59a76266ccf723c5e88480e47d7088c4482436f882fa37c3989`** — **VERIFIED** against the published `sha256sums.txt`; volume label **`ARCH_202606`** |
| **kernel version** | **`7.0.10-arch1-1`** (the VM's own Arch kernel — distinct from the cachyos host `7.0.9-1-cachyos`) |
| **glibc version / loader path** | **glibc 2.43** (`ldd (GNU libc) 2.43`) · loader **`/lib64/ld-linux-x86-64.so.2`** — newest glibc in the entire gauntlet |
| **binary_origin** | **host-built glibc PIE** (`target/release/zic-rs`) — *the binary under test* (`under_test=/tmp/zg`); **musl-static** carried as a cross-check |
| **transport mode** | **data ISO** (`qa-data-arch.iso`, label `QADATA`, sha256 `f925c5c17fc99ef8757a72801b154eb8daa844b3aa8fad2478b4a06602d715a5`) mounted in-VM (`/dev/sr1` → `/mnt`) |
| **network mode** | **none** (no `-netdev`; data-ISO transport — nothing fetched in-VM) |
| **recipe hash** | `arch-dropin-vm-recipe.sh` sha256 `c216d1fb6ba42e3e7ef87d32773462bbe8fa4fa2ea9262c186ef6f68f368d974` |
| **zic-rs binary hash** | glibc PIE `b499a96f7928687583431046472503360897cf4e14f6bef108a634f439182613` · musl `a9617888163dc7767f80c2f6a1b2937ae5a39dab23b0919db3fbd054228bf12d` |
| **tzdata.zi hash** | `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (`2026b-dirty`; same source as the whole matrix) |
| **file count** | **598** |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — byte-identical to the cachyos-glibc host, Arch-glibc container, Alpine-musl container, Alpine-musl QEMU VM, and SLES-15-SP7 BCI |
| **exit status** | glibc-PIE `--version` = 0 · musl `--version` = 0 · **compile = 0** |
| **serial log path** | `../zic-rs-vendor-oracle-lab/logs/arch_dropin_vm/serial.log` (driver `qemu/arch-dropin-vm-drive.py`, recipe `qemu/arch-dropin-vm-recipe.sh`) |

## Result — captured from the booted VM serial console

```text
osrel="Arch Linux"            kernel=7.0.10-arch1-1
glibc=ldd (GNU libc) 2.43     loader=/lib64/ld-linux-x86-64.so.2
has_zic=/usr/bin/zic          (archiso ships the Arch `tzdata`-pkg tzcode zic — T16.5b.17 lineage)
glibc-PIE --version: zic-rs 0.1.0   g_ver_exit=0     <- the host glibc PIE RUNS on the VM's glibc 2.43
musl --version:      zic-rs 0.1.0   m_ver_exit=0
under_test=/tmp/zg (glibc PIE)
compile=0   files=598
bundle_hash: 453641ff2568d8b1…   (IDENTICAL to the entire matrix)
```

**The witness:** zic-rs runs **as a native glibc binary inside a real booted Arch Linux VM** (own kernel,
own glibc 2.43, own loader), and its staged-tree `bundle_hash` is byte-identical to every other environment.
Cross-environment determinism now spans **host · container · two real VMs**, across **glibc + musl**, across
**three kernels** (cachyos host · Alpine `6.18-virt` · Arch `7.0.10-arch1`), across **two libc loaders**.

## Drop-in evidence ladder (where this row sits)

```text
Tier 0  host baseline                         — RECEIPT-host.md §A (cachyos host)
Tier 1  container recipe ran                  — arch-container · alpine-container · sles-bci  (retained, lower tier)
Tier 2  real QEMU VM recipe ran               — alpine-qemu · THIS Arch row (canonical Arch)   ◀
Tier 3  distro-native package build in VM     — not yet (PKGBUILD/abuild/rpmbuild from source)
Tier 4  official package-system integration   — not claimed
Tier 5  distro acceptance / default `zic`     — not claimed
```

## Non-claims (loud)

```text
Arch QEMU VM recipe ran  != official Arch package acceptance
                         != native PKGBUILD/makepkg package build (binary is host-built, not in-VM-built)
                         != distro policy / pacman-hook integration
                         != universal /usr/sbin/zic replacement
data-ISO transport       != network/pacman install path
binary host-built        != Arch-native (in-VM) build reproducibility (a Tier-3 rung)
```

- The reference `zic` was **not** run in-VM (offline data-ISO transport); reference-*behaviour* parity is the
  host/container work (`RECEIPT-host.md` §A: file-set 598=598, slim residuals behaviour-matched, the
  deliberate argv/exit-code divergences). This receipt proves **a host-built glibc zic-rs runs natively in a
  real Arch VM (kernel/glibc/loader identity recorded) + RPM-/PKGBUILD-shape compile + 598 files +
  cross-environment deterministic `bundle_hash`**. The next rung is an **in-VM `makepkg`/PKGBUILD-from-source**
  build (distro-native `binary_origin`) — archiso ships `/usr/bin/zic` and could also supply an in-VM
  reference-behaviour comparison in a future cut.
