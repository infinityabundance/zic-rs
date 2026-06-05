# RECEIPT — INSTALL-SEMANTICS.1 (filesystem / materialization replacement semantics) — 2026-06-05

> **Claim wording (binding):** *INSTALL-SEMANTICS.1 does not claim official distro adoption. It records how
> zic-rs materializes, refuses, or preserves filesystem objects under package-relevant install conditions, and
> classifies differences from reference `zic` as match, safer-divergence, install-policy difference,
> unsupported-by-design, or divergent.*

## Method

Reference `zic` 2026b and zic-rs run on the **same** install fixtures under package-relevant conditions, in a
disposable root (**no host mutation**). Classified by what lands on disk. Reproduce:
`bash reports/install-semantics/gauntlet.sh` → `reports/install-semantics/install-semantics.tsv`.

## Result — 13 scenarios

| verdict | count |
|---|--:|
| **match** | **9** |
| install-policy-difference | 2 |
| safer-no-partial | 1 |
| unsupported-by-design | 1 |
| **divergent** | **0** |

| scenario | reference `zic` | zic-rs | verdict |
|---|---|---|---|
| existing file, no `--force` | overwrites in place | **refuses (atomic no-clobber)** | install-policy-difference (safer) |
| existing file, `--force` | overwrites | overwrites atomically (temp+rename) | match |
| dir where a file should be | exit 1 | exit 1 | match |
| read-only parent dir | exit 1 | exit 1 | match |
| **symlink → outside root at leaf** | victim preserved | victim preserved (symlink replaced, not followed) | match — **both safe** |
| umask 000 | mode 644 | mode 644 | match *(after the fix below)* |
| umask 022 | mode 644 | mode 644 | match |
| umask 077 | mode 600 | mode 600 | match |
| `-m 600` | mode 600 | mode 600 | match |
| `-D` into missing dir | exit 1 | exit 1 | match |
| **partial output on fatal** | writes-as-it-goes (2 files) | compile-all-then-write (0 files) | safer-no-partial |
| `-p posixrules` | deprecated ("obsolete") | ignored / unsupported | unsupported-by-design |
| link materialization | hardlink (nlink=2) | copy (nlink=1) | install-policy-difference |

**0 divergent.** Metadata table (zic-rs build, sample zone): `type=regular file · mode=644 · uid/gid=process
(staged) · nlink=1 (copy) · size=…`; **mtime = write-time** (content determinism is via `bundle_hash`, not
mtime).

## The real divergence found **and fixed** (a 1-line src change)

Under a permissive **umask 000**, zic-rs created files at base mode **0666 (world-writable)** — Rust's
`File::create` default — where reference `zic` uses base **0644**. Under the normal umask 022 both yield 0644,
so this only surfaced under umask 000, but a package build under a permissive umask would have shipped
world-writable zoneinfo. **Fixed:** `src/fs/atomic_write.rs` now creates the temp file with
`OpenOptions::mode(0o644)` on Unix (the process umask still applies; an explicit `--mode` still overrides) —
**reference-matching and never world-writable.** Content is unchanged (only the file mode), so **CORE.1 stays
341/0/0 byte-identical**; pinned by `tests/cli_operational_parity.rs::default_file_mode_is_0644_base_under_permissive_umask`
(runs under `umask 000`, asserts mode 0644, never group/other-writable). 520 tests.

## The two install-policy differences (documented bucket-3 *safer* policies)

1. **Atomic no-clobber.** zic-rs refuses to overwrite an existing output file without `--force` (reference
   overwrites in place); with `--force` it overwrites **atomically** (temp + rename). Safer: a re-run never
   half-writes over a live tree. A package build that expects in-place overwrite passes `--force`.
2. **Copy not hardlink.** zic-rs writes independent copies of linked zones (relocatable); reference hardlinks
   (shared inode). Identical file set + content; increases installed size — a packager-facing footprint
   choice (`--link-mode symlink` is available).

## What matched (the reassuring part)

`-m`, `-D`, umask (all three), the existing-dir/read-only/dir-collision error cases, **and the
symlink-to-outside-root leaf** all match reference — including the key safety property that **neither writes
through a symlink to clobber a victim outside the output root** (zic-rs additionally replaces the symlink with
a regular file rather than following it). The only fatal-time difference is zic-rs's **no-partial-output**
(safer).

## Non-claims

- **Not distro adoption.** Records filesystem materialization semantics + zic-rs↔reference parity, not that
  any distro has accepted zic-rs.
- Bounded to these 13 scenarios on this host/filesystem (ext4-class); cross-filesystem (tmpfs/overlay/
  case-insensitive) and rootless/fakeroot are tracked as future surfaces, not claimed here.
- `-u` ownership remains deferred (Unix-gated). mtime is not made deterministic (content determinism is the
  `bundle_hash` contract; mtime is write-time, like reference).

## Gate

`src/fs/atomic_write.rs` mode fix is content-neutral: **CORE.1 341/0/0** byte-unchanged; **520 tests**;
`fmt`/`clippy -D warnings` clean; doc-staleness green. Cross-linked from STATUS ·
`docs/differences-from-reference-zic.md` · `docs/zic-operational-parity-matrix.md`.
