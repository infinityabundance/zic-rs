//! `support-report` — an honest **frontier map** of which identifiers in a tzdata source file
//! zic-rs can compile, and *why* the rest cannot.
//!
//! This turns "a partial compiler" into a transparent, machine-generated inventory: load a whole
//! source file (the installed `/usr/share/zoneinfo/tzdata.zi` is the headline target — zic-rs
//! parses the zishrink form directly), attempt to compile **every** canonical zone, and bucket
//! the outcome. Links are accounted separately (production systems use aliases, not just
//! canonical zones — a current-offset table does not describe a zone's historical data).
//!
//! ## What "supported" means here (no overclaiming)
//!
//! A zone is **compile-supported** if [`compile_zone`] returns `Ok` — i.e.
//! zic-rs produces a valid TZif file for it. That is *not* a claim of behavioural correctness:
//! the binding correctness contract is reference `zic`/`zdump` (see `compare`), which this report
//! does not run. So the report answers "can I compile it?", not "is every transition right?".
//! Every zone lands in exactly one bucket (supported, or a named unsupported reason); the
//! accounting is exact (`supported + Σ unsupported == zones parsed`) and a catch-all `other`
//! bucket keeps the raw diagnostic so nothing is ever silently dropped.

use std::collections::BTreeMap;

use crate::diagnostics::DiagnosticCode;
use crate::json::escape;
use crate::model::Database;
use crate::{compile_zone, resolve_link_target, Error};

/// Schema identifier for the JSON form.
const SCHEMA: &str = "zic-rs-support-report-v4";

/// How many example identifiers to show per bucket in the text report (the full list is always
/// present in the JSON form — this is a display affordance, not a silent cap; the elision is
/// shown explicitly as `(+N more)`).
const TEXT_EXAMPLES: usize = 6;

/// One unsupported bucket: the zones that hit it, plus a representative raw diagnostic message
/// (so the `other` bucket — and any bucket — can be inspected without re-running).
#[derive(Debug, Default)]
pub struct Bucket {
    pub zones: Vec<String>,
    pub example_message: String,
}

/// How `Link` identifiers resolve, accounted separately from canonical zones.
#[derive(Debug, Default)]
pub struct LinkAccounting {
    /// Links whose canonical zone is compile-supported.
    pub to_supported: Vec<String>,
    /// Links that resolve, but to a canonical zone that is *not* compile-supported.
    pub to_unsupported: Vec<String>,
    /// Links whose chain forms a cycle (e.g. `B -> A -> B`).
    pub cycles: Vec<String>,
    /// Links whose target names neither a zone nor another link.
    pub missing: Vec<String>,
}

/// The complete frontier map for one source file.
#[derive(Debug)]
pub struct SupportReport {
    /// tzdb release as declared by a leading `# version …` comment in the source, if any.
    pub tzdb_version: Option<String>,
    pub zones_parsed: usize,
    pub links_parsed: usize,
    /// Canonical zones that compile, sorted.
    pub supported_zones: Vec<String>,
    /// Unsupported canonical zones, keyed by a stable bucket label (sorted).
    pub unsupported: BTreeMap<String, Bucket>,
    pub links: LinkAccounting,
}

/// Build the frontier map for `db`. `tzdb_version` is the best-effort release string the caller
/// sniffed from the source header (`None` if absent).
pub fn build_support_report(db: &Database, tzdb_version: Option<String>) -> SupportReport {
    let mut supported_zones: Vec<String> = Vec::new();
    let mut unsupported: BTreeMap<String, Bucket> = BTreeMap::new();

    for zone in &db.zones {
        match compile_zone(db, &zone.name) {
            Ok(_) => supported_zones.push(zone.name.clone()),
            Err(e) => {
                let (label, message) = classify(&e);
                let bucket = unsupported.entry(label).or_default();
                bucket.zones.push(zone.name.clone());
                if bucket.example_message.is_empty() {
                    bucket.example_message = message;
                }
            }
        }
    }
    supported_zones.sort();
    for b in unsupported.values_mut() {
        b.zones.sort();
    }

    // Links resolve independently of *compile* support; classify each.
    let supported_set: std::collections::BTreeSet<&str> =
        supported_zones.iter().map(String::as_str).collect();
    let mut links = LinkAccounting::default();
    for link in &db.links {
        match resolve_link_target(db, &link.link_name) {
            Ok(canonical) => {
                if supported_set.contains(canonical) {
                    links.to_supported.push(link.link_name.clone());
                } else {
                    links.to_unsupported.push(link.link_name.clone());
                }
            }
            Err(e) => {
                // `resolve_link_target` distinguishes cycle vs missing by message.
                if e.to_string().contains("cycle") {
                    links.cycles.push(link.link_name.clone());
                } else {
                    links.missing.push(link.link_name.clone());
                }
            }
        }
    }
    links.to_supported.sort();
    links.to_unsupported.sort();
    links.cycles.sort();
    links.missing.sort();

    SupportReport {
        tzdb_version,
        zones_parsed: db.zones.len(),
        links_parsed: db.links.len(),
        supported_zones,
        unsupported,
        links,
    }
}

/// Map a compile error to a `(bucket_label, raw_message)`. The label is stable: it keys on the
/// `ZIC0xx` diagnostic code, and for the broad `ZIC001_UNSUPPORTED_DIRECTIVE` it sub-classifies
/// into a small fixed reason set matched from the message. Anything unrecognised falls into an
/// explicit `other` reason **and** carries its raw message — we never silently drop a failure.
fn classify(e: &Error) -> (String, String) {
    let Some(diag) = e.diagnostic() else {
        // Non-diagnostic errors (I/O, config, internal) — keep the whole message as the label
        // tail so they are visible and never silently merged with real unsupported constructs.
        let m = e.to_string();
        return (format!("error: {m}"), m);
    };
    let code = diag.code.as_str();
    let msg = diag.message.clone();
    if diag.code == DiagnosticCode::UnsupportedDirective {
        let m = &diag.message;
        let reason = if m.contains("negative inline SAVE") {
            "inline-save: negative SAVE"
        } else if m.contains("inline-save FORMAT") {
            "inline-save: %s or STD/DST slash FORMAT"
        } else if m.contains("rule context") {
            "no-rules era: %s or STD/DST slash FORMAT"
        } else if m.contains("not POSIX-expressible") {
            "recurring footer: non-POSIX day form"
        } else if m.contains("unknown rule set") {
            "unknown rule set"
        } else {
            "other (see message)"
        };
        (format!("{code}: {reason}"), msg)
    } else {
        // Other codes are already specific enough to be their own bucket (e.g. ZIC009
        // too-many-transitions, ZIC010 leap seconds).
        (code.to_string(), msg)
    }
}

/// Map an unsupported/fail-closed **bucket label** to the deep `zic` semantic law it represents
/// (the audit map in [`docs/zic-deep-semantics.md`](../docs/zic-deep-semantics.md)). This is the
/// *single source of truth* shared by `--explain-buckets` and that doc, so they cannot drift. Match
/// on the normalized reason substring (robust to the `ZIC0xx_…:` code prefix); unknown buckets
/// return `None` (honest — we do not invent a law for a reason we have not pinned).
pub fn deep_semantic(label: &str) -> Option<&'static str> {
    if label.contains("negative SAVE") {
        Some(
            "law 7 — SAVE is signed state (negative SAVE is valid; Ireland). Implement as \
              first-class signed SAVE, not a per-zone exception.",
        )
    } else if label.contains("non-POSIX day form") {
        Some(
            "law 10 — ON day forms can leave the nominal month (e.g. `Sun>=31`); a recurring such \
              form is not POSIX-footer-expressible, so an exact footer cannot be synthesised.",
        )
    } else if label.contains("STD/DST slash") || label.contains("%s or STD/DST") {
        Some("law 9 — `%s`, `%z`, and `STD/DST` slash are three distinct FORMAT paths; the slash/`%s` \
              forms on this era are not yet pinned against reference `zic`.")
    } else {
        None
    }
}

impl SupportReport {
    /// Total identifiers (canonical zones + links) in the source.
    pub fn identifiers(&self) -> usize {
        self.zones_parsed + self.links_parsed
    }

    /// Identifiers that ultimately reach a compile-supported zone (supported zones + links to
    /// supported zones).
    pub fn supported_identifiers(&self) -> usize {
        self.supported_zones.len() + self.links.to_supported.len()
    }

    /// The number of unsupported zones accounted across all buckets.
    pub fn unsupported_zone_count(&self) -> usize {
        self.unsupported.values().map(|b| b.zones.len()).sum()
    }

    /// Accounting invariant: every parsed zone is in exactly one place.
    pub fn is_fully_accounted(&self) -> bool {
        self.supported_zones.len() + self.unsupported_zone_count() == self.zones_parsed
    }

    /// The largest unsupported bucket — the "biggest unlock" pointer — as `(label, count)`.
    pub fn largest_bucket(&self) -> Option<(&str, usize)> {
        self.unsupported
            .iter()
            .map(|(k, b)| (k.as_str(), b.zones.len()))
            .max_by_key(|(_, n)| *n)
    }

    /// Deterministic human-readable report.
    pub fn to_text(&self) -> String {
        self.render_text(false)
    }

    /// As [`Self::to_text`], plus a `↳ deep law:` line under each unsupported bucket mapping it to
    /// the `zic` semantic it represents (the [`deep_semantic`] audit map — see
    /// `docs/zic-deep-semantics.md`). Used by `support-report --explain-buckets`.
    pub fn to_text_explained(&self) -> String {
        self.render_text(true)
    }

    fn render_text(&self, explain: bool) -> String {
        let mut s = String::new();
        let version = self.tzdb_version.as_deref().unwrap_or("unknown");
        s.push_str(&format!(
            "zic-rs support report — tzdb release: {version}\n"
        ));
        s.push_str(
            "(reports COMPILE support — a valid TZif is produced; behavioural correctness is a\n\
             separate question answered by the reference `zic`/`zdump` oracle, not this report.)\n\n",
        );
        s.push_str(&format!("identifiers:      {}\n", self.identifiers()));
        s.push_str(&format!(
            "  canonical zones:  {} parsed, {} compile-supported\n",
            self.zones_parsed,
            self.supported_zones.len()
        ));
        s.push_str(&format!(
            "  links:            {} parsed ({} → supported, {} → unsupported, {} cycle, {} missing)\n",
            self.links_parsed,
            self.links.to_supported.len(),
            self.links.to_unsupported.len(),
            self.links.cycles.len(),
            self.links.missing.len(),
        ));
        s.push_str(&format!(
            "  total supported:  {} / {} identifiers\n\n",
            self.supported_identifiers(),
            self.identifiers()
        ));

        if self.unsupported.is_empty() {
            s.push_str("unsupported zones: none\n");
        } else {
            s.push_str(&format!(
                "unsupported zones ({} across {} buckets):\n",
                self.unsupported_zone_count(),
                self.unsupported.len()
            ));
            for (label, bucket) in &self.unsupported {
                s.push_str(&format!("  [{}] {}\n", bucket.zones.len(), label));
                if explain {
                    match deep_semantic(label) {
                        Some(law) => s.push_str(&format!("      ↳ deep law: {law}\n")),
                        None => s.push_str("      ↳ deep law: (not yet mapped)\n"),
                    }
                }
                let shown = bucket.zones.len().min(TEXT_EXAMPLES);
                for z in &bucket.zones[..shown] {
                    s.push_str(&format!("      {z}\n"));
                }
                if bucket.zones.len() > shown {
                    s.push_str(&format!("      (+{} more)\n", bucket.zones.len() - shown));
                }
            }
            if let Some((label, n)) = self.largest_bucket() {
                s.push_str(&format!(
                    "\nbiggest unlock: the `{label}` bucket ({n} zones) — addressing it admits the most zones.\n"
                ));
            }
        }
        // Accounting check, surfaced (honesty: prove nothing was dropped).
        s.push_str(&format!(
            "\naccounting: {} supported + {} unsupported == {} zones parsed [{}]\n",
            self.supported_zones.len(),
            self.unsupported_zone_count(),
            self.zones_parsed,
            if self.is_fully_accounted() {
                "OK"
            } else {
                "MISMATCH"
            },
        ));
        // T12.6 — the static provenance/capability statement (manifest schema + source-variant
        // reference-pin gate), so an operator sees the trust boundary without reading the manifest.
        s.push_str(&crate::manifest::provenance_block_text());
        s
    }

    /// Deterministic JSON (hand-rolled, shared escaper — no serde). Full lists, no elision.
    pub fn to_json(&self) -> String {
        let arr = |names: &[String]| -> String {
            let items: Vec<String> = names.iter().map(|n| escape(n)).collect();
            format!("[{}]", items.join(", "))
        };
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", escape(SCHEMA)));
        // T12.6 — static provenance/capability block (schema + source-variant pin-gate state +
        // T15.2 `negative_capabilities`).
        s.push_str(&crate::manifest::provenance_block_json());
        // T15.2 — oracle availability, typed. `support-report` is compile-coverage: it consults **no**
        // oracle, so this is honestly `not_run` (it makes no behaviour-verified claim). Visible, never
        // silent — see `docs/zic-conformance-engine.md`.
        s.push_str(&format!(
            "  \"oracle_mode\": {},\n",
            crate::manifest::OracleMode::NotRun.to_json_field()
        ));
        // T15.5 — the one-line conformance rollup: the claim *envelope* (report kind · bounded level ·
        // declared_scope_hash · compiler/workspace/report provenance · available proof surfaces), so a
        // reviewer reads the scope in one scan. A *report is itself a claim surface*, hence its provenance.
        s.push_str(&crate::manifest::ConformanceStatus::support().to_json_block());
        match &self.tzdb_version {
            Some(v) => s.push_str(&format!("  \"tzdb_version\": {},\n", escape(v))),
            None => s.push_str("  \"tzdb_version\": null,\n"),
        }
        s.push_str(&format!("  \"zones_parsed\": {},\n", self.zones_parsed));
        s.push_str(&format!("  \"links_parsed\": {},\n", self.links_parsed));
        s.push_str(&format!(
            "  \"supported_identifiers\": {},\n",
            self.supported_identifiers()
        ));
        s.push_str(&format!(
            "  \"fully_accounted\": {},\n",
            self.is_fully_accounted()
        ));
        s.push_str(&format!(
            "  \"supported_zones\": {},\n",
            arr(&self.supported_zones)
        ));
        s.push_str("  \"unsupported\": {");
        let mut first = true;
        for (label, bucket) in &self.unsupported {
            s.push_str(if first { "\n" } else { ",\n" });
            first = false;
            let law = match deep_semantic(label) {
                Some(l) => escape(l),
                None => "null".to_string(),
            };
            s.push_str(&format!(
                "    {}: {{ \"count\": {}, \"deep_semantic\": {}, \"example_message\": {}, \"zones\": {} }}",
                escape(label),
                bucket.zones.len(),
                law,
                escape(&bucket.example_message),
                arr(&bucket.zones),
            ));
        }
        s.push_str(if self.unsupported.is_empty() {
            "},\n"
        } else {
            "\n  },\n"
        });
        s.push_str("  \"links\": {\n");
        s.push_str(&format!(
            "    \"to_supported\": {},\n",
            arr(&self.links.to_supported)
        ));
        s.push_str(&format!(
            "    \"to_unsupported\": {},\n",
            arr(&self.links.to_unsupported)
        ));
        s.push_str(&format!("    \"cycles\": {},\n", arr(&self.links.cycles)));
        s.push_str(&format!("    \"missing\": {}\n", arr(&self.links.missing)));
        s.push_str("  }\n");
        s.push_str("}\n");
        s
    }
}

/// Best-effort tzdb release sniff: the first `# version <X>` comment line in `bytes` (the form
/// `zic`'s single-file output and the IANA `version` file use). Returns the token after
/// `version`, e.g. `2026b`. Comments are stripped by the lexer, so we read the raw source here.
pub fn sniff_tzdb_version(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    for line in text.lines().take(40) {
        let l = line.trim_start();
        if let Some(rest) = l.strip_prefix('#') {
            let rest = rest.trim_start();
            if let Some(v) = rest.strip_prefix("version ") {
                let token = v.split_whitespace().next()?;
                return Some(token.to_string());
            }
        }
    }
    None
}
