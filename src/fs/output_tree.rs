//! The output tree: turning zone/link *names* into safe paths under the output root, and
//! materialising files there.
//!
//! Zone names are untrusted input (`Link ../../etc/passwd evil` is a thing an attacker
//! would try). This module is the choke point that guarantees **every** write lands
//! strictly inside the explicit `--out` directory. The rules, enforced in
//! [`safe_relative_path`]:
//!
//! * the name must be non-empty and use `/` as the only separator;
//! * no component may be empty, `.`, or `..` (no traversal, no current-dir tricks);
//! * the name must not be absolute (no leading `/`);
//! * no component may begin with `-` (avoids names that look like CLI options and other
//!   tooling foot-guns; `zic -v` merely warns, we reject — fail closed);
//! * no NUL bytes.
//!
//! Violations produce a `ZIC008_OUTPUT_PATH_TRAVERSAL` diagnostic.

use std::path::{Component, Path, PathBuf};

use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::error::{Error, Result};
use crate::fs::atomic_write::write_atomic;
use crate::LinkMode;

/// Validate a zone/link name and convert it to a relative path under the output root.
pub fn safe_relative_path(name: &str) -> Result<PathBuf> {
    let reject = |msg: &str| {
        Err(Error::from(Diagnostic::error(
            DiagnosticCode::OutputPathTraversal,
            format!("unsafe zone/link name {name:?}: {msg}"),
            Path::new("<output>"),
            0,
        )))
    };

    if name.is_empty() {
        return reject("name is empty");
    }
    if name.contains('\0') {
        return reject("name contains a NUL byte");
    }
    if name.starts_with('/') {
        return reject("name is absolute");
    }

    let mut path = PathBuf::new();
    for comp in name.split('/') {
        if comp.is_empty() {
            return reject("name has an empty path component");
        }
        if comp == "." || comp == ".." {
            return reject("name contains a '.' or '..' component");
        }
        if comp.starts_with('-') {
            return reject("a path component starts with '-'");
        }
        path.push(comp);
    }
    Ok(path)
}

/// Resolve `candidate` and `root` lexically (no filesystem access) and report whether
/// `candidate` is contained within `root`. Used as a defence-in-depth check after path
/// construction, and exposed for direct testing.
pub fn is_contained(root: &Path, candidate: &Path) -> bool {
    fn normalise(p: &Path) -> Vec<Component<'_>> {
        let mut out: Vec<Component<'_>> = Vec::new();
        for c in p.components() {
            match c {
                Component::CurDir => {}
                Component::ParentDir => {
                    // Pop a normal component; if there is nothing to pop, keep the `..`
                    // so an escape above the root is detectable below.
                    if matches!(out.last(), Some(Component::Normal(_))) {
                        out.pop();
                    } else {
                        out.push(c);
                    }
                }
                other => out.push(other),
            }
        }
        out
    }

    let root_n = normalise(root);
    let cand_n = normalise(candidate);
    cand_n.len() >= root_n.len() && cand_n[..root_n.len()] == root_n[..]
}

/// Write a compiled zone's bytes to `root`/`name`, creating parent directories.
///
/// Returns the path written. Refuses to escape `root`, refuses to overwrite unless
/// `overwrite`, and writes atomically. `durable` requests the install-grade
/// content-fsync + atomic-publish + **parent-directory-fsync** sequence (T17.4); ephemeral callers
/// (the `compare` oracle tree, the release-diff zdump scratch tree) pass `false`.
pub fn write_zone_file(
    root: &Path,
    name: &str,
    bytes: &[u8],
    overwrite: bool,
    durable: bool,
) -> Result<PathBuf> {
    let rel = safe_relative_path(name)?;
    let target = root.join(&rel);

    // Defence in depth: the construction above cannot produce traversal, but we verify
    // containment anyway in case `root` itself is unusual.
    if !is_contained(root, &target) {
        return Err(Error::from(Diagnostic::error(
            DiagnosticCode::OutputPathTraversal,
            format!("resolved path for {name:?} escapes the output root"),
            Path::new("<output>"),
            0,
        )));
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    write_atomic(&target, bytes, overwrite, durable)?;
    Ok(target)
}

/// Materialise a `Link`: either copy the target file's bytes to the link name, or create a
/// relative symlink, per `mode`. The target file must already have been written.
pub fn write_link(
    root: &Path,
    link_name: &str,
    target_name: &str,
    mode: LinkMode,
    overwrite: bool,
    durable: bool,
) -> Result<PathBuf> {
    // Both names are untrusted and must be safe.
    let link_rel = safe_relative_path(link_name)?;
    let target_rel = safe_relative_path(target_name)?;
    let link_path = root.join(&link_rel);
    let target_path = root.join(&target_rel);

    if !is_contained(root, &link_path) {
        return Err(Error::from(Diagnostic::error(
            DiagnosticCode::OutputPathTraversal,
            format!("link path for {link_name:?} escapes the output root"),
            Path::new("<output>"),
            0,
        )));
    }
    let parent = link_path.parent().map(|p| p.to_path_buf());
    if let Some(parent) = &parent {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }

    match mode {
        LinkMode::Copy => {
            let bytes = std::fs::read(&target_path).map_err(|e| Error::io(&target_path, e))?;
            write_atomic(&link_path, &bytes, overwrite, durable)?;
        }
        LinkMode::Symlink => {
            // A relative symlink keeps the tree relocatable.
            let rel_target = relative_link_target(&link_rel, &target_rel);
            // T17.4: publish the symlink without a check-then-act window.
            // * no-overwrite → `symlink()` is itself an **exclusive create** (fails `EEXIST` if the
            //   name exists), so there is no `exists()`-then-create race — symmetric with the
            //   `hard_link` path for regular files.
            // * `--force` → create the link at a temp name, then `rename` it over the target. `rename`
            //   acts on the *link itself* (never follows it), so a pre-planted symlink/file at the
            //   target is replaced, not written through — and there is no remove-then-create gap.
            if overwrite {
                let dir = parent.as_deref().unwrap_or(root);
                let tmp = symlink_temp_path(dir, &link_rel);
                symlink(&rel_target, &tmp)?;
                if let Err(e) = std::fs::rename(&tmp, &link_path) {
                    let _ = std::fs::remove_file(&tmp);
                    return Err(Error::io(&link_path, e));
                }
            } else {
                symlink_exclusive(&rel_target, &link_path)?;
            }
            // Layer-3 durability for the symlink's directory entry (T17.4), install path only.
            if durable {
                if let Some(parent) = &parent {
                    crate::fs::atomic_write::fsync_dir(parent)?;
                }
            }
        }
    }
    Ok(link_path)
}

/// A unique temp name for the atomic-symlink-overwrite publish, in the link's own directory.
fn symlink_temp_path(dir: &Path, link_rel: &Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let base = link_rel
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("link");
    dir.join(format!(".{base}.symlink.tmp.{}.{seq}", std::process::id()))
}

/// Compute the symlink body: a path from the link's directory to the target.
fn relative_link_target(link_rel: &Path, target_rel: &Path) -> PathBuf {
    // Number of directories to climb from the link to the root.
    let up = link_rel.components().count().saturating_sub(1);
    let mut out = PathBuf::new();
    for _ in 0..up {
        out.push("..");
    }
    out.push(target_rel);
    out
}

/// Create a symbolic link. Isolated here so the platform `cfg` lives in one place; the rest
/// of the crate stays platform-agnostic. Used for the `--force` path's temp link (a unique name that
/// cannot pre-exist) and as the primitive under [`symlink_exclusive`].
fn symlink(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| Error::io(link, e))
    }
    #[cfg(not(unix))]
    {
        let _ = target;
        Err(Error::config(format!(
            "symlink mode is only supported on Unix; cannot create {}",
            link.display()
        )))
    }
}

/// Create a symlink as an **exclusive create** (T17.4): `symlink(2)` itself fails with `EEXIST` if the
/// name already exists, so there is no `exists()`-then-create (TOCTOU) window — a pre-existing entry is
/// reported as "already exists (use --force)", never silently followed or clobbered. Symmetric with the
/// `hard_link` exclusive publish for regular files.
fn symlink_exclusive(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(target, link) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(Error::config(format!(
                "{} already exists (use --force to overwrite)",
                link.display()
            ))),
            Err(e) => Err(Error::io(link, e)),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = target;
        Err(Error::config(format!(
            "symlink mode is only supported on Unix; cannot create {}",
            link.display()
        )))
    }
}

/// Apply Unix file permission bits to a freshly-written **regular** file (reference `zic`'s `-m`,
/// T9.5). Isolated here so the platform `cfg` lives in one place. Never call this on a symlink entry
/// — `set_permissions` follows the link and would chmod the *target*; callers apply it only to TZif
/// zone files and **copied** link files. On a non-Unix platform this is a config error (callers
/// validate platform support before any write, so this is defence in depth).
pub fn set_file_mode(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|e| Error::io(path, e))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
        Err(Error::config(
            "--mode (file permission bits) is only supported on Unix platforms",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T17.4: durable write + race-free / atomic symlink materialization ──

    #[test]
    fn durable_write_succeeds_and_round_trips() {
        // Exercises the install-grade path (content fsync + atomic publish + parent-dir fsync). On the
        // test filesystem the directory fsync must succeed; the bytes must round-trip.
        let dir = tempfile::tempdir().unwrap();
        let p = write_zone_file(dir.path(), "Zone/D", b"hello", false, true).unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"hello");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_exclusive_create_rejects_preexisting_without_following() {
        // First create is fine; a second (no `--force`) must fail closed with "already exists" — the
        // `symlink(2)` exclusive create (EEXIST), not an `exists()`-then-create race.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_zone_file(root, "B", b"zone-b", false, true).unwrap();
        write_link(root, "A", "B", LinkMode::Symlink, false, true).unwrap();
        let err = write_link(root, "A", "B", LinkMode::Symlink, false, true).unwrap_err();
        assert!(
            err.to_string().contains("already exists"),
            "expected an exclusive-create rejection, got: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_force_overwrite_replaces_atomically() {
        // `--force` must atomically replace an existing symlink (temp-symlink + rename, which operates
        // on the link itself and never follows it) — the link ends up pointing at the new target.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_zone_file(root, "B", b"zone-b", false, true).unwrap();
        write_zone_file(root, "C", b"zone-c", false, true).unwrap();
        write_link(root, "A", "B", LinkMode::Symlink, false, true).unwrap();
        // Overwrite A to point at C.
        write_link(root, "A", "C", LinkMode::Symlink, true, true).unwrap();
        let dest = std::fs::read_link(root.join("A")).unwrap();
        assert_eq!(
            dest,
            PathBuf::from("C"),
            "force-overwrite must retarget the link"
        );
        // Reading through the link yields C's bytes (the link was replaced, not written through).
        assert_eq!(std::fs::read(root.join("A")).unwrap(), b"zone-c");
    }

    #[test]
    fn accepts_normal_names() {
        assert_eq!(
            safe_relative_path("Europe/London").unwrap(),
            PathBuf::from("Europe/London")
        );
        assert_eq!(safe_relative_path("UTC").unwrap(), PathBuf::from("UTC"));
    }

    #[test]
    fn rejects_traversal_and_tricks() {
        for bad in [
            "../etc/passwd",
            "/abs/path",
            "a/../../b",
            "a//b",
            "-flag/x",
            "..",
            ".",
            "",
        ] {
            assert!(safe_relative_path(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn containment_check() {
        let root = Path::new("/out/zone");
        assert!(is_contained(root, Path::new("/out/zone/Europe/London")));
        assert!(!is_contained(root, Path::new("/out/zone/../escape")));
        assert!(!is_contained(root, Path::new("/elsewhere")));
    }
}
