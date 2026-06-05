# Drop-in receipt coverage — status tracker

> **🏁 The drop-in gauntlet is complete.** 15 actionable receipts across Linux, BSD, and illumos all build
> or run zic-rs through their **native operating environments** and emit the same **598-file TZif tree** with
> `bundle_hash 453641ff2568d8b110441731cb84df3b73082615af0d7ba8fe02f2b767133ec6`; remaining non-green rows
> are classified as **access/scope boundaries, not zic-rs failures** (SLES gated image, host baseline, Yocto
> producer/consumer). zic-rs has produced a **byte-identical installed timezone tree from the same crates.io
> source crate** across every actionable major Unix-like family tested: Linux/glibc, Linux/musl, FreeBSD,
> OpenBSD, NetBSD, DragonFly, OmniOS, OpenIndiana, and SmartOS/illumos.
>
> **The determinant is the zic-rs source + declared recipe** — *not* libc, kernel, package manager, VM
> transport, filesystem layout, default shell ecology, or Rust minor version. This is no longer a "port"; it
> is a **cross-Unix reproducibility court** around a Rust timezone compiler.
>
> **Crown-jewel lesson (SmartOS):** *do not force the platform into your expectation; find the platform's own
> admissible path and make the receipt say exactly what happened.* SmartOS is a stateless RAM global zone
> with no persistent `/usr` — rather than fight it, the build used the platform's **own setup wizard** to
> install pkgsrc into the GZ on a zones zpool, then built natively. The honest "impatience" notes in the
> receipts (stalled downloads, partial pkg caches, weird installers; the result still came from each
> platform's own recovery path) are a *credibility feature* — they show a real lab, not a sanitized demo.

Tracks the **drop-in** receipt (does zic-rs *itself* run / build-and-run there, producing 598 TZif from IANA
2026b with `bundle_hash 453641ff…`) against the **vendor-oracle** receipt list (native-`zic` harvests). One
row per vendor-oracle VM. Updated as each receipt lands.

Legend: ✅ done · ⬜ to do · ⚠️ partial · n/a.

| vendor-oracle VM | drop-in done? | receipt file | how |
|---|---|---|---|
| archlinux        | ✅ | RECEIPT-arch-container.md / -vm.md / -source.md | container + VM + source-build |
| alpine_3234      | ✅ | RECEIPT-alpine-container.md / -vm.md / -source.md | container + VM + source-build |
| debian_13        | ✅ | RECEIPT-matrix.md | VM |
| ubuntu_2404      | ✅ | RECEIPT-matrix.md | VM |
| almalinux_9      | ✅ | RECEIPT-matrix.md | VM |
| opensuse_leap_16 | ✅ | RECEIPT-matrix.md | VM |
| gentoo_20260531  | ✅ | RECEIPT-matrix.md | VM |
| nixos_2511       | ✅ | RECEIPT-matrix.md | VM |
| freebsd_14       | ✅ | RECEIPT-freebsd.md | source-build (cargo install from crates.io) |
| openbsd_79       | ✅ | RECEIPT-openbsd.md | source-build |
| dragonfly_642    | ✅ | RECEIPT-dragonfly.md | source-build (cargo install from crates.io; tmpfs build, live img) |
| netbsd_101       | ✅ | RECEIPT-netbsd.md  | source-build (cargo install from crates.io; host relay for pkgsrc redirect) |
| omnios_illumos   | ✅ | RECEIPT-omnios.md  | source-build (illumos: `pkg install ooce/developer/rust`; cargo at `/opt/ooce/rust/bin`) |
| openindiana_hipster | ✅ | RECEIPT-openindiana.md | source-build via **disk install** (live ISO read-only lofi /usr; qmon-driven install → `developer/lang/rustc` + `developer/gcc-14`) |
| smartos_20260528 | ✅ | RECEIPT-smartos.md | source-build via setup-wizard zpool + **pkgsrc-in-GZ** (`pkgin install rust` + gcc); qmon-driven setup |
| sles_15sp7       | ⚠️ | RECEIPT-sles-container.md | container only — VM/source-build blocked (SLES image subscription-gated) |
| linux_glibc_host | n/a | RECEIPT-host.md | the host baseline row itself |
| yocto_poky_buildhost | n/a | — | build-system (the producer), not a stock VM |
| yocto_poky_target_runtime | n/a | — | TZif consumer, no on-device compiler |

## Coverage: 15 drop-in receipts · EVERY actionable row DONE 🎯

- **Linux** (glibc + musl, 9 distros): arch · alpine · debian · ubuntu · almalinux · opensuse-leap · gentoo · nixos (+ host baseline).
- **BSD** — **complete (4/4)**: ✅ freebsd · ✅ openbsd · ✅ **netbsd** · ✅ **dragonfly** (native source-build).
- **illumos** — **complete (3/3, the triad)**: ✅ **omnios** (cloud qcow2) · ✅ **openindiana** (headless disk install — live ISO's read-only lofi `/usr` can't build, so the installer was driven to a 24 GB disk with `qmon.py`, then the installed system built it) · ✅ **smartos** (setup-wizard `zones` zpool + pkgsrc-in-GZ; `pkgin install rust`). Three independent illumos builds, distinct binaries, identical output.

**Every stock vendor-oracle VM with an obtainable image now has a native drop-in.** Each emits the
**byte-identical** 598-file tree, `bundle_hash 453641ff2568d8b1…`, across glibc/musl/FreeBSD/OpenBSD/NetBSD/
DragonFly/illumos ABIs and rustc 1.74→1.96 — the crate source is the portable unit; output determinism is
independent of OS/libc/toolchain/build provenance.

## Remaining: none actionable

The only non-✅ rows are **not** zic-rs/platform limits: SLES (subscription-gated image → container-only,
`RECEIPT-sles-container.md`), the host baseline (is the reference), and the Yocto build-host/target pair
(a build-system producer + a TZif-consumer image, not stock VMs). Nothing else to run.

> **How the hard rows were beaten (the durable lessons):** live images with no serial console were driven
> over the **VGA framebuffer** via the QEMU monitor (`qmon.py` `screendump`/`sendkey`); a CDN 302-redirect
> that hangs NetBSD `ftp`/`pkg_add` was bypassed with a **host relay** (`nbproxy.py`); a *writable-but-full*
> live root (DragonFly) was given room with a **tmpfs nullfs-mounted over the package prefix**; a
> *read-only* live `/usr` (OpenIndiana text ISO) required a **headless disk install** (the tmpfs trick can't
> apply to a read-only prefix); and a **stateless RAM global zone** (SmartOS) was handled the SmartOS-native
> way — its **setup wizard installs pkgsrc into the GZ** on a persistent zpool. illumos rust also needs an
> explicitly-installed gcc as the linker (OI: `developer/gcc-14` for `/usr/gcc/14/bin/gcc`; SmartOS: pkgsrc gcc).

Each landed receipt followed the FreeBSD/OpenBSD pattern: in the VM → install Rust (platform `pkg`/`pkgsrc`/
`dports`) → `cargo install zic-rs --version 0.1.0 --locked` from crates.io → run `dropin-build-run.sh <plat>`
→ produces 598 TZif from 2026b, hashes via `size-report`, uploads. zic-rs is a Linux ELF; these non-Linux
platforms **build it natively** (Rust targets x86_64-unknown-{freebsd,netbsd,openbsd,dragonfly,illumos}).

**Session tooling (reusable):** `qemu/qmon.py` (drives a VGA-console guest headless via QEMU-monitor
`screendump`/`sendkey` — for live images with no serial: NetBSD, DragonFly); `qemu/nbproxy.py` (host HTTP
relay around CDN 302-redirects that hang BSD `ftp`/`pkg_add` — NetBSD); the gpt+wedge / overlay / tmpfs-over-
prefix disk recipes in the `*-dropin.sh` wrappers.

Blocked / not applicable: SLES (gated image; container-only), host (is the baseline), yocto×2 (build-system/consumer).

> Companion: an independent Rust TZif **witness**, `zdump-rs` (`../zdump-rs/`, published v0.3.0), now
> cross-checks the emitted TZif against reference `zdump` — see that crate's README (`T23.zdump-witness.1/.2/.3`).
