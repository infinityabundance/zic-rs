# Arch container package/build-recipe receipt — T23.drop-in-gauntlet.2.arch

The first **Tier-B container package-recipe** actually executed: zic-rs driven through a PKGBUILD/tzdata-build
shape **inside a real `archlinux:latest` container, hermetically (`podman --network=none`)**, compared to a
reference `zic`. Converts the Arch matrix row from `vendor-receipt ✅ / recipe_not_yet_run` →
**`container recipe RAN`**.

- **Date:** 2026-06-03 · **Engine:** `podman run --rm --network=none` · **Image:**
  `docker.io/library/archlinux:latest` (`a6f760002b3b`) · **Recipe:** `reports/drop-in/arch-recipe.sh`.
- **Mounts (ro):** host `target/release/zic-rs` (glibc PIE → runs in the Arch glibc userland) ·
  `/usr/bin/zic` + `/usr/bin/zdump` (host **Arch-family tzcode 2026b**, the reference) ·
  `/usr/share/zoneinfo/tzdata.zi` (source). Output dir rw.
- **Honest scope:** Arch's *own* `tzdata` package is **not** installed (`--network=none` ⇒ no `pacman`); the
  reference `zic` is the **mounted host Arch-family tzcode 2026b**. The container supplies the **Arch
  userland**; this proves zic-rs runs there and is drivable through a package-build shape — *not* that Arch's
  packaged `zic` was rebuilt.

## Result (executed in-container)

| check | outcome |
|---|---|
| container userland | `"Arch Linux"` |
| zic-rs runs in Arch userland | `zic-rs 0.1.0`, `--version` exit **0** |
| reference `zic` | `zic (tzcode) 2026b-dirty` (mounted) |
| PKGBUILD-style compile `--all-supported tzdata.zi` | zic-rs exit **0**; reference `zic -d` exit **0** |
| **output file-set** | **rs = 598 · ref = 598 — MATCH** |
| **`bundle_hash` (size-report over staged tree)** | **`453641ff2568d8b1…` — IDENTICAL to the host perf receipt** (`reports/perf/RECEIPT-x86_64-2026-06-02.md`) → **deterministic across environments** (clean Arch container = cachyos host; no host contamination) |
| slim byte parity (4 zones) | match 0 / diff 4 → **accepted divergence**: zic-rs default is *fat* (behaviour-matched); slim has documented **structural residuals** vs reference slim (bucket-3); byte parity claimed only where pinned · behaviour/`zdump`-matched (CORE.1) |
| invalid flag | rs-exit **2** / ref-exit **1** → **accepted divergence** (documented exit taxonomy: clap-usage 2 vs `zic` 1) |
| missing source | rs-exit **1** / ref-exit **1** → **MATCH-class** |

**Status:** `matched_with_declared_divergence (RAN)`. The output tree matches by file-set and is
byte-deterministic (bundle_hash) with the host; the divergences are the deliberate CLI shape, the usage-error
exit code, and the documented slim structural residuals.

## Non-claims (loud)

```text
container recipe ran        != Arch package acceptance
                            != official PKGBUILD approval
                            != universal /usr/sbin/zic replacement
                            != all pacman hooks
                            != all downstream scripts
                            != all distro policy
```

- The recipe proves only that **this Arch-like container recipe executed** and that zic-rs can be driven
  through this package/build-script shape with classified outputs. Arch's packaged `zic` was not rebuilt
  (`--network=none`); exact stderr wording, pacman hooks, and distro policy are not claimed.
- **Next:** Alpine (musl/base-image, a different libc/package world) as `T23.drop-in-gauntlet.3.alpine`.
