# Install / materialization contract (T17.4)

> **Doctrine.** *A compiled output tree is not merely bytes; it is bytes placed through **paths, names,
> links, renames, and durability boundaries**. zic-rs claims only the materialization properties it can
> **enforce and test**, and refuses every stronger one explicitly.* This is the filesystem-integrity
> companion to [`risk-register.md`](risk-register.md) (`RISK.PATH.1`, `RISK.INSTALL.1`),
> [`zic-hostile-output-tree.md`](zic-hostile-output-tree.md) (T14.6 hostile-tree boundary), and the
> module header of `src/fs/atomic_write.rs`. It states the *contract*; those state the surrounding
> evidence. Where a guarantee is platform-scoped or unclaimed, it says so — no "we are crash-durable now"
> hand-wave.

Materialization is the choke point in `src/fs/output_tree.rs` + `src/fs/atomic_write.rs`. Every write of a
compiled artifact goes through it; nothing else in the crate touches the output tree.

## The policies (each: rule · enforcement · status)

| Surface | Policy | Enforced by | Status |
|---|---|---|---|
| **Path traversal** | a zone/link *name* may never escape `--out`: non-empty, `/`-separated, no empty/`.`/`..` component, not absolute, no leading-`-` component, no NUL | `safe_relative_path` + `is_contained` defence-in-depth → `ZIC008` | **guarded** (`tests/output_safety.rs`, `tests/zone_name_path_policy.rs`) |
| **Explicit output only** | no implicit system install; output lands strictly under the caller's `--out` | required `--out`; `is_contained` | **guarded** (bucket-3 safer divergence vs `zic`) |
| **No partial install after a fatal** | compile *all* selected zones to memory, then materialize; a fatal leaves nothing half-written | T9.3 two-phase `plan::run` | **guarded** |
| **Temp file** | content is written to a `create_new` (`O_EXCL`) temp in the *same directory* (one filesystem, no cross-device rename), unique name (`pid`+counter) | `atomic_write::write_atomic` | **guarded** |
| **Content fsync** | the temp file is `sync_all`'d **before** publish — the published name never points at unflushed bytes (layer 1) | `write_atomic` | **guarded** |
| **Atomic publish — regular file** | default = `hard_link` (atomic **exclusive create**, `EEXIST` if the leaf exists — *no* `exists()`-then-create race, never follows a pre-planted symlink); `--force` = `rename` (atomic replace, replaces not follows) (layer 2) | `write_atomic` | **guarded** (`tests/hostile_output_tree.rs`) |
| **Atomic publish — symlink** | default = `symlink(2)` as **exclusive create** (`EEXIST` race-free, T17.4); `--force` = **temp-symlink + `rename`** (atomic, operates on the link itself — replaces a pre-planted symlink/file, never writes through it; no remove-then-create gap, T17.4) | `output_tree::{symlink_exclusive, write_link}` | **guarded** (`fs::output_tree::tests::symlink_*`, T17.4) |
| **Clobber** | never overwrite without `--force`; the exclusive-create publish makes "refuse to clobber" race-free | `write_atomic` / `symlink_exclusive` | **guarded** |
| **Directory-entry fsync (durability layer 3)** | on the **install** path, the parent directory is fsync'd after publish, so the new directory *entry* is crash-durable (not just the file content); ephemeral scratch writers (`compare` oracle tree, release-diff zdump tree) pass `durable=false` and skip it | `atomic_write::fsync_dir` (Unix) gated by `durable` | **guarded on Unix** (`fs::output_tree::tests::durable_write_succeeds_and_round_trips`); a documented no-op on non-Unix |
| **File mode (`-m`)** | octal-only, validated before any write, Unix-only, applied to TZif files + **copied** links, **never** symlinks | T9.5 `set_file_mode` (cfg-gated) | **guarded** |
| **Owner/group (`-u`)** | not implemented; no flag, so it cannot be mistaken for a no-op | — | **deferred** (privileged Unix-only; named, not faked) |

## The crash-durability claim — exactly what is and is not promised

**Claimed (Unix):** *per-file crash-durable publish.* Each written file goes through **content fsync →
atomic publish → parent-directory fsync**, so after a *successful* run each file's content **and** its
directory entry survive a power loss. This is the standard durable-publish idiom, implemented (T17.4), not
asserted.

**Not claimed (`RISK.INSTALL.1` non-claim `does_not_claim_whole_tree_crash_atomic_install`):**
**whole-tree crash-atomicity.** There is no tree-level transaction — a crash *mid-run* can leave some
files durably published and others not. The set is not atomic; each file is. Directory-entry durability is
a **Unix** guarantee (opening a directory and `sync_all`-ing it); on non-Unix `fsync_dir` is a documented
no-op, so the durability claim is Unix-scoped, stated honestly rather than faked.

## TOCTOU boundary — leaf closed, parent-component residual named (`RISK.PATH.1`)

**Closed (T17.4):** every *leaf* operation is check-then-act-free — regular files via exclusive
`hard_link` / atomic `rename`; symlinks via exclusive `symlink(2)` / temp-symlink+`rename`. A pre-planted
file/symlink/dir at the output leaf fails closed and is never written through.

**Still open, named precisely (`RequiresOpenatStyleHardening`):** a **concurrent parent-*component*
symlink swap** *during path resolution* — `create_dir_all`/`open` resolve parent symlinks at syscall time,
so an attacker racing a swap of an intermediate directory component mid-resolution is **not** defended.
Closing it needs fd-relative `openat`/`O_NOFOLLOW` materialization, which std does not expose without
`unsafe` or a new dependency — **both forbidden** by the project's `#![forbid(unsafe_code)]` + no-new-deps
posture. So it is recorded as an explicit non-claim (`does_not_claim_full_toctou_resistance`, now scoped
specifically to the parent-component race), owned by T20 (where the unsafe/dep decision could be revisited).

## Tested materialization cases

`tests/output_safety.rs` · `tests/hostile_output_tree.rs` (T14.6) · `tests/zone_name_path_policy.rs`
(T14.5) · `src/fs/output_tree.rs` unit tests (T17.4): durable write round-trips · symlink exclusive-create
rejects a pre-existing entry without following it · symlink `--force` overwrite replaces atomically. The
*un*tested-by-design cases are the ones that need power-loss simulation (crash mid-run) or hostile
concurrency (parent-component swap) — those are the explicit non-claims above, not silent gaps.

> **Boundary, one line:** *zic-rs durably and race-free-ly publishes each output file under an explicit
> root; it does not provide whole-tree crash-atomicity, and does not defend the parent-component symlink
> swap race — both are stated, not hidden.*
