//! `doctor` (T16.6b) — a read-only environment probe for operators/packagers.
//!
//! Answers "is this environment fit to *verify* (and optionally cross-check) zic-rs output?" — like
//! `brew doctor`. It **never compiles, never writes, never admits**: it inspects the host for the
//! reference `zic`/`zdump` tools and an optional `tzdata.zi`, and reports the build identity. Every
//! probe degrades to an explicit `absent`/`not_found` (never a silent gap), and `doctor` **always exits
//! 0** — it is a diagnosis, not a gate.
//!
//! Non-claims: `doctor` reads the host env **for diagnosis only**; it admits/validates nothing, and
//! **tool presence ≠ tool correctness ≠ an admitted reference** (admission stays the T16.3
//! versioned-archive + integrity-pin rule). The production `compile` path requires none of these tools;
//! only the conformance/`compare` path does (see `docs/platform-portability.md`).
//!
//! **T17.3 reliability hardening (schema `v1 → v2`):** a resolved tool's `--version` outcome is the typed
//! [`ToolVersionStatus`] (a non-zero exit is `Unsupported` — old forks — never a fake version) and its
//! hash is the typed [`HashReadStatus`] (an unreadable file is a typed status, never the literal
//! `"unreadable"` inside a `sha256` field).

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::json::escape;
use crate::manifest::CompilerIdentity;

/// The outcome of asking a resolved tool for its `--version` (T17.3 — was a bare `Option<String>` that
/// silently turned an old fork's "unknown option" or a non-zero exit into a "version" string). The tool
/// *ran* (or didn't) is now distinguished from the tool *has a usable version*: a `--version` that the
/// tool rejected (old forks like OpenBSD/DragonFly `zic`) is `Unsupported`, never a fake version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolVersionStatus {
    /// `--version` exited 0 and printed a usable first line.
    Supported { version: String },
    /// `--version` exited **non-zero** — the tool does not support the flag (typical of old `zic` forks,
    /// which print usage to stderr and exit 1). The captured first line + exit status are recorded, but
    /// it is **not** treated as a version.
    Unsupported {
        first_line: Option<String>,
        exit_status: Option<i32>,
    },
    /// `--version` ran but neither cleanly succeeded with a version nor cleanly rejected the flag (e.g.
    /// exited 0 with no usable output, or was terminated by a signal — `exit_status: None`).
    CommandFailed {
        first_line: Option<String>,
        exit_status: Option<i32>,
    },
    /// The `--version` command could not be spawned at all (resolved file not executable, race, …).
    NotRun { reason: String },
}

impl ToolVersionStatus {
    fn to_json(&self) -> String {
        let optline = |o: &Option<String>| {
            o.as_ref()
                .map(|s| escape(s))
                .unwrap_or_else(|| "null".into())
        };
        let optcode = |o: &Option<i32>| o.map(|c| c.to_string()).unwrap_or_else(|| "null".into());
        match self {
            ToolVersionStatus::Supported { version } => {
                format!(
                    "{{ \"status\": \"supported\", \"version\": {} }}",
                    escape(version)
                )
            }
            ToolVersionStatus::Unsupported {
                first_line,
                exit_status,
            } => format!(
                "{{ \"status\": \"unsupported\", \"first_line\": {}, \"exit_status\": {} }}",
                optline(first_line),
                optcode(exit_status)
            ),
            ToolVersionStatus::CommandFailed {
                first_line,
                exit_status,
            } => format!(
                "{{ \"status\": \"command_failed\", \"first_line\": {}, \"exit_status\": {} }}",
                optline(first_line),
                optcode(exit_status)
            ),
            ToolVersionStatus::NotRun { reason } => {
                format!(
                    "{{ \"status\": \"not_run\", \"reason\": {} }}",
                    escape(reason)
                )
            }
        }
    }

    /// Short human label for the text report.
    fn label(&self) -> String {
        match self {
            ToolVersionStatus::Supported { version } => version.clone(),
            ToolVersionStatus::Unsupported { exit_status, .. } => {
                format!(
                    "(--version unsupported, exit {})",
                    exit_status.unwrap_or(-1)
                )
            }
            ToolVersionStatus::CommandFailed { .. } => "(--version failed)".into(),
            ToolVersionStatus::NotRun { .. } => "(--version not run)".into(),
        }
    }
}

/// The outcome of reading a resolved tool's bytes for hashing (T17.3 — replaces the old habit of putting
/// the non-hash literal `"unreadable"` into a field named `sha256`). The field is now honest by type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashReadStatus {
    /// The bytes were read and hashed.
    Read { sha256: String },
    /// The resolved file exists but could not be read (permissions, race, …).
    Unreadable { reason: String },
    /// No hash was attempted (not reached in the current flow — a resolved tool is always hashed — but
    /// kept so a future caller that defers hashing has an honest value rather than a fake hash).
    NotAttempted { reason: String },
}

impl HashReadStatus {
    fn to_json(&self) -> String {
        match self {
            HashReadStatus::Read { sha256 } => {
                format!("{{ \"status\": \"read\", \"sha256\": {} }}", escape(sha256))
            }
            HashReadStatus::Unreadable { reason } => {
                format!(
                    "{{ \"status\": \"unreadable\", \"reason\": {} }}",
                    escape(reason)
                )
            }
            HashReadStatus::NotAttempted { reason } => {
                format!(
                    "{{ \"status\": \"not_attempted\", \"reason\": {} }}",
                    escape(reason)
                )
            }
        }
    }

    fn label(&self) -> String {
        match self {
            HashReadStatus::Read { sha256 } => {
                // first 12 hex chars, like the lab's evidence shorthand
                sha256.chars().take(12).collect::<String>()
            }
            HashReadStatus::Unreadable { .. } => "(unreadable)".into(),
            HashReadStatus::NotAttempted { .. } => "(hash not attempted)".into(),
        }
    }
}

/// The presence/identity of a reference tool on the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    /// The tool was resolved on the host. Its `--version` outcome and hash-read outcome are **typed**
    /// (T17.3): a rejected `--version` is `Unsupported`, never a fake version; an unreadable file is a
    /// typed `HashReadStatus`, never the literal `"unreadable"` in a `sha256` field.
    Present {
        path: String,
        version_status: ToolVersionStatus,
        hash_status: HashReadStatus,
    },
    /// The tool could not be resolved (not on `PATH`, or the given path does not exist).
    Absent,
}

impl ToolStatus {
    fn to_json(&self) -> String {
        match self {
            ToolStatus::Present {
                path,
                version_status,
                hash_status,
            } => {
                format!(
                    "{{ \"status\": \"present\", \"path\": {}, \"version_status\": {}, \"hash_status\": {} }}",
                    escape(path),
                    version_status.to_json(),
                    hash_status.to_json()
                )
            }
            ToolStatus::Absent => "{ \"status\": \"absent\" }".to_string(),
        }
    }

    fn label(&self) -> String {
        match self {
            ToolStatus::Present {
                path,
                version_status,
                hash_status,
            } => {
                format!(
                    "present — {path} [{}] ({})",
                    version_status.label(),
                    hash_status.label()
                )
            }
            ToolStatus::Absent => "absent".into(),
        }
    }
}

/// The installed-`tzdata.zi` probe (explicit path only — `doctor` never reads the host tree implicitly).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TzdataStatus {
    NotProbed,
    NotFound(String),
    Found {
        path: String,
        detected_version: Option<String>,
        sha256: String,
    },
}

/// Options for the doctor probe.
#[derive(Debug, Clone)]
pub struct DoctorOptions {
    pub reference_zic: String,
    pub reference_zdump: String,
    pub tzdata: Option<PathBuf>,
}

/// The doctor report.
#[derive(Debug)]
pub struct DoctorReport {
    pub reference_zic: ToolStatus,
    pub reference_zdump: ToolStatus,
    pub tzdata: TzdataStatus,
    pub compiler: CompilerIdentity,
}

/// The schema id. **Bumped `v1 → v2` at T17.3** (intentional, recorded): the `Present` tool object
/// replaced the free `version: string|null` + `sha256: string` fields with the typed `version_status`
/// and `hash_status` objects (so a rejected `--version` is `unsupported`, not a fake version, and an
/// unreadable file is a typed status, not the literal `"unreadable"` in a `sha256` field). `doctor-v1`
/// shipped in T16.6b earlier this cycle and has no external consumers; the bump is the honest record of
/// the shape change rather than an in-place mutation of a frozen id.
pub const SCHEMA: &str = "zic-rs-doctor-v2";

/// Resolve a program name to a file path: an explicit path is taken as-is; a bare name is searched on
/// `$PATH`. Returns `None` if nothing exists (→ `Absent`). `pub(crate)` so `release_diff` can reuse the
/// one resolver to decide global-tool-unavailable vs per-identifier oracle failure (T17.3).
pub(crate) fn resolve(program: &str) -> Option<PathBuf> {
    let p = Path::new(program);
    if p.is_absolute() || p.components().count() > 1 {
        return if p.is_file() {
            Some(p.to_path_buf())
        } else {
            None
        };
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(program);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Probe one reference tool (read-only): resolve it, hash it (typed [`HashReadStatus`]), and classify
/// its `--version` outcome (typed [`ToolVersionStatus`]). T17.3: a non-zero `--version` exit is
/// `Unsupported` (old forks), an unreadable file is a typed hash status — never a fake version or a
/// non-hash string in a `sha256` field.
fn probe_tool(program: &str) -> ToolStatus {
    let Some(path) = resolve(program) else {
        return ToolStatus::Absent;
    };
    let hash_status = match std::fs::read(&path) {
        Ok(b) => HashReadStatus::Read {
            sha256: crate::hash::sha256_hex(&b),
        },
        Err(e) => HashReadStatus::Unreadable {
            reason: e.to_string(),
        },
    };
    let version_status = match std::process::Command::new(&path).arg("--version").output() {
        Err(e) => ToolVersionStatus::NotRun {
            reason: e.to_string(),
        },
        Ok(o) => {
            let pick = if o.stdout.is_empty() {
                &o.stderr
            } else {
                &o.stdout
            };
            let first_line = String::from_utf8_lossy(pick)
                .lines()
                .next()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty());
            match o.status.code() {
                // exited cleanly with a usable line → a real version.
                Some(0) if first_line.is_some() => ToolVersionStatus::Supported {
                    version: first_line.unwrap(),
                },
                // exited cleanly but produced nothing usable — ran, but no version.
                Some(0) => ToolVersionStatus::CommandFailed {
                    first_line,
                    exit_status: Some(0),
                },
                // exited non-zero → the tool rejected `--version` (old fork). NOT a version.
                Some(code) => ToolVersionStatus::Unsupported {
                    first_line,
                    exit_status: Some(code),
                },
                // terminated by a signal (no exit code).
                None => ToolVersionStatus::CommandFailed {
                    first_line,
                    exit_status: None,
                },
            }
        }
    };
    ToolStatus::Present {
        path: path.to_string_lossy().into_owned(),
        version_status,
        hash_status,
    }
}

/// Run the read-only environment probe. Never fails (a diagnosis, not a gate).
pub fn run_doctor(opts: &DoctorOptions) -> Result<DoctorReport> {
    let tzdata = match &opts.tzdata {
        None => TzdataStatus::NotProbed,
        Some(p) => match std::fs::read(p) {
            Ok(bytes) => TzdataStatus::Found {
                path: p.to_string_lossy().into_owned(),
                detected_version: crate::report::sniff_tzdb_version(&bytes),
                sha256: crate::hash::sha256_hex(&bytes),
            },
            Err(e) => TzdataStatus::NotFound(format!("{}: {e}", p.display())),
        },
    };
    Ok(DoctorReport {
        reference_zic: probe_tool(&opts.reference_zic),
        reference_zdump: probe_tool(&opts.reference_zdump),
        tzdata,
        compiler: CompilerIdentity::capture(),
    })
}

impl DoctorReport {
    /// Render as deterministic JSON (`zic-rs-doctor-v2`).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", escape(SCHEMA)));
        s.push_str(&crate::manifest::provenance_block_json());
        s.push_str(
            "  \"non_claim\": \"doctor reads the host environment for DIAGNOSIS ONLY; it admits/validates \
             nothing. Tool presence ≠ tool correctness ≠ an admitted reference (admission stays the \
             versioned-archive + integrity-pin rule). The production compile path needs none of these tools.\",\n",
        );
        s.push_str(&format!(
            "  \"reference_zic\": {},\n",
            self.reference_zic.to_json()
        ));
        s.push_str(&format!(
            "  \"reference_zdump\": {},\n",
            self.reference_zdump.to_json()
        ));
        s.push_str(&format!("  \"tzdata\": {},\n", self.tzdata_json()));
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

    fn tzdata_json(&self) -> String {
        match &self.tzdata {
            TzdataStatus::NotProbed => "{ \"status\": \"not_probed\" }".into(),
            TzdataStatus::NotFound(reason) => {
                format!(
                    "{{ \"status\": \"not_found\", \"reason\": {} }}",
                    escape(reason)
                )
            }
            TzdataStatus::Found {
                path,
                detected_version,
                sha256,
            } => {
                let v = detected_version
                    .as_ref()
                    .map(|s| escape(s))
                    .unwrap_or_else(|| "null".into());
                format!(
                    "{{ \"status\": \"found\", \"path\": {}, \"detected_version\": {}, \"sha256\": {} }}",
                    escape(path),
                    v,
                    escape(sha256)
                )
            }
        }
    }

    /// Render a short human-readable summary.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        s.push_str(
            "zic-rs doctor (read-only environment probe — diagnosis only, admits nothing)\n",
        );
        s.push_str(&format!(
            "  reference zic   : {}\n",
            self.reference_zic.label()
        ));
        s.push_str(&format!(
            "  reference zdump : {}\n",
            self.reference_zdump.label()
        ));
        let tz = match &self.tzdata {
            TzdataStatus::NotProbed => "not probed (pass --tzdata <path>)".to_string(),
            TzdataStatus::NotFound(r) => format!("not found ({r})"),
            TzdataStatus::Found {
                path,
                detected_version,
                ..
            } => format!(
                "{path} (version {})",
                detected_version.as_deref().unwrap_or("unknown")
            ),
        };
        s.push_str(&format!("  tzdata.zi       : {tz}\n"));
        s.push_str(&format!(
            "  zic-rs          : {} ({} / {})\n",
            self.compiler.zic_rs_version, self.compiler.target, self.compiler.profile
        ));
        s
    }
}
