# Alpine **QEMU VM** package/build-recipe receipt — T23.drop-in-gauntlet.3.alpine (VM rung)

Upgrades the Alpine *container* recipe to a **real booted QEMU VM**: the musl-built `zic-rs` run inside an
actual Alpine Linux kernel+userland (not a shared container kernel). The lab boots it; the core admits this
receipt. Evidence tier: **VM-recipe-ran** (above `container-recipe-ran`).

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64.
- **VM:** `qemu-system-x86_64 -M q35 -m 1024 -enable-kvm -cpu host`, boot CD
  `alpine-virt-3.23.4-x86_64.iso` (sha256 `f802033362595ad5…`) → **Alpine Linux v3.23**, kernel
  `6.18.22-0-virt`, **musl** userland.
- **Transport: a data ISO, NO network** (eliminates the HTTP/DHCP fragility): second CD-ROM
  `qa-data.iso` (sha256 `c69b081a491fcca8…`) carrying the **host-cross-built musl-static** `zic-rs`
  (sha256 `a9617888163dc776…`) + `tzdata.zi` + `recipe.sh`. Mounted in-guest (`mount -t iso9660 /dev/sr1
  /mnt`), recipe run from `/mnt`.
- **Automation:** a Python serial-console driver (boots the VM, logs in `root` on the live ISO, mounts the
  data ISO, runs the recipe, captures the `RBEGIN…REND` block). Lab artifacts (external, not vendored into
  core): `../zic-rs-vendor-oracle-lab/qemu/alpine-dropin-vm-{drive.py,recipe.sh}` ·
  `logs/alpine_dropin_vm/serial.log`.

## Result — captured from the booted VM serial console

```text
MNT=recipe.sh,tzdata.zi,zic-rs-musl,        # data ISO mounted in-guest
osrel="Alpine Linux v3.23"
uname=Linux x86_64
ldd=/lib/ld-musl-x86_64.so.1                # MUSL runtime confirmed in the VM
zic-rs 0.1.0   ver=0                        # musl zic-rs runs in the VM
compile=0
files=598
bundle_hash: 453641ff2568d8b1…             # IDENTICAL to host(glibc) + Arch(glibc) + Alpine-container(musl)
```

| field | value |
|---|---|
| `binary_origin` | host-cross-built **musl-static** |
| `runtime_abi` | musl (in a real Alpine VM kernel) |
| compile exit / file-set | 0 / **598** |
| `bundle_hash` | `453641ff2568d8b1…` — **identical** across cachyos-glibc host, Arch-glibc container, Alpine-musl container, **and this Alpine-musl VM** |
| recipe evidence tier | **VM-recipe-ran** |
| status | `matched_with_declared_divergence (RAN in VM)` |

**The witness:** zic-rs's output zoneinfo is **byte-deterministic across libc (glibc/musl), across
environment (host/container/VM), and across kernel (host vs a separately-booted Alpine kernel)** — the
strongest reproducibility signal in the gauntlet so far.

## Non-claims (loud)

```text
VM recipe ran (musl)        != Alpine apk acceptance
                            != official APKBUILD / abuild-from-source
                            != universal /usr/sbin/zic replacement
musl binary host-cross-built!= Alpine-native (in-VM) build reproducibility
data-ISO transport          != network/package-manager install path
```

- Reference `zic` was **not** run in-VM (offline data-ISO transport); reference-*behaviour* comparison is the
  host/Arch work. This receipt proves **musl runtime in a real Alpine VM + package-build shape + cross-libc/
  cross-VM deterministic output**. The next rung is an **Alpine-native `abuild`-from-source** build of `zic-rs`
  inside the VM (needs the Alpine Rust toolchain) → that would move `binary_origin` from host-cross-built to
  distro-native.
