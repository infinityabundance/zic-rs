//! Atomic file replacement: write to a temporary file in the destination directory, then
//! `rename` it into place.
//!
//! Rename within a directory is atomic on POSIX, so a reader never observes a half-written
//! TZif file, and an interrupted run leaves either the old file or nothing — never a
//! truncated one. The temp file is created in the *same directory* as the target so the
//! rename stays within one filesystem (cross-device renames fail).
//!
//! **The durability contract (T17.4), three layers — claimed exactly, not more:**
//! 1. **content fsync** — the temp file's bytes are `sync_all`'d *before* the publish, so the
//!    published name never points at unflushed content.
//! 2. **atomic publish** — `hard_link` (exclusive create, default) or `rename` (`--force`) makes the
//!    name appear (or replace) in one step; a reader/crash never sees a partial file.
//! 3. **directory-entry fsync** — when `durable` is set (the *install* path, not ephemeral scratch),
//!    the **parent directory** is fsync'd after the publish, so the new directory entry itself
//!    survives a crash. Without this layer a crash after the rename can still lose the entry on some
//!    filesystems (the file content was durable, but the link to it was not).
//!
//! **What this does NOT claim (`RISK.INSTALL.1`):** a **whole-tree** crash-atomic install. Each file
//! is durably published, but a crash *mid-run* can leave some files published and others not — there is
//! no tree-level transaction. Directory-entry fsync is a **Unix** guarantee (opening a directory and
//! `sync_all`-ing it); on non-Unix it is a documented no-op, so the durability claim is Unix-scoped.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{io_at, Error, Result};

/// fsync a directory so a just-published entry within it is crash-durable (T17.4 layer 3). On Unix,
/// opening the directory read-only and `sync_all`-ing it flushes the directory inode. On non-Unix this
/// is a no-op (directory fsync is not portably available) — durability is therefore a Unix claim, stated
/// honestly rather than faked.
#[cfg(unix)]
pub(crate) fn fsync_dir(dir: &Path) -> Result<()> {
    io_at(dir, File::open(dir).and_then(|d| d.sync_all()))
}
#[cfg(not(unix))]
pub(crate) fn fsync_dir(_dir: &Path) -> Result<()> {
    Ok(())
}

/// Per-process counter so concurrent writes within one process get distinct temp names.
static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Atomically write `bytes` to `target`.
///
/// We always write the content to a temp file in the *same directory* (so the final step
/// stays within one filesystem) and then publish it atomically:
///
/// * `overwrite = false` (the default): publish with `hard_link(temp → target)`, which the
///   kernel performs as an **atomic exclusive create** — it fails with `EEXIST` if `target`
///   already exists. This closes the check-then-act (TOCTOU) race that a separate
///   `exists()` test would leave open: there is no window between testing and creating.
/// * `overwrite = true` (`--force`): publish with `rename`, which atomically replaces any
///   existing file.
///
/// Either way a reader never observes a partially-written file. When `durable` is set, the parent
/// directory is fsync'd after the publish (T17.4 layer 3) so the new directory *entry* is crash-durable
/// — the install path sets it; ephemeral scratch writes (e.g. the release-diff zdump tree, the `compare`
/// oracle tree) pass `false` to skip the (pointless, for soon-deleted files) directory fsync.
pub fn write_atomic(target: &Path, bytes: &[u8], overwrite: bool, durable: bool) -> Result<()> {
    let dir = target
        .parent()
        .ok_or_else(|| Error::config("target has no parent directory"))?;
    let file_name = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::config("target has no file name"))?;

    // Unique temp name: PID + a monotonic per-process counter. `create_new` guarantees we
    // never clobber a stray temp file (a crash leftover surfaces as an error, not a silent
    // truncate).
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{file_name}.tmp.{}.{seq}", std::process::id()));

    // Create with base mode 0644 on Unix (then the process umask applies), matching reference `zic`'s
    // data-file mode. Rust's default `create` mode is 0666, which under a permissive umask (e.g. 000)
    // would leave **world-writable** zoneinfo files and diverge from reference; 0644 base is both
    // reference-matching and safer (INSTALL-SEMANTICS.1). An explicit `--mode`/`-m` still overrides this
    // afterwards via `set_file_mode`, so this only sets the default when no mode was requested.
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o644);
    }
    let mut f: File = io_at(&tmp, opts.open(&tmp))?;
    io_at(&tmp, f.write_all(bytes))?;
    io_at(&tmp, f.sync_all())?;
    drop(f);

    let result = if overwrite {
        std::fs::rename(&tmp, target).map_err(|e| Error::io(target, e))
    } else {
        // Atomic exclusive publish. `hard_link` fails if `target` exists, with no race.
        match std::fs::hard_link(&tmp, target) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(Error::config(format!(
                "{} already exists (use --force to overwrite)",
                target.display()
            ))),
            Err(e) => Err(Error::io(target, e)),
        }
    };

    // In the hard_link path the content now lives at both `tmp` and `target`; remove `tmp`.
    // In the rename path `tmp` is already gone. Best-effort cleanup either way.
    let _ = std::fs::remove_file(&tmp);
    result?;

    // Layer 3: make the directory ENTRY durable (the content was fsync'd pre-publish). Only on the
    // install path — for ephemeral scratch this would be wasted fsyncs on files about to be deleted.
    if durable {
        fsync_dir(dir)?;
    }
    Ok(())
}
