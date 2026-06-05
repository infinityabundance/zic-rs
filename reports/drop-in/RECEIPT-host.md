# Drop-in / package-recipe gauntlet — receipt (T23.drop-in-gauntlet.1, first cut)

> **The biggest *universal* `/usr/sbin/zic`-replacement mover — and the one with the most overclaim risk.**
> zic-rs is **deliberately not a literal argv drop-in** (subcommand CLI + required `--out`; see
> `docs/drop-in-compatibility-contract.md`). This gauntlet **quantifies that gap** against reference `zic`,
> and enumerates the curated lab ecologies by **recipe mode** — making **no** claim of distro acceptance or
> universal replacement beyond the exact rows actually executed.

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64 (glibc; an Arch-like rolling Linux) ·
  reference `zic`/`zdump` tzcode 2026b · source `/usr/share/zoneinfo/tzdata.zi`. Harness:
  `reports/drop-in/gauntlet.sh`.

## A. Argv / install-tree / error matrix — **RAN on this host** (Tier A, one representative ecology)

| row | ref-exit / rs-exit | classification |
|---|---|---|
| compile → tree (default) | 0 / 0 | **argv DIVERGE** (zic-rs: `compile --input … --out … --all-supported`); **output file-set MATCH (598 = 598)**; bytes diverge-by-default (zic-rs fat default vs `zic` slim — documented bucket-3) |
| `-b slim` byte parity (4 zones) | 0 / 0 | **accepted divergence** — match 1 / diff 3; the diffs are **structural residuals** (timecnt/footer), **behaviour/`zdump`-matched (CORE.1)**; full byte parity claimed only where pinned |
| `-L` right/ profile | 0 / 0 | **file-set MATCH (598 = 598); `right/Etc/UTC` bytes MATCH** (after the T23.reader-compat.2 leap fix) |
| `--version` | 0 / 0 | **argv MATCH** (both print + exit 0) |
| invalid flag | 1 / 2 | **accepted divergence** — both reject, but exit code differs (`zic` 1 / zic-rs clap-usage 2; documented exit taxonomy) |
| missing source file | 1 / 1 | **MATCH-class** (both nonzero operational error) |
| `-p posixrules` | 0 / n/a | **not-claimed** — reference accepts but warns *"-p is obsolete and likely ineffective"*; zic-rs does not implement `-p` |

**Headline:** where invoked equivalently, **output trees, the `right/` profile, and broad error dispositions
are compatible**; the divergences are exactly the **deliberate CLI shape** (subcommand + `--out`), the
**usage-error exit code** (1 vs 2), **slim byte residuals** (behaviour-matched), and **`-p`** (obsolete even
in `zic`). **Exact stderr wording is not claimed.** This is **not** a universal argv drop-in — by design.

## B. Curated lab-ecology package/build-recipe matrix (recipe mode + status — receipt-bounded)

Every row already has a **vendor-oracle receipt** (T16.5b) — that is *not* a package recipe. Recipe modes:
**A** runnable here now · **B** container recipe feasible (not yet run) · **C** VM/operator recipe from a lab
image · **D** build-host-only (producer) · **E** out of scope.

| ecology | `zic` lineage (T16.5b) | package/build system | recipe mode | status |
|---|---|---|---|---|
| **this host (glibc Linux)** | reference tzcode | direct CLI | **A — RAN** | **matched_with_declared_divergence** (matrix A above) |
| Arch | tzcode (`tzdata` pacman) | PKGBUILD | **B — RAN** | **container recipe RAN** (`podman --network=none archlinux:latest`): file-set 598=598, `bundle_hash` host-identical, `matched_with_declared_divergence` — `RECEIPT-arch-container.md` |
| Alpine | tzcode (`tzdata-utils`, musl) | APKBUILD | **B — RAN (musl)** | **container recipe RAN** (`python:3.12-alpine`, musl-built static zic-rs): file-set 598, **bundle_hash host-identical (cross-libc)**, ABI boundary demonstrated — `RECEIPT-alpine-container.md` |
| Debian | glibc-zic (`libc-bin`) | deb/`rules` | B (container) | recipe_not_yet_run · vendor receipt ✅ |
| Ubuntu | glibc-zic (`libc-bin`) | deb | B (container) | recipe_not_yet_run · vendor receipt ✅ |
| AlmaLinux / RHEL | glibc-zic (`glibc-common`) | RPM | B (container; real RHEL not claimed) | recipe_not_yet_run · vendor receipt ✅ |
| openSUSE Leap | tzcode (`timezone` RPM) | RPM/zypper | B (container) | recipe_not_yet_run · vendor receipt ✅ |
| SLES | tzcode (`timezone` RPM) | RPM (BCI container) | B (BCI) | recipe_not_yet_run · vendor receipt ✅ |
| NixOS | glibc-zic (`/nix/store`) | derivation | C (VM/derivation) | recipe_not_yet_run · vendor receipt ✅ |
| Gentoo | tzcode (`sys-libs/timezone-data`) | ebuild | C (VM/operator) | recipe_not_yet_run · vendor receipt ✅ (selected tzcode-zic despite glibc 2.42) |
| Yocto/Poky | tzcode-native (build host) | bitbake recipe | **D — build-host only** | recipe_not_yet_run · producer ≠ on-device `zic` (target = consumer) |
| FreeBSD | tzcode 2022g (base) | base/ports | C (VM/operator) | recipe_not_yet_run · vendor receipt ✅ |
| NetBSD | tzcode 2022g (base) | base | C (VM/operator) | recipe_not_yet_run · vendor receipt ✅ |
| OpenBSD | tzcode-fork (base) | base | C (VM/operator) | recipe_not_yet_run · vendor receipt ✅ |
| DragonFly | oldest tzcode-fork (base) | base | C (VM/operator) | recipe_not_yet_run · vendor receipt ✅ (cannot ingest modern zishrink `tzdata.zi`) |
| OmniOS / illumos | tzcode 2025a (`pkg`/IPS) | IPS | C (VM/operator) | recipe_not_yet_run · vendor receipt ✅ |

**Several representative rows ARE run** (the Tier-A host matrix in §A); the rest are enumerated honestly as
**recipe_not_yet_run** with their feasible mode. No row implies distro acceptance.

## C. Executed recipes — ABI/origin-typed, evidence-tier-ranked (VM-canonical doctrine)

> **VM-canonical doctrine (operator-directed):** a real booted OS image is the preferred drop-in witness;
> container rows are retained as **lower-tier** evidence. Where a VM row exists for an ecology, **it is the
> canonical row** and the container row is superseded (kept for provenance).

**Evidence-tier ladder** (kept apart, never collapsed):

```text
Tier 0  host baseline           Tier 3  distro-native package build in VM/container
Tier 1  container recipe ran    Tier 4  official package-system integration
Tier 2  real QEMU VM recipe ran Tier 5  distro acceptance / default system `zic`
```

| ecology | `binary_origin` | `runtime_abi` | **tier** | file-set | `bundle_hash` | status |
|---|---|---|---|---|---|---|
| **Arch (glibc, QEMU VM)** ◀ canonical | host-built glibc PIE (runs natively) | glibc 2.43 (real Arch VM, kernel `7.0.10-arch1`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-arch-vm.md` |
| Arch (glibc, container) — superseded | host-built glibc PIE (mounted) | glibc | Tier 1 — container ran | 598 = 598 vs ref | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-arch-container.md` |
| **Alpine (musl, QEMU VM)** ◀ canonical | host-cross-built musl static | musl (real Alpine v3.23 VM, kernel `6.18-virt`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-alpine-vm.md` |
| Alpine (musl, container) — superseded | host-cross-built musl static | musl | Tier 1 — container ran | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-alpine-container.md` |
| **Debian 13 (glibc/deb, QEMU VM)** | host-built glibc PIE (runs natively) | glibc 2.41 (trixie genericcloud, kernel `6.12`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` |
| **Ubuntu 24.04 (glibc/deb, QEMU VM)** | host-built glibc PIE (runs natively) | glibc 2.39 (real Noble VM, kernel `6.8.0-117`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` |
| **Gentoo (glibc/source, QEMU VM)** | host-built glibc PIE (runs natively) | glibc 2.42 (minimal live ISO, kernel `6.18.33`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` |
| **openSUSE Leap 16 (RPM/SUSE, QEMU VM)** | host-built glibc PIE (runs natively) | glibc 2.40 (Leap-16 cloud, kernel `6.12`) | **Tier 2 — VM ran** (result-disk capture) | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` (supersedes the SLES-BCI Tier-1 container for SUSE-family VM evidence) |
| **AlmaLinux 9.8 (glibc/RPM, QEMU VM)** | host-cross-built musl static² | musl-on-glibc-2.34 (real el9 VM, kernel `5.14`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` |
| **Fedora 43 (glibc/RPM fast-cadence, QEMU VM)** | host-built glibc PIE (runs natively) | glibc 2.42 (real fc43 VM, kernel `6.17`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` |
| **NixOS 25.11 (glibc/content-addressed, QEMU VM)** | host-cross-built musl static³ | musl-on-glibc-2.40 (real Nix VM, kernel `7.0.10`) | **Tier 2 — VM ran** | 598 | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-matrix.md` |
| SLES 15-SP7 (glibc/RPM, BCI container) | host-cross-built musl static¹ | musl-on-glibc-2.38 | Tier 1 — container ran | 598 = 598 vs SLES `zic` | `453641ff2568d8b1…` | matched_with_declared_divergence — `RECEIPT-sles-container.md` |

¹ SLES BCI ships glibc 2.38; the host glibc PIE (needs `GLIBC_2.39`) is **rejected by the loader there** —
the version/ABI-honest fail, **not** a parity failure — so the musl-static binary is the SLES binary under test.
² AlmaLinux 9.8 ships glibc 2.34 (< 2.39) → same ABI-honest glibc-PIE rejection → musl-static under test.
³ NixOS is **non-FHS** (no `/lib64/ld-linux-x86-64.so.2`) → glibc-PIE `cannot execute` → musl-static under
test (a *different* fallback trigger than Alma's version mismatch — both ABI-honest, output identical).

**Cross-*libc* + cross-*environment* + cross-*VM*/kernel + cross-*distro-family* determinism:** the
staged-tree `bundle_hash` is **byte-identical** (`453641ff2568d8b1…`) across the cachyos-glibc host, **NINE
real QEMU VMs** (Arch glibc-2.43 · Gentoo glibc-2.42 · Fedora glibc-2.42 · Debian glibc-2.41 · NixOS
glibc-2.40 · openSUSE Leap glibc-2.40 · Ubuntu glibc-2.39 · AlmaLinux glibc-2.34 · Alpine musl), and the
Arch + Alpine + SLES-BCI
containers — deterministic regardless of build libc (**glibc 2.34/2.38/2.39/2.40/2.41/2.42/2.43 + musl**),
runtime environment (host/container/VM), kernel (8 distinct), loader (incl. NixOS non-FHS → musl), or package
family (**pacman · apk · deb · RPM/zypper · SUSE · source · content-addressed**). **Every serious Linux
ecology with a bootable image is now a Tier-2 VM row.** No host/libc/kernel contamination. See
`RECEIPT-matrix.md` for all rows, the three binary-load outcomes (PIE-runs ·
version-too-old→musl · non-FHS→musl), the per-image capture methods (serial · cloud-init · result-disk), and
the BSD/illumos drop-in non-applicability (zic-rs is a Linux ELF).

## Non-claims (the brutal-honesty ladder — keep these apart)

```text
included in matrix      ≠ recipe run
recipe run              ≠ distro acceptance
vendor-oracle receipt   ≠ package compatibility
package compatibility   ≠ universal /usr/sbin/zic replacement
argv divergence is DELIBERATE (subcommand CLI + required --out), not a bug
```

- zic-rs is **not** a literal argv/system-`zic` drop-in (by design). Equivalent invocations produce
  compatible output trees / `right/` profile / broad error dispositions on this host; exact stderr wording,
  whole-tree install semantics, `localtime`/`posixrules`/distro-layout parity, and real distro packaging
  acceptance are **not** claimed. **VM-canonical doctrine now in force:** real QEMU VM > container > host;
  the canonical Arch + Alpine rows are **Tier-2 VM** receipts. Next cuts: bring the remaining ecologies that
  already have local images to **Tier 2 (real QEMU VM)** — AlmaLinux 9 · Debian 13 · Ubuntu Noble · openSUSE
  Leap 16 · Gentoo · NixOS 25.11 · the BSDs (FreeBSD/OpenBSD/NetBSD/DragonFly) · illumos (OmniOS/OI/SmartOS) —
  then **Tier 3** distro-native package builds (PKGBUILD/abuild/rpmbuild/deb from source) inside the VMs.
