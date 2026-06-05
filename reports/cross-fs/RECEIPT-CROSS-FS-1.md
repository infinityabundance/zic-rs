# RECEIPT — CROSS-FS.1 (materialization across filesystem / container / rootless environments) — 2026-06-05

> **Claim wording (binding):** *CROSS-FS.1 does not claim universal filesystem portability. It tests zic-rs
> materialization policy across representative filesystem/container/rootless environments and records whether
> copy/symlink behaviour remains safe, deterministic, and package-usable.*

## Method

The same `tzdata.zi` 2026b built (copy and `--link-mode symlink`) across the reachable environments, checking
the deterministic `bundle_hash` (`453641ff2568d8b1`, the canonical host/container/VM/source-build value) and
whether a *link resolves* (read `America/Montreal` → `America/Toronto` through the materialized link). No
host mutation (everything under disposable dirs, cleaned up). Reproduce: `bash reports/cross-fs/gauntlet.sh`.

This host exposes **two distinct ext4 filesystems** (`/` and a separate mounted ext4 filesystem) + **tmpfs**
(`/dev/shm`, `/tmp`), so **real cross-filesystem relocation** — where hardlinks genuinely cannot follow — is
testable without root.

## Result — 9 rows, all deterministic-and-safe (0 broken)

| environment | fs | materialization | `bundle_hash` | link resolves | verdict |
|---|---|---|---|---|---|
| ext4 (home `/`) | ext4 | copy | `453641ff…` ✓ | yes | deterministic-and-safe |
| ext4 (home `/`) | ext4 | symlink (relative) | — | yes | deterministic-and-safe |
| tmpfs (`/dev/shm`) | tmpfs | copy | `453641ff…` ✓ | yes | deterministic-and-safe |
| tmpfs (`/dev/shm`) | tmpfs | symlink (relative) | — | yes | deterministic-and-safe |
| ext4 (2nd, separate mount) | ext4 | copy | `453641ff…` ✓ | yes | deterministic-and-safe |
| ext4 (2nd) | ext4 | symlink (relative) | — | yes | deterministic-and-safe |
| **relocate copy** (tmpfs → ext4, `cp -a`) | ext4 | copy | `453641ff…` ✓ | yes | deterministic-and-safe |
| **relocate symlink** (tmpfs → ext4) | ext4 | symlink (relative) | — | **yes** | deterministic-and-safe |
| rootless (uid 1000, no root) | all | copy/symlink | `453641ff…` ✓ | yes | deterministic-and-safe |

- **Determinism holds across filesystems:** the copy tree's `bundle_hash` is **identical** on both ext4
  filesystems and tmpfs — materialization is filesystem-independent.
- **Relocation-safe:** the copy tree survives a cross-filesystem `cp -a` with an unchanged `bundle_hash`; the
  **relative** symlinks (`../America/Toronto`, not absolute) **still resolve after relocation**.
- **Rootless:** every build ran unprivileged (uid 1000, no root); fakeroot is grounded in PACKAGER-POLICY.1.

## The hardlink-fragility contrast (why the copy default exists — now demonstrated, not asserted)

Reference `zic`'s hardlink tree has **341 unique inodes** (257 links share their target's inode). A **naive
cross-filesystem copy** (`cp -r`, the shape of an overlayfs copy-up or a partial layer copy) **breaks that
sharing → 598 unique inodes** — the compactness is *filesystem-bound* and does not survive relocation intact.
zic-rs's copy (598 inodes) and relative-symlink trees have **no inode-sharing to break**: they are relocatable
by construction. This is the concrete justification for the copy default + symlink option (bucket-3).

## What could not be tested directly (recorded, not claimed)

- **A true no-hardlink filesystem** (e.g. FAT) and **a case-insensitive mount** need `mount` (root), which is
  unavailable here. Honest substitutes: (a) zic-rs copy/symlink use **no hardlinks by construction**, so they
  are no-hardlink-safe — and the cross-fs relocation above is the hardlink-hostile proxy (hardlinks cannot
  span filesystems); (b) tzdb zone names are **case-collision-free by design** (the database targets
  case-insensitive filesystems), and zic-rs's name policy (T14.5 `ZoneNamePathPolicy`) already classifies
  case-collisions as platform-dependent/deferred. A real FAT/CI-mount run is a future operator step.
- **overlayfs** likewise needs root to mount; `cp -a`/`cp -r` across filesystems is the copy-up proxy used
  above (an overlayfs copy-up is exactly a per-file copy from a lower to an upper layer).

## Non-claims

- **Not universal filesystem portability** — bounded to ext4/tmpfs + cross-fs relocation + rootless on this
  host; FAT/CI/overlayfs-proper are proxied or deferred (above), not directly claimed.
- The `bundle_hash` determinism is the **content** invariant; mtime is still write-time (per INSTALL-SEMANTICS).
- zic-rs intentionally does **not** offer a hardlink mode (it would reintroduce the filesystem-bound fragility
  demonstrated here); symlink is the compact relocatable middle option.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 520 tests unaffected; doc-staleness green. Cross-linked
from STATUS · `reports/link-materialization/` · `reports/install-semantics/` · `reports/packager-policy/`.
