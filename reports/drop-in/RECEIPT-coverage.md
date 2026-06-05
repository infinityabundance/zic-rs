# T23.drop-in-gauntlet.vm-sweep.close — Tier-2 VM sweep closure

> **Closes the Tier-2 real-QEMU-VM drop-in sweep.** Every serious Linux ecology is given a **typed
> disposition**: `vm_ran` (with a receipt) · `vm_ran_redundant` (ran, but adds no new Tier-2 axis) ·
> `attempted_unsuitable` (with evidence) · `redundant_covered` (by an existing row, with rationale) ·
> `new_axis_deferred` (would add an axis, not run, with rationale) · `not_applicable` (typed reason). A row
> is admitted as a *genuine* VM axis only if it contributes a new **libc/loader · `zic` lineage · package
> family · release cadence · enterprise lineage · binary-load outcome** — not merely another distro name.

## A. `vm_ran` — genuine new-axis VMs (9)

All: `compile=0 · files=598 · bundle_hash 453641ff2568d8b1…` (identical). Detail in
`RECEIPT-matrix.md` + the standalone arch/alpine QEMU receipts.

| VM | libc · loader | `zic` lineage | package family | cadence / lineage axis |
|---|---|---|---|---|
| **Arch 2026.06** | glibc 2.43 · FHS | tzcode (`tzdata`) | pacman | rolling |
| **Gentoo** | glibc 2.42 · FHS | tzcode (`timezone-data`) | source (`emerge`) | source-built |
| **Fedora 43** | glibc 2.42 · FHS | glibc (`glibc-common`) | rpm/dnf | **fast / RHEL-upstream** |
| **Debian 13** | glibc 2.41 · FHS | glibc (`libc-bin`) | deb | stable |
| **NixOS 25.11** | glibc 2.40 · **non-FHS → musl** | glibc (Nix store) | content-addressed | content-addressed |
| **openSUSE Leap 16** | glibc 2.40 · FHS | tzcode (`timezone` RPM) | SUSE rpm/zypper | SUSE (+ SLES-BCI Tier-1) |
| **Ubuntu 24.04** | glibc 2.39 · FHS | glibc (`libc-bin`) | deb | LTS |
| **AlmaLinux 9.8** | glibc 2.34 · **version-rejected → musl** | glibc (`glibc-common`) | rpm/dnf | **frozen el9 (RHEL-rebuild)** |
| **Alpine 3.23** | **musl** | tzcode (`tzdata-utils`) | apk | current-musl |

Axes covered: **libc** glibc 2.34/2.39/2.40/2.41/2.42/2.43 + musl · **loader** FHS-glibc / musl / non-FHS ·
**both `zic` lineages** · **7 package families** · **cadence** rolling/fast/LTS/stable/frozen/current ·
**binary-load** PIE-runs / version-too-old→musl / non-FHS→musl. The glibc-`zic` RPM family is **bracketed
end-to-end** (Fedora 2.42 ↔ AlmaLinux 2.34).

## B. `vm_ran_redundant` — ran, but adds no new Tier-2 axis (1)

| VM | result | why redundant |
|---|---|---|
| **Void Linux** (musl, live ISO) | ran: musl, `loader=none` → glibc-PIE not-found → **musl fallback** → 598 · `453641ff…` (`logs/void_dropin_vm/serial.log`) | Its **runtime** behaviour is already covered — musl = **Alpine**, non-FHS→musl = **NixOS**. The genuinely-new Void axis is **`xbps`** (its package manager), which only manifests at **Tier-3 source-build** (`xbps-src`), *not* the Tier-2 runtime. **Recorded honestly, not counted as a distinct Tier-2 axis** → its real contribution is a deferred Tier-3 item (B-list below). |

## C. `redundant_covered` — covered by an existing row (rationale)

| ecology | covered by | rationale |
|---|---|---|
| **Rocky Linux · Oracle Linux · CentOS Stream · RHEL** | **AlmaLinux 9.8** (+ Fedora) | bit-for-bit RHEL rebuilds / the same el9 glibc-`zic` lineage; CentOS Stream sits between Alma (rebuild) and Fedora (upstream) — already bracketed. RHEL itself is licence-gated and behaviourally ≡ Alma. |
| **Linux Mint · Pop!_OS · Devuan · Kali** | **Ubuntu / Debian** | Debian/Ubuntu derivatives, same `libc-bin` glibc-`zic`, deb family. |
| **Manjaro · EndeavourOS · CachyOS** | **Arch** (+ the host *is* cachyos) | Arch derivatives, same pacman tzcode-`zic`. |
| **SLES (commercial)** | **openSUSE Leap VM + SLES-BCI container** | same tzcode-`zic`-via-`timezone`-RPM lineage (T16.5b.16); the container is the commercial-SLES Tier-1 row. |
| **Mageia · OpenMandriva** | **Fedora / AlmaLinux** | rpm family, glibc-`zic`, no new lineage. |

## D. `new_axis_deferred` — would add an axis, not run (rationale)

| ecology | the new axis it would add | why deferred |
|---|---|---|
| **Void `xbps-src`** | the **xbps** package family at *build* time | belongs to the **Tier-3** source-build campaign, not Tier-2 runtime (the runtime row is redundant, §B). |
| **Chimera Linux** | musl + **apk-tools** + **BSD userland** on Linux — a genuinely novel mix | real but niche; runtime musl is already covered; deferred unless a consumer needs it. |
| **Slackware** | no dependency-resolving package manager (manual `installpkg`) | a distinct *packaging-absence* axis; niche; deferred. |
| **Clear Linux** | Intel-tuned glibc + aggressive optimisation flags | a *perf/optimisation* axis (→ better suited to the perf ledger than the determinism sweep); deferred. |

## E. `not_applicable` — typed reason

| ecology | reason |
|---|---|
| **FreeBSD · OpenBSD · NetBSD · DragonFly** · **illumos** (OmniOS/OI/SmartOS) | zic-rs is a **Linux ELF** — it cannot execute on a BSD/illumos kernel without a native build target. Their **vendor-oracle receipts stand** (T16.5b.1–7). Typed non-applicability, not a failure. |
| **Android** (bionic libc) | a real bionic-libc axis, but there is no zic-rs aarch64/bionic build artifact → `not_applicable` until such a target is built. |
| **Windows · macOS** | out of the Linux drop-in campaign scope (macOS operator-dropped earlier; Windows is a separate install model). |

## Closure statement

**The Tier-2 real-QEMU-VM drop-in sweep is COMPLETE for serious Linux ecologies.** Nine genuine-axis VMs ran
(identical `bundle_hash`); every other candidate is dispositioned `redundant_covered`, `new_axis_deferred`,
or `not_applicable` with rationale, and Void is recorded honestly as `vm_ran_redundant`. No serious Linux
ecology remains un-classified. **Non-claims unchanged:** VM-ran ≠ native package build ≠ distro acceptance ≠
universal `/usr/sbin/zic` replacement. Next rungs (post-sweep): the **system-slot install matrix** (harness
staged: `system-slot.sh`) and the **adoption-readiness** docs (FMEA, SBOM/signing, constrained-env profile).
