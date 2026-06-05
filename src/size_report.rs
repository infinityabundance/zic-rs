//! `size-report` (T21.2) — a **read-only** footprint report over a produced output tree (`--out <dir>`),
//! for the container / embedded image builder (see `docs/container-embedded-builder.md`).
//!
//! It answers *"how big is this timezone bundle, what is in it, and is it reproducible?"* — it **never
//! compiles, never writes, never admits**. It walks the tree, classifies each entry **structurally** (a
//! regular file that parses as TZif · a symlink alias · any other file), tallies the footprint, and
//! computes a **deterministic `bundle_hash`** (sorted (relative-path, content-hash) pairs → one SHA-256,
//! independent of readdir order).
//!
//! **Honest boundary (non-claims):** size-report reports the **tree as it is on disk**. It does **not**
//! distinguish a real zone from a *copied* link (copy-mode produces a byte-identical TZif file — telling
//! them apart needs the `alias-map.json`, a future cross-reference), and it makes **no** runtime/reader
//! approval claim (the reader-compatibility gauntlet is T21/T22). A symlink is counted as a link; a
//! regular file that parses as TZif is `tzif_files` (zone-or-copied-link); everything else is `other_files`.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::json::escape;
use crate::manifest::CompilerIdentity;

/// The schema id for the JSON form. `v1` — first cut; additive optional fields would not bump it
/// (`docs/schema-compatibility-policy.md`).
pub const SCHEMA: &str = "zic-rs-size-report-v1";

/// Options for [`run_size_report`].
#[derive(Debug)]
pub struct SizeReportOptions {
    /// The output tree to measure (an existing directory produced by `compile --out`).
    pub out: PathBuf,
}

/// One TZif file's footprint, for the largest-file pick.
#[derive(Debug, Clone)]
pub struct TzifEntry {
    pub rel: String,
    pub bytes: u64,
}

/// The size report.
#[derive(Debug)]
pub struct SizeReport {
    pub root: String,
    /// Regular files that parse as TZif (a zone *or* a copy-mode link — not distinguished here).
    pub tzif_files: u64,
    /// Symlink entries (alias links in symlink-mode).
    pub symlink_links: u64,
    /// Regular files that are **not** valid TZif (e.g. `manifest.json`, `alias-map.json`).
    pub other_files: u64,
    /// Total bytes of all regular files (the on-disk bundle size).
    pub total_bytes: u64,
    /// Total bytes of the TZif files only (the timezone payload).
    pub total_tzif_bytes: u64,
    /// The largest TZif file (relative path, bytes), if any.
    pub largest_tzif: Option<TzifEntry>,
    /// Count of TZif files at version v1 / v2 / v3 / v4 (index 0..3).
    pub version_histogram: [u64; 4],
    /// TZif files carrying a non-empty POSIX-TZ footer.
    pub footer_present: u64,
    /// Deterministic bundle hash: SHA-256 over the sorted `relpath\0content-hash` lines of **every**
    /// regular file + `relpath\0symlink:<target>` for each symlink (so link structure is captured).
    /// Same tree → same hash, regardless of traversal order.
    pub bundle_hash: String,
    pub compiler: CompilerIdentity,
}

/// Walk `out` (read-only) and build the report. Errors only if `out` is not a readable directory — the
/// builder asked to measure a tree that is not there (`Error::config`, exit 1); a *present but odd* tree
/// is reported, never an error.
pub fn run_size_report(opts: &SizeReportOptions) -> Result<SizeReport> {
    let root = &opts.out;
    if !root.is_dir() {
        return Err(Error::config(format!(
            "size-report: --out {} is not a readable directory",
            root.display()
        )));
    }

    let mut tzif_files = 0u64;
    let mut symlink_links = 0u64;
    let mut other_files = 0u64;
    let mut total_bytes = 0u64;
    let mut total_tzif_bytes = 0u64;
    let mut largest_tzif: Option<TzifEntry> = None;
    let mut version_histogram = [0u64; 4];
    let mut footer_present = 0u64;
    // (relpath, hash-or-symlink-marker) pairs for **every** entry — sorted before hashing for
    // order-independence. We include non-TZif files (e.g. `manifest.json`) and symlink targets too, so
    // `bundle_hash` witnesses the *entire* tree on disk, not just the timezone payload: any change to any
    // bundled file (or a retargeted alias) changes the hash, which is what a reproducible-image builder needs.
    let mut hash_lines: Vec<String> = Vec::new();

    // Iterative DFS over the output tree (explicit stack, not recursion: bounded stack memory on a deep
    // tree, and no `walkdir` dependency — the core stays lean). Traversal order does not matter: the report
    // tallies are commutative and `hash_lines` is sorted before hashing.
    let mut stack: Vec<PathBuf> = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| {
            Error::config(format!("size-report: cannot read {}: {e}", dir.display()))
        })?;
        for entry in entries {
            let entry =
                entry.map_err(|e| Error::config(format!("size-report: dir entry error: {e}")))?;
            let path = entry.path();
            // `symlink_metadata` does NOT follow the link — so a symlink is classified as a link, never
            // silently followed into its target's bytes.
            let meta = std::fs::symlink_metadata(&path)
                .map_err(|e| Error::config(format!("size-report: stat {}: {e}", path.display())))?;
            let rel = rel_path(root, &path);
            // Classification order is deliberate: **symlink first** (a symlink to a directory must NOT be
            // recursed into, and a symlink to a TZif must NOT be counted as a second zone — it is an alias);
            // then directories (recurse); then regular files (classify as TZif vs other).
            if meta.file_type().is_symlink() {
                symlink_links += 1;
                let target = std::fs::read_link(&path)
                    .map(|t| t.to_string_lossy().into_owned())
                    .unwrap_or_default();
                hash_lines.push(format!("{rel}\0symlink:{target}"));
            } else if meta.is_dir() {
                stack.push(path);
            } else if meta.is_file() {
                let bytes = std::fs::read(&path).map_err(|e| {
                    Error::config(format!("size-report: read {}: {e}", path.display()))
                })?;
                let len = bytes.len() as u64;
                total_bytes += len;
                hash_lines.push(format!("{rel}\0{}", crate::hash::sha256_hex(&bytes)));
                match crate::tzif::validate::parse(&bytes) {
                    Ok(parsed) => {
                        tzif_files += 1;
                        total_tzif_bytes += len;
                        if let Some(idx) = version_index(parsed.version) {
                            version_histogram[idx] += 1;
                        }
                        if !parsed.footer.is_empty() {
                            footer_present += 1;
                        }
                        // MSRV-safe (`Option::is_none_or` is 1.82+; the floor is 1.74).
                        let is_larger = match &largest_tzif {
                            Some(e) => len > e.bytes,
                            None => true,
                        };
                        if is_larger {
                            largest_tzif = Some(TzifEntry {
                                rel: rel.clone(),
                                bytes: len,
                            });
                        }
                    }
                    // A regular file that is not valid TZif (e.g. `manifest.json`, `alias-map.json`, or a
                    // copy-mode link that happens not to parse) — counted + hashed above, but not a zone.
                    Err(_) => other_files += 1,
                }
            }
            // anything else (fifo/socket/device) is ignored — a zoneinfo tree never contains them.
        }
    }

    hash_lines.sort();
    let bundle_hash = crate::hash::sha256_hex(hash_lines.join("\n").as_bytes());

    Ok(SizeReport {
        root: root.to_string_lossy().into_owned(),
        tzif_files,
        symlink_links,
        other_files,
        total_bytes,
        total_tzif_bytes,
        largest_tzif,
        version_histogram,
        footer_present,
        bundle_hash,
        compiler: CompilerIdentity::capture(),
    })
}

/// Map a TZif header version byte to a histogram index: `\0`(NUL)→v1, `'2'`→v2, `'3'`→v3, `'4'`→v4.
fn version_index(version: u8) -> Option<usize> {
    match version {
        0 => Some(0),
        b'2' => Some(1),
        b'3' => Some(2),
        b'4' => Some(3),
        _ => None,
    }
}

/// Path of `path` relative to `root`, with `/` separators (portable, deterministic). Falls back to the
/// full lossy path if `path` is somehow not under `root`.
fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .map(|p| {
            p.components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

impl SizeReport {
    /// Render as deterministic JSON (`zic-rs-size-report-v1`).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", escape(SCHEMA)));
        s.push_str(&crate::manifest::provenance_block_json());
        s.push_str(
            "  \"non_claim\": \"size-report measures the output tree ON DISK (read-only). It does NOT \
             distinguish a zone from a copy-mode link (a copied link is a byte-identical TZif; telling them \
             apart needs alias-map.json), and makes NO runtime/reader approval claim (the reader gauntlet \
             is future). bundle_hash is deterministic over the tree, not a signed attestation.\",\n",
        );
        s.push_str(&format!("  \"root\": {},\n", escape(&self.root)));
        s.push_str(&format!("  \"tzif_files\": {},\n", self.tzif_files));
        s.push_str(&format!("  \"symlink_links\": {},\n", self.symlink_links));
        s.push_str(&format!("  \"other_files\": {},\n", self.other_files));
        s.push_str(&format!("  \"total_bytes\": {},\n", self.total_bytes));
        s.push_str(&format!(
            "  \"total_tzif_bytes\": {},\n",
            self.total_tzif_bytes
        ));
        match &self.largest_tzif {
            Some(e) => s.push_str(&format!(
                "  \"largest_tzif\": {{ \"path\": {}, \"bytes\": {} }},\n",
                escape(&e.rel),
                e.bytes
            )),
            None => s.push_str("  \"largest_tzif\": null,\n"),
        }
        let h = &self.version_histogram;
        s.push_str(&format!(
            "  \"version_histogram\": {{ \"v1\": {}, \"v2\": {}, \"v3\": {}, \"v4\": {} }},\n",
            h[0], h[1], h[2], h[3]
        ));
        s.push_str(&format!("  \"footer_present\": {},\n", self.footer_present));
        s.push_str(&format!(
            "  \"bundle_hash\": {},\n",
            escape(&self.bundle_hash)
        ));
        let c = &self.compiler;
        let opt = |o: Option<&str>| o.map(escape).unwrap_or_else(|| "null".into());
        s.push_str(&format!(
            "  \"compiler_identity\": {{ \"zic_rs_version\": {}, \"rustc\": {}, \"target\": {}, \
             \"profile\": {}, \"git_commit\": {} }}\n",
            escape(c.zic_rs_version),
            opt(c.rustc),
            escape(&c.target),
            escape(c.profile),
            opt(c.git_commit),
        ));
        s.push_str("}\n");
        s
    }

    /// Render as a short human-readable summary.
    pub fn to_text(&self) -> String {
        let h = &self.version_histogram;
        let largest = self
            .largest_tzif
            .as_ref()
            .map(|e| format!("{} ({} bytes)", e.rel, e.bytes))
            .unwrap_or_else(|| "(none)".into());
        format!(
            "size-report for {root}\n\
             TZif files:      {tzif} ({tzif_bytes} bytes)\n\
             symlink links:   {links}\n\
             other files:     {other}\n\
             total on disk:   {total} bytes\n\
             versions:        v1={v1} v2={v2} v3={v3} v4={v4}\n\
             footer present:  {footer}\n\
             largest TZif:    {largest}\n\
             bundle_hash:     {hash}\n",
            root = self.root,
            tzif = self.tzif_files,
            tzif_bytes = self.total_tzif_bytes,
            links = self.symlink_links,
            other = self.other_files,
            total = self.total_bytes,
            v1 = h[0],
            v2 = h[1],
            v3 = h[2],
            v4 = h[3],
            footer = self.footer_present,
            largest = largest,
            hash = self.bundle_hash,
        )
    }
}
