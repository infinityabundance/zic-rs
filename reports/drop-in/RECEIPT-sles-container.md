# SLES BCI container package/build-recipe receipt — T23.drop-in-gauntlet.4.sles-bci

The **enterprise RPM / SUSE commercial-support** ecology, run hermetically in the freely-redistributable
SUSE Base Container Image. ABI-honest: the host **glibc PIE** (built against `GLIBC_2.39`) is **rejected by
the SLES glibc-2.38 loader** — recorded as a version/ABI mismatch, **not** a parity failure; the
**musl-static** zic-rs is the binary under test. Reference = the BCI's **own** `/usr/sbin/zic` (the SLES
`timezone` RPM lineage first measured at T16.5b.16). Evidence tier: **container-recipe-ran**.

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64 (glibc) · **podman** 5.8.2,
  `--network=none` (no network during the run). Harness: `reports/drop-in/sles-recipe.sh`.

## Required receipt fields

| field | value |
|---|---|
| **container image identity** | `registry.suse.com/bci/bci-base:latest` (image id `9b036947a10d`) → **SUSE Linux Enterprise Server 15 SP7**, `glibc-2.38-150600.14.46.1.x86_64` |
| **binary_origin** | **host-cross-built musl-static** (`x86_64-unknown-linux-musl`, sha256 `a9617888163dc776…`) — *under test*. Host **glibc PIE** (`target/release/zic-rs`, needs `GLIBC_2.39`) demonstrated **ABI-incompatible** here |
| **runtime ABI** | **musl-static** (fully static → runs on the SLES glibc-2.38 userland independent of its libc). glibc PIE = glibc, **rejected**: `libc.so.6: version 'GLIBC_2.39' not found` (SLES ships 2.38) |
| **timezone source identity** | host `/usr/share/zoneinfo/tzdata.zi`, sha256 `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (`2026b-dirty`), mounted **ro** (same source as the host/Arch/Alpine matrix) |
| **reference zic identity** | the BCI's own `/usr/sbin/zic` → **`zic (tzcode) 2025b`**, owner **`timezone-2025b-150600.91.6.2.x86_64`** (the SLES `timezone`-RPM tzcode lineage, T16.5b.16; commercial SLES ≡ free Leap ecology) |
| **zic-rs command matrix** | `compile --all-supported` · `compile -b slim --all-supported` · `size-report` · `--bogusflag` · `compile --input /nonexistent.zi` |
| **output file count** | **598** (zic-rs) **= 598** (reference `zic -d`) |
| **bundle_hash** | **`453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`** — **byte-identical** to the cachyos-glibc host, Arch-glibc container, Alpine-musl container, and Alpine-musl QEMU VM |
| **declared divergences** | (1) **argv shape** (`zic-rs compile --input … --out …` vs bare `zic -d`); (2) **usage-error exit code** — invalid flag: zic-rs `2` (clap) vs reference `1` (`missing-src` matches: `1`/`1`); (3) **slim byte residuals** — `-b slim` vs reference slim: **match 1 / diff 3** (4-zone probe), the documented structural residuals, **behaviour/`zdump`-matched (CORE.1)** |
| **non-claims** | (loud, below) |

## Result — captured in the booted BCI

```text
container userland: "SUSE Linux Enterprise Server 15 SP7"   glibc 2.38
host glibc PIE:  /lib64/libc.so.6: version `GLIBC_2.39' not found  (expected — SLES ships 2.38)
musl zic-rs:     zic-rs 0.1.0                 (runs)
reference zic:   zic (tzcode) 2025b           owner timezone-2025b-150600.91.6.2
compile exit:    0   reference zic -d exit: 0
file-set:        rs=598  ref=598
slim parity:     match=1 diff=3  (behaviour/zdump-matched residuals)
bundle_hash:     453641ff2568d8b1…            (IDENTICAL to host+Arch+Alpine-container+Alpine-VM)
argv matrix:     invalid-flag rs=2 ref=1 · missing-src rs=1 ref=1
status:          matched_with_declared_divergence (RAN)
```

**Cross-*libc* + cross-*environment* + cross-*distro-family* determinism extends to enterprise SUSE:** the
staged-tree `bundle_hash` is now identical across cachyos-glibc, Arch-glibc, Alpine-musl (container **and**
QEMU VM), **and SLES-15-SP7-glibc** — the output zoneinfo is byte-deterministic regardless of host/distro
libc, package family (pacman · apk · **RPM/zypper**), or environment.

**Recipe caveat (honest):** the glibc-PIE step prints `exit=0` because `$?` captured the trailing `head -1`
in the pipe, not the binary — the **real** evidence of the fail is the loader's `GLIBC_2.39 not found`
message (the dynamic loader never handed control to zic-rs). Not masked: stated.

## Non-claims (loud)

```text
SLES-BCI recipe ran (musl) != SLES package acceptance
                           != official RPM / `timezone`-package integration
                           != universal /usr/sbin/zic replacement
musl binary host-cross-built != SLES-native (in-distro) build reproducibility
glibc PIE rejected here      =  GLIBC version/ABI mismatch (2.39 vs 2.38), NOT a parity failure
data-mounted host tzdata.zi  != the SLES-shipped tzdata (SLES's own /usr/share/zoneinfo/tzdata.zi differs)
```

- This receipt proves **musl runtime in a real enterprise SLES userland + RPM-build-shape compile +
  cross-libc/cross-distro deterministic output + file-set match against the SLES `timezone`-RPM `zic`**.
  It does **not** claim SLES/SUSE acceptance, official `timezone`-RPM integration, exact stderr wording,
  whole-tree install semantics, or universal `/usr/sbin/zic` replacement. The next rung would be an
  **RPM/`rpmbuild`-from-source** package of zic-rs inside the BCI (distro-native `binary_origin`), needing
  the SLES Rust toolchain.
