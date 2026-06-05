//! T16.4 — **auxiliary-table validator** (`zone.tab` / `zone1970.tab` / `zonenow.tab` / `iso3166.tab`)
//! plus a **bounded** install-ecology status.
//!
//! **The central category law (the reusable T12.5c lesson):** these `.tab` files are **policy / index /
//! reference** artifacts — they are **not compile inputs** and **not semantic-output witnesses**. This
//! validator checks only **table structural admissibility**:
//!
//! > A valid zone-table row proves the row is *structurally well-formed*. It does **not** prove the named
//! > zone was compiled, semantically witnessed, historically equivalent, or present in an installed tree.
//!
//! Two further doctrine lines (T16.4):
//! - *Auxiliary tables are policy/index artifacts with **table-specific** invariants; they are not compile
//!   inputs, not semantic witnesses, and **not one-row-per-country maps*** — a country legitimately spans
//!   many rows (the bug this validator caught: `US` ≈ 29 rows in `zone.tab`).
//! - *Auxiliary-table validation is **release-ecology evidence, not reference-oracle identity*** — it is a
//!   separate `zic-rs-aux-table-validation-v1` report surface, never emitted as `oracle_identity` evidence.
//! - **A validator must always name the universe it validates against:** this one resolves **no** zone
//!   names (structural-only), so "unknown zone name" is deliberately not evaluated (see [`ZoneUniverse`]).
//!
//! It is bounds-safe (malformed bytes → typed findings, never a panic) and reads no files itself — the
//! caller supplies bytes, exactly like the rest of the no-host library core.

use crate::json::escape;
use crate::manifest::ArtifactCategory;
use std::collections::BTreeSet;

/// Which auxiliary table this is (T16.4). A **finite** vocabulary — each table has a *different* column
/// shape and a *different* evidence role; they are never conflated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneTableKind {
    /// Legacy country→zone index (`CC \t coord \t TZ [\t comment]`). A country may appear in **many**
    /// rows (one per represented zone — e.g. `US` ≈ 29); it is **not** a one-row-per-country map.
    ZoneTab,
    /// Post-1970 country/location table (`CC[,CC…] \t coord \t TZ [\t comment]`).
    Zone1970Tab,
    /// **Now/future-agreement** table (`CC \t coord \t TZ [\t comment]`; `XX` allowed) — *not* all-history.
    ZonenowTab,
    /// ISO-3166 country-code reference (`CC \t name`) — a code reference, not a zone table.
    Iso3166Tab,
}

impl ZoneTableKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ZoneTableKind::ZoneTab => "zone_tab",
            ZoneTableKind::Zone1970Tab => "zone1970_tab",
            ZoneTableKind::ZonenowTab => "zonenow_tab",
            ZoneTableKind::Iso3166Tab => "iso3166_tab",
        }
    }

    /// The evidence category — **never `compile_input`**. Zone tables are generation/selection *policy*;
    /// `iso3166.tab` is a *reference* code table. (The typed form of the `zone.tab`-is-not-compile law.)
    pub fn artifact_category(self) -> ArtifactCategory {
        match self {
            ZoneTableKind::Iso3166Tab => ArtifactCategory::ReferenceInput,
            _ => ArtifactCategory::PolicyInput,
        }
    }

    /// What the table *covers* — so `zonenow.tab` is never overread as all-history equivalence.
    pub fn coverage(self) -> &'static str {
        match self {
            ZoneTableKind::ZoneTab => "country_zone_index_all_eras",
            ZoneTableKind::Zone1970Tab => "post_1970_country_location",
            ZoneTableKind::ZonenowTab => "now_future_agreement_only",
            ZoneTableKind::Iso3166Tab => "country_code_reference",
        }
    }
}

/// A single typed structural finding (T16.4) — wording-independent, like the diagnostic classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneTableFinding {
    NonUtf8,
    EmptyTable,
    InvalidColumnCount,
    InvalidCountryCode,
    InvalidCoordinateFormat,
    /// The zone-name (zone tables) or country-name (`iso3166.tab`) field is empty.
    EmptyNameField,
    /// A **semantic** duplicate: the same identity tuple repeats — `(country-code-set, coordinates,
    /// zone-name)` for zone tables, or the country *code* for `iso3166.tab`. **Comments are excluded**
    /// from row identity. NOT a duplicate country *code* across rows — a country legitimately spans many
    /// rows in `zone.tab`/`zone1970.tab` (e.g. `US` ≈ 29); only an identical identity tuple is suspect.
    DuplicateSemanticRow,
    /// A country code repeated *within a single* `zone1970.tab` comma-list (set-semantics, not string).
    DuplicateCodeInRow,
}

impl ZoneTableFinding {
    pub fn as_str(self) -> &'static str {
        match self {
            ZoneTableFinding::NonUtf8 => "non_utf8",
            ZoneTableFinding::EmptyTable => "empty_table",
            ZoneTableFinding::InvalidColumnCount => "invalid_column_count",
            ZoneTableFinding::InvalidCountryCode => "invalid_country_code",
            ZoneTableFinding::InvalidCoordinateFormat => "invalid_coordinate_format",
            ZoneTableFinding::EmptyNameField => "empty_name_field",
            ZoneTableFinding::DuplicateSemanticRow => "duplicate_semantic_row",
            ZoneTableFinding::DuplicateCodeInRow => "duplicate_code_in_row",
        }
    }
}

/// **The universe a table validator resolves *names* against** (T16.4 — the central rule: *a
/// validator must always name the universe it validates against*). zic-rs's aux-table validator checks
/// **row structure only** and does **not** resolve zone names against any universe — so "unknown zone
/// name" is deliberately *not evaluated* (it would be meaningless without a declared universe). Resolving
/// names against admitted-source / compiled-output / reference-distribution is tracked, not done here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneUniverse {
    /// Names are **not** resolved — structural row admissibility only (the current honest state).
    NotResolvedStructuralOnly,
    AdmittedSourceDefinitions,
    CompiledOutputTree,
    ReferenceDistributionTable,
    SourcePlusBackwardLinks,
    Unknown,
}

impl ZoneUniverse {
    pub fn as_str(self) -> &'static str {
        match self {
            ZoneUniverse::NotResolvedStructuralOnly => "not_resolved_structural_only",
            ZoneUniverse::AdmittedSourceDefinitions => "admitted_source_definitions",
            ZoneUniverse::CompiledOutputTree => "compiled_output_tree",
            ZoneUniverse::ReferenceDistributionTable => "reference_distribution_table",
            ZoneUniverse::SourcePlusBackwardLinks => "source_plus_backward_links",
            ZoneUniverse::Unknown => "unknown",
        }
    }
}

/// **Which authority `zone1970.tab` country codes are cross-validated against** (T16.4). The safe choice
/// is the **same admitted release's** `iso3166.tab` — never a host/system ISO list (which drifts across
/// releases). `NotCrossValidated` when no `iso3166.tab` was supplied (codes are still shape-checked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountryCodeAuthority {
    SameAdmittedReleaseIso3166Tab,
    ExternalIsoRegistry,
    HostLibrary,
    NotCrossValidated,
}

impl CountryCodeAuthority {
    pub fn as_str(self) -> &'static str {
        match self {
            CountryCodeAuthority::SameAdmittedReleaseIso3166Tab => {
                "same_admitted_release_iso3166_tab"
            }
            CountryCodeAuthority::ExternalIsoRegistry => "external_iso_registry",
            CountryCodeAuthority::HostLibrary => "host_library",
            CountryCodeAuthority::NotCrossValidated => "not_cross_validated",
        }
    }
}

/// The table-structural verdict (T16.4). Deliberately **not** a single `valid: true` — `Conformant` means
/// *structurally admissible as a table*, nothing more (see the module non-claim).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneTableStructuralVerdict {
    Conformant,
    Violation,
}

impl ZoneTableStructuralVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            ZoneTableStructuralVerdict::Conformant => "conformant",
            ZoneTableStructuralVerdict::Violation => "violation",
        }
    }
}

/// The validation of one table.
#[derive(Debug, Clone)]
pub struct AuxTableValidation {
    pub kind: ZoneTableKind,
    pub verdict: ZoneTableStructuralVerdict,
    pub rows_checked: usize,
    /// Which authority country codes were cross-validated against (T16.4).
    pub country_code_authority: CountryCodeAuthority,
    /// Typed findings with their 1-based data-row line (bounded to the first few, to avoid spam).
    pub findings: Vec<(usize, ZoneTableFinding)>,
}

impl AuxTableValidation {
    fn to_json(&self) -> String {
        let mut findings = String::from("[");
        for (i, (line, f)) in self.findings.iter().enumerate() {
            if i > 0 {
                findings.push_str(", ");
            }
            findings.push_str(&format!(
                "{{ \"line\": {}, \"finding\": {} }}",
                line,
                escape(f.as_str())
            ));
        }
        findings.push(']');
        format!(
            "{{ \"kind\": {}, \"artifact_category\": {}, \"coverage\": {}, \"verdict\": {}, \
             \"rows_checked\": {}, \"country_code_authority\": {}, \"findings\": {} }}",
            escape(self.kind.as_str()),
            escape(self.kind.artifact_category().as_str()),
            escape(self.kind.coverage()),
            escape(self.verdict.as_str()),
            self.rows_checked,
            escape(self.country_code_authority.as_str()),
            findings
        )
    }
}

/// A 2-letter uppercase ASCII country code.
fn is_country_code(s: &str) -> bool {
    s.len() == 2 && s.bytes().all(|b| b.is_ascii_uppercase())
}

/// An ISO-6709-style `±DDMM[SS]±DDDMM[SS]` coordinate as used by the zone tables: a sign, 4 or 6 digits
/// (latitude DDMM / DDMMSS), then a sign, 5 or 7 digits (longitude DDDMM / DDDMMSS).
fn is_coordinate(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() || (b[0] != b'+' && b[0] != b'-') {
        return false;
    }
    // Split into [sign digits][sign digits].
    let rest = &s[1..];
    let lon_sign = match rest.find(['+', '-']) {
        Some(i) => i,
        None => return false,
    };
    let lat_digits = &rest[..lon_sign];
    let lon_part = &rest[lon_sign + 1..];
    let lat_ok = (lat_digits.len() == 4 || lat_digits.len() == 6)
        && lat_digits.bytes().all(|c| c.is_ascii_digit());
    let lon_ok = (lon_part.len() == 5 || lon_part.len() == 7)
        && lon_part.bytes().all(|c| c.is_ascii_digit());
    lat_ok && lon_ok
}

/// Validate one auxiliary table from raw bytes (bounds-safe; never panics). `iso3166_codes`, when
/// supplied, cross-validates the country-code columns of `zone1970.tab`. **`zonenow.tab` allows `XX`** (its
/// placeholder convention) so country codes there are not cross-checked.
pub fn validate_zone_table(
    kind: ZoneTableKind,
    bytes: &[u8],
    iso3166_codes: Option<&BTreeSet<String>>,
) -> AuxTableValidation {
    let mut findings: Vec<(usize, ZoneTableFinding)> = Vec::new();
    let push = |findings: &mut Vec<(usize, ZoneTableFinding)>, line: usize, f: ZoneTableFinding| {
        if findings.len() < 32 {
            findings.push((line, f));
        }
    };

    // Which authority did we cross-validate country codes against? `zone.tab` + `zone1970.tab` are
    // cross-checked against a supplied **same-release** `iso3166.tab` (never a host/world ISO list).
    // `zonenow.tab` uses the `XX` placeholder so it is not cross-validated; `iso3166.tab` *is* the source.
    let cross_validates = matches!(kind, ZoneTableKind::ZoneTab | ZoneTableKind::Zone1970Tab);
    let country_code_authority = if cross_validates && iso3166_codes.is_some() {
        CountryCodeAuthority::SameAdmittedReleaseIso3166Tab
    } else {
        CountryCodeAuthority::NotCrossValidated
    };

    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => {
            return AuxTableValidation {
                kind,
                verdict: ZoneTableStructuralVerdict::Violation,
                rows_checked: 0,
                country_code_authority,
                findings: vec![(0, ZoneTableFinding::NonUtf8)],
            };
        }
    };

    let mut rows_checked = 0usize;
    // **Semantic-row** duplicate detection: the *identity tuple* (not the whole line incl. comment, and
    // NOT the country code alone — a country legitimately spans many rows). Zone tables key on
    // `(cc-set, coord, zone-name)`; `iso3166.tab` keys on the country code (codes are a unique reference).
    let mut seen_identity: BTreeSet<String> = BTreeSet::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = idx + 1;
        // Comments + blank lines are not data rows (tz tables use `#`).
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        rows_checked += 1;
        let fields: Vec<&str> = raw.split('\t').collect();

        match kind {
            ZoneTableKind::Iso3166Tab => {
                // `CC \t name` — exactly 2 fields; code 2-upper; name non-empty; code unique.
                if fields.len() != 2 {
                    push(&mut findings, line, ZoneTableFinding::InvalidColumnCount);
                    continue;
                }
                if !is_country_code(fields[0]) {
                    push(&mut findings, line, ZoneTableFinding::InvalidCountryCode);
                } else if !seen_identity.insert(fields[0].to_string()) {
                    push(&mut findings, line, ZoneTableFinding::DuplicateSemanticRow);
                }
                if fields[1].is_empty() {
                    push(&mut findings, line, ZoneTableFinding::EmptyNameField);
                }
            }
            ZoneTableKind::ZoneTab | ZoneTableKind::Zone1970Tab | ZoneTableKind::ZonenowTab => {
                // `CC[,CC…] \t coord \t TZ [\t comment]` — at least 3 fields.
                if fields.len() < 3 {
                    push(&mut findings, line, ZoneTableFinding::InvalidColumnCount);
                    continue;
                }
                // Country code(s): a comma-separated list for zone1970; single for zone.tab; `XX` allowed
                // for zonenow (its placeholder convention) so we do not cross-check those.
                let codes = fields[0];
                let code_ok = codes.split(',').all(|c| {
                    is_country_code(c) || (kind == ZoneTableKind::ZonenowTab && c == "XX")
                });
                if !code_ok {
                    push(&mut findings, line, ZoneTableFinding::InvalidCountryCode);
                } else {
                    // Set semantics for the comma-list (zone1970): a code repeated within the row is a
                    // finding. (zone.tab is single-code, so this never fires there.)
                    if kind == ZoneTableKind::Zone1970Tab {
                        let mut in_row: BTreeSet<&str> = BTreeSet::new();
                        for c in codes.split(',') {
                            if !in_row.insert(c) {
                                push(&mut findings, line, ZoneTableFinding::DuplicateCodeInRow);
                            }
                        }
                    }
                    // Cross-validate each code against the same-release ISO-3166 authority — for BOTH
                    // `zone.tab` and `zone1970.tab` (zonenow's `XX` is excluded by `cross_validates`).
                    if cross_validates {
                        if let Some(set) = iso3166_codes {
                            if !codes.split(',').all(|c| set.contains(c)) {
                                push(&mut findings, line, ZoneTableFinding::InvalidCountryCode);
                            }
                        }
                    }
                }
                if !is_coordinate(fields[1]) {
                    push(
                        &mut findings,
                        line,
                        ZoneTableFinding::InvalidCoordinateFormat,
                    );
                }
                // Zone-name field must be non-empty.
                if fields[2].is_empty() {
                    push(&mut findings, line, ZoneTableFinding::EmptyNameField);
                }
                // Semantic-row identity = (cc-set, coord, zone-name), comments excluded. A country
                // spanning many rows is legal; an identical identity tuple is the real duplicate.
                let identity = format!("{}\t{}\t{}", fields[0], fields[1], fields[2]);
                if !seen_identity.insert(identity) {
                    push(&mut findings, line, ZoneTableFinding::DuplicateSemanticRow);
                }
            }
        }
    }

    if rows_checked == 0 {
        push(&mut findings, 0, ZoneTableFinding::EmptyTable);
    }
    let verdict = if findings.is_empty() {
        ZoneTableStructuralVerdict::Conformant
    } else {
        ZoneTableStructuralVerdict::Violation
    };
    AuxTableValidation {
        kind,
        verdict,
        rows_checked,
        country_code_authority,
        findings,
    }
}

/// Parse the set of country codes from an `iso3166.tab` (first field of each data row) — used to
/// cross-validate `zone1970.tab`. Bounds-safe; non-UTF-8 → empty set.
pub fn iso3166_codes(bytes: &[u8]) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    if let Ok(text) = std::str::from_utf8(bytes) {
        for raw in text.lines() {
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            if let Some(code) = raw.split('\t').next() {
                if is_country_code(code) {
                    set.insert(code.to_string());
                }
            }
        }
    }
    set
}

/// **Bounded** install-ecology status (T16.4) — deliberately narrow so it never implies packager parity.
/// zic-rs writes a **compile output tree under an explicit `--out`** and nothing more: it claims no
/// reference install layout, no `posix`/`right` REDO layout, no `localtime`/`posixrules` completeness, and
/// no runtime tzfile-refresh. The status is a typed inventory value, not a parity claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallEcologyStatus {
    /// Nothing about install ecology is claimed.
    NotClaimed,
    /// Only inventoried (named in the T16.1 inventory), not executed.
    InventoryOnly,
    /// zic-rs materialises only a compile output tree under `--out` — the current, honest state.
    CompileOutputTreeOnly,
}

impl InstallEcologyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            InstallEcologyStatus::NotClaimed => "not_claimed",
            InstallEcologyStatus::InventoryOnly => "inventory_only",
            InstallEcologyStatus::CompileOutputTreeOnly => "compile_output_tree_only",
        }
    }
    /// The current shipped status: a compile output tree only — no install-layout parity is claimed.
    pub fn current() -> Self {
        InstallEcologyStatus::CompileOutputTreeOnly
    }
}

/// The aux-table validation report (T16.4) — schema `zic-rs-aux-table-validation-v1`. A **separate** proof
/// surface from compile/semantic/structural: it asserts table *structural admissibility* only.
#[derive(Debug, Clone)]
pub struct AuxTableValidationReport {
    pub tables: Vec<AuxTableValidation>,
    pub install_ecology: InstallEcologyStatus,
}

impl AuxTableValidationReport {
    pub fn to_json(&self) -> String {
        let mut tables = String::from("[");
        for (i, t) in self.tables.iter().enumerate() {
            if i > 0 {
                tables.push_str(", ");
            }
            tables.push_str(&t.to_json());
        }
        tables.push(']');
        format!(
            "{{\n  \"schema\": \"zic-rs-aux-table-validation-v1\",\n  \
             \"non_claim\": \"a conformant table row proves table structural admissibility only — NOT \
             that the named zone was compiled, semantically witnessed, historically equivalent, or \
             installed; coordinate syntax does NOT claim geodetic accuracy; a public-domain notice is not \
             provenance; release identity is never inferred from table comments\",\n  \
             \"zone_universe\": {},\n  \"table_diagnostic_code_space\": \"separate_table_codes\",\n  \
             \"coordinate_verdict\": \"syntax_only_geodetic_truth_not_claimed\",\n  \
             \"table_comment_disposition\": \"ignored_for_validation\",\n  \
             \"install_ecology_status\": {},\n  \"tables\": {}\n}}\n",
            // The validator resolves NO zone names — structural admissibility only (name the universe!).
            escape(ZoneUniverse::NotResolvedStructuralOnly.as_str()),
            escape(self.install_ecology.as_str()),
            tables
        )
    }
}
