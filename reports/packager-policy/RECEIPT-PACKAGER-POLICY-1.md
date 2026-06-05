# RECEIPT — PACKAGER-POLICY.1 (distro/package policy compatibility matrix) — 2026-06-05

> **Claim wording (binding):** *PACKAGER-POLICY.1 does not claim distro adoption. It classifies what
> package-maintainer adaptations, if any, are required for zic-rs to occupy the `zic` build slot under
> representative distro packaging policies.*

## What grounds this matrix (not speculation)

- **PACKAGE-ACCEPTANCE.1** — zic-rs occupies the canonical tzcode/tzdata `$(ZIC)` build slot via
  `zic-shim-v1`; posix+right trees are file-set- and behaviour-identical (`bundle_hash 453641ff`).
- **SHIM-CONTRACT.1** — the shim is versioned + conformance-tested (8/8).
- **INSTALL-SEMANTICS.1** — disk materialization is equivalent-or-safer; the 0666→0644 mode bug is fixed.
- **Tier-3 source-builds** (`reports/drop-in/`) — zic-rs already **built from the crates.io crate** under
  Alpine `abuild`, Arch `makepkg` (real `.pkg.tar.zst`), Debian `cargo`, RPM `rpmbuild` (real `.rpm`), and on
  FreeBSD/OpenBSD/NetBSD/DragonFly/illumos.
- **Rootless/fakeroot grounding (this campaign):** the full package slot (posix+right+tables, **1199 files**)
  builds **under `fakeroot 1.38.1`** — files appear **root:root mode 644** in the staged archive while the
  real owner is unprivileged (`one:one`). The build never needs real root (`fakeroot-grounding.txt`).

**Distribution model:** zic-rs ships **only as a crates.io source crate** (`cargo install zic-rs --locked`) —
no binary release. So every recipe **builds zic-rs from the crate** as a build-dependency, then shims it.

## The matrix

| ecosystem | recipe / build tool | `zic` slot via shim | DESTDIR / staging | localtime policy | posixrules | hardlink→copy size | rootless/fakeroot | classification |
|---|---|---|---|---|---|---|---|---|
| **Debian/Ubuntu** | `debian/rules` (debhelper) | `make … ZIC='zic-shim.sh'` | `DESTDIR=debian/tmp` (clean, standard) | set by the **tzdata postinst** (debconf), *not* build-time `zic -t` → the `-t` refusal is irrelevant | already dropped (modern) | zoneinfo grows (no inode dedup); within distro limits | `dpkg-buildpackage -rfakeroot` ✓ | **works-with-shim** |
| **RPM (Fedora/SUSE)** | `.spec` `%build`/`%install` | `make install DESTDIR=%{buildroot} ZIC='zic-shim.sh'` | `%{buildroot}` (clean) | set by the **timezone** pkg scripts, not build-time `-t` | dropped | same copy note | `rpmbuild` runs non-root ✓ | **works-with-shim** |
| **Alpine** | `APKBUILD` / `abuild` | `make … ZIC='zic-shim.sh'` | `$pkgdir` (clean) | apk scripts, not build-time `-t` | n/a | copy note | `abuild` uses `fakeroot` ✓ (Tier-3 built the crate here) | **works-with-shim** |
| **Arch** | `PKGBUILD` / `makepkg` | `make … ZIC='zic-shim.sh'` | `$pkgdir` (clean) | hooks, not build-time `-t` | n/a | copy note | `makepkg` `package()` under `fakeroot` ✓ (Tier-3 built a real `.pkg.tar.zst`) | **works-with-shim** |
| **FreeBSD ports** | `Makefile` + `pkg-plist` | `ZIC=…` (base `zic` slot) | `STAGEDIR` (clean) | base `tzsetup`, not build-time `-t` | base policy | copy note changes `pkg-plist` sizes | ports build non-root ✓ | **requires-recipe-adaptation** (crate source-builds on FreeBSD per drop-in; the *port* Makefile + `pkg-plist` must be authored) |
| **pkgsrc / illumos** | `Makefile` + `PLIST` | `ZIC=…` | `DESTDIR` (clean) | OS scripts | OS policy | copy note changes `PLIST` | non-root ✓ | **requires-recipe-adaptation** (crate source-builds on illumos per drop-in; the pkgsrc Makefile + `PLIST` must be authored) |

**Tally:** 4 `works-with-shim` · 2 `requires-recipe-adaptation` · **0 `blocked-by-policy`** · 0 `not-tested`
(every row is at least policy-analyzed + grounded by a Tier-3 source-build on that platform/family).

## The required adaptations (the maintainer-consumable answer)

To put zic-rs in the `zic` build slot, a maintainer needs, in order:

1. **Build zic-rs from the crate** (`cargo install zic-rs --version 0.1.0 --locked`) as a build-dependency —
   there is no binary release. (Source-build proven across the Tier-3 ecosystems.)
2. **Point `$(ZIC)` at the argv shim** (`zic-shim-v1`) — zic-rs's CLI is a subcommand CLI (`--out` not `-d`)
   by design; the shim translates `-d/-L/-l/-t/-b/-D/-m files` with a tested contract.
3. **A clean DESTDIR/staging dir needs nothing extra** (the universal package norm). Only a rebuild *into a
   populated dir* needs `--force` (zic-rs's atomic no-clobber); package builds stage into a fresh root.
4. **Accept copy-not-hardlink** (a modest installed-size increase, no inode sharing) **or** pass
   `--link-mode symlink`. This is the one packager-facing footprint decision.
5. **Leave `/etc/localtime` to the package's post-install scripts** (where every distro already sets it) —
   zic-rs refuses an absolute/outside build-time `-t` (safer); see LOCALTIME-STAGING in `SHIM-CONTRACT.md`.
6. **Omit `-p posixrules`** (reference `zic` itself deprecates it; modern distros already dropped it).

Nothing here is a **policy block** — no representative distro requires an official upstream `zic` *binary*
(they all build from source), and the cargo-from-crates.io source model fits that. The two
`requires-recipe-adaptation` rows are *recipe authoring* (a ports Makefile / pkgsrc PLIST), not policy denial.

## Non-claims

- **No distro adoption claim** — no `.deb`/`.rpm`/`.apk`/port has been submitted or accepted; this is a policy
  + adaptation analysis grounded in the canonical build pattern, the Tier-3 source-builds, and a fakeroot run.
- The full per-distro *package* build (a complete `.deb`/port with the shim in the `zic` slot end-to-end) is
  **not** run here — the Tier-3 receipts built the crate + ran the matrix; the package-manager wrapper is the
  next rung, deliberately not claimed.
- Installed-size deltas are directional (copy-not-hardlink), not measured per-distro here (a LINK-MATERIALIZATION
  footprint receipt is the follow-up the review named).

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 520 tests unaffected; doc-staleness green.
Cross-linked from STATUS · `docs/replacement-readiness-ladder.md` · `reports/package-acceptance/` ·
`reports/install-semantics/` · `reports/drop-in/` (Tier-3).
