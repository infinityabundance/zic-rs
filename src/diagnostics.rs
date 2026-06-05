//! Located, coded diagnostics about tzdata source.
//!
//! Every diagnostic carries a stable code (`ZIC0xx`), a severity, a human message, the
//! originating file and line, an optional column span within the line, and an optional
//! suggestion. Codes are part of the tool's contract: they let scripts and tests match on
//! a failure class without scraping prose.

use std::fmt;
use std::path::{Path, PathBuf};

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        f.write_str(s)
    }
}

/// Stable diagnostic codes.
///
/// Numbering follows the project's documented scheme; gaps are reserved for future use so
/// existing codes never shift meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// A directive/keyword we do not implement in this version.
    UnsupportedDirective,
    /// A line had the wrong number of fields for its record type.
    InvalidFieldCount,
    /// A month name was unrecognised or ambiguously abbreviated.
    InvalidMonth,
    /// A name abbreviation matched more than one keyword.
    AmbiguousNameAbbreviation,
    /// An `ON` day specification was malformed.
    InvalidDayRule,
    /// An `AT`/`SAVE` time had an invalid suffix.
    InvalidTimeSuffix,
    /// A `FROM`/`TO` year keyword we do not implement.
    UnsupportedYearType,
    /// A zone/link name (or its output path) would escape the output root.
    OutputPathTraversal,
    /// A zone produced more transitions than the configured limit.
    TooManyTransitions,
    /// Leap-second input was supplied but is not implemented.
    UnsupportedLeapSeconds,
    /// Our output disagreed with reference `zic`.
    ReferenceZicMismatch,
    /// A field value was syntactically invalid (offset, name, etc.).
    InvalidValue,
    /// A line in command position began with a token that is not a recognised record keyword
    /// (`Rule`/`Zone`/`Link`). Lexical layer; reference `zic` reports "input line of unknown type".
    /// Distinct from [`UnsupportedDirective`](Self::UnsupportedDirective), which is a *recognised*
    /// construct zic-rs deliberately does not compile (a safety/scope divergence, T13.1).
    UnknownLineType,
    /// An indented (continuation-shaped) line appeared where no zone is open to continue. Structural
    /// layer. zic-rs classifies this more precisely than reference `zic` (which folds it into "input
    /// line of unknown type") — a documented, intentional refinement, not a wording mismatch.
    ContinuationWithoutZone,
    /// A second `Zone` was defined with a name already in use. Structural layer; carries the conflicting
    /// name and the original definition's line (mirrors reference `zic`'s "duplicate zone name N").
    DuplicateZone,
    /// The input contained a NUL byte (lexical/admissibility). Reference `zic`: "NUL input byte".
    NulByteInInput,
    /// A source line exceeded the byte-length cap (lexical/admissibility). The cap (`MAX_LINE_LEN`)
    /// was reconciled against reference `zic` in **T14.1** and **matches** its `_POSIX2_LINE_MAX = 2048`
    /// (the older "511" manpage figure is stale — verified against the pinned 2026b source).
    OverlongInputLine,
    /// A time-zone abbreviation's **length** is outside the portable range (tzfile(5): **3–6
    /// characters**) — fewer than 3 or more than 6 — a non-fatal **warning** (the file still compiles).
    /// Mirrors reference `zic`'s always-on "time zone abbreviation has too many characters (X)" /
    /// "…has fewer than 3 characters (X)" (thresholds pinned empirically against tzcode 2026b). The
    /// first non-fatal warning class (T13.3); the `< 3` side was added in T13.4.
    AbbreviationPolicyViolation,
    /// A time-zone abbreviation contains a character outside the POSIX-portable set (only ASCII
    /// alphanumerics and `+`/`-`), a non-fatal **warning**. Mirrors reference `zic`'s "time zone
    /// abbreviation differs from POSIX standard (X)" (T13.4). A **distinct rule** from the length
    /// policy ([`AbbreviationPolicyViolation`](Self::AbbreviationPolicyViolation)) — both can fire for
    /// one abbreviation. (Case is *not* a violation: lowercase letters are alphanumeric and accepted.)
    AbbreviationNotPosix,
    /// The emitted transition count exceeds a client-compatibility threshold (an **emitted-stream**
    /// warning, `-v`/verbose-only in reference `zic`). Pinned from `zic.c`: `1200 < timecnt` →
    /// "pre-2014 clients may mishandle more than 1200 transition times", `TZ_MAX_TIMES < timecnt` →
    /// "reference clients mishandle …". **Reserved class (T13.5):** the code + reference rule are
    /// recorded; emission is wired in T13.6 (gated by the verbosity model), since it depends on the
    /// finished `timecnt` and must be quiet-by-default to match reference.
    TooManyTransitionsForLegacyClient,
    /// The final input line has content but no terminating newline (lexical/admissibility). Mirrors
    /// reference `zic`'s `inputline`: at EOF with `linelen > 0` it reports "unterminated line" and exits.
    /// A POSIX text file ends every line with `\n`; zic-rs was previously lenient here (accepted the
    /// trailing partial line) — T14.2 makes it fail closed, matching reference. The class name is the
    /// **rule violated**, not the wording ("unterminated line").
    UnterminatedInputLine,
    /// A double-quoted field is opened but never closed before end-of-line (lexical/admissibility).
    /// Mirrors reference `zic`'s `getfields`, which hits the line's terminating `\0` inside a quote and
    /// reports "Odd number of quotation marks", then exits. zic-rs already failed closed here but under
    /// the generic [`InvalidValue`](Self::InvalidValue) (`ZIC012`); T14.2 gives it a dedicated class.
    UnterminatedQuote,
    /// Two rules expand to a transition at the **same instant** (semantic). Mirrors reference `zic`'s
    /// "two rules for same instant", which it treats as a fatal error (`errors=true` → exit 1). zic-rs
    /// fails closed with this code rather than emitting a non-monotonic (invalid) TZif — a same-instant
    /// rule conflict is not silently resolved. Surfaced by the T14.4 pathology ledger, which found that
    /// the prior `debug_assert!` would panic in debug / emit an invalid stream in release.
    SimultaneousTransition,
    /// A zone/link **name** (which becomes an output path) contains a byte outside the portable benign
    /// set (`-`, `_`, ASCII letters, and `/` as the separator) — e.g. a digit, `+`, `.`, or a high/control
    /// byte. A non-fatal **warning** mirroring reference `zic`'s `-v`-gated "file name '%s' contains byte
    /// '…'" (`namecheck`). The hard *structural* path rules (absolute · `..` · `//` · empty · trailing
    /// `/`) are fatal [`OutputPathTraversal`](Self::OutputPathTraversal); this is the softer portability
    /// axis (T14.5). zic-rs has no quiet mode → collects always; surfaced only under `--verbose`.
    ZoneNameNonPortableByte,
    /// A single component of a zone/link **name** exceeds the portable length (reference `zic`'s
    /// `component_len_max = 14`). A non-fatal **warning** mirroring `zic`'s `-v`-gated "file name '%s'
    /// contains overlength component" (`componentcheck`); the file still compiles (T14.5).
    ZoneNameOverlengthComponent,
    /// A parsed time value (a `STDOFF` / `SAVE` / `AT` / `UNTIL` field) has a magnitude **strictly
    /// greater than 24:00:00**. A non-fatal **warning** mirroring reference `zic`'s `noise`/`-v`-gated
    /// `gethms` check (`zic.c`: `hh > HOURSPERDAY || (hh == HOURSPERDAY && (mm||ss))` → "values over 24
    /// hours not handled by pre-2007 versions of zic"). Exactly `24:00:00` does **not** warn; the file
    /// still compiles. zic-rs has no quiet mode → collects always; surfaced only under `--verbose`
    /// (T15.5-remainder; closes the T14.4 ledger residual).
    ValueOverTwentyFourHours,
    /// A Rule `TYPE` (5th) field carries a value other than `-` — the obsolete `-y`/`yearistype`
    /// ecology (`even`/`odd`/`uspres`/`nonpres`), which reference `zic` removed in tzcode 2020a.
    /// **Semantic, fatal.** The modern default refuses it (silently ignoring a year-parity predicate
    /// would mis-compile the rule); `--legacy-yearistype` (YEARISTYPE.1) admits the four known
    /// predicates, but an unknown ("wild") type — or a typed single year — is still this error in
    /// legacy mode (the historical `yearistype.sh` itself exits 1 on a wild type). Distinct from
    /// [`UnsupportedYearType`](Self::UnsupportedYearType), which is the `FROM`/`TO` keyword field.
    UnsupportedRuleType,
}

/// The **layer** a diagnostic belongs to — the first axis of the T13 comparison method
/// (`docs/zic-warning-parity.md`). Recorded as a machine-checkable accessor so the taxonomy cannot
/// silently drift from the doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLayer {
    /// The byte/line stream is not well-formed `zic` text (NUL, overlong, unknown keyword, field count).
    Lexical,
    /// Lines are well-formed but don't assemble into a valid record graph (continuation/duplicate/…).
    Structural,
    /// A field/value is individually invalid or unrepresentable (month, day-rule, time, value, …).
    Semantic,
    /// Accepted, but a portability/obsolescence/client-compat hazard (`-v`-style warnings).
    Warning,
    /// Caused by *output materialization*, not the source text (path traversal, no-clobber, mode,
    /// alias-map validation). A diagnostic, but **not** a source `zic -v` diagnostic (T13.5).
    OperationalMaterialization,
    /// Tool-level / oracle meta (e.g. a `compare` mismatch), not a source diagnostic.
    Meta,
}

/// How verbose a run must be for a diagnostic to surface, mirroring reference `zic`'s `noise`/`-v`
/// gating. **Per-diagnostic, not per-code** — `zic.c::checkabbr` gates "fewer than 3" behind `-v`
/// while "too many"/"differs from POSIX" (same `ZIC018`/`ZIC019` family) are always-on, so the gating
/// is set when the diagnostic is *emitted*, not by its code. The CLI filters on it (T13.6): quiet
/// prints `AlwaysOn` only, `--verbose`/`-v` prints both — matching `zic` vs `zic -v`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticVerbosity {
    /// Surfaced regardless of verbosity (all errors; always-on warnings).
    AlwaysOn,
    /// Surfaced only in verbose mode (reference `zic -v`-gated, e.g. the "fewer than 3" abbreviation).
    VerboseOnly,
}

/// The **source-location precision** a diagnostic currently carries (T13.5 policy → T13.6 made
/// machine-checked). Locations are *not* all equal; this records the level each class actually
/// populates today (a refinement target, not a claim of perfection — e.g. exact token columns are
/// future work for several classes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSpanPrecision {
    /// File only, no meaningful line (a tool/oracle-level meta diagnostic).
    FileOnly,
    /// File + 1-based line.
    Line,
    /// File + line + a related-entity cross-reference location (e.g. duplicate-zone's original line).
    RelatedEntityLocation,
    /// An output/materialization path rather than a source location.
    OperationalPath,
}

impl DiagnosticCode {
    /// The stable string form, e.g. `ZIC001_UNSUPPORTED_DIRECTIVE`.
    pub fn as_str(self) -> &'static str {
        use DiagnosticCode::*;
        match self {
            UnsupportedDirective => "ZIC001_UNSUPPORTED_DIRECTIVE",
            InvalidFieldCount => "ZIC002_INVALID_FIELD_COUNT",
            InvalidMonth => "ZIC003_INVALID_MONTH",
            AmbiguousNameAbbreviation => "ZIC004_AMBIGUOUS_NAME_ABBREVIATION",
            InvalidDayRule => "ZIC005_INVALID_DAY_RULE",
            InvalidTimeSuffix => "ZIC006_INVALID_TIME_SUFFIX",
            UnsupportedYearType => "ZIC007_UNSUPPORTED_YEAR_TYPE",
            OutputPathTraversal => "ZIC008_OUTPUT_PATH_TRAVERSAL",
            TooManyTransitions => "ZIC009_TOO_MANY_TRANSITIONS",
            UnsupportedLeapSeconds => "ZIC010_UNSUPPORTED_LEAP_SECONDS",
            ReferenceZicMismatch => "ZIC011_REFERENCE_ZIC_MISMATCH",
            InvalidValue => "ZIC012_INVALID_VALUE",
            UnknownLineType => "ZIC013_UNKNOWN_LINE_TYPE",
            ContinuationWithoutZone => "ZIC014_CONTINUATION_WITHOUT_ZONE",
            DuplicateZone => "ZIC015_DUPLICATE_ZONE",
            NulByteInInput => "ZIC016_NUL_INPUT_BYTE",
            OverlongInputLine => "ZIC017_OVERLONG_INPUT_LINE",
            AbbreviationPolicyViolation => "ZIC018_ABBREVIATION_POLICY_VIOLATION",
            AbbreviationNotPosix => "ZIC019_ABBREVIATION_NOT_POSIX",
            TooManyTransitionsForLegacyClient => "ZIC020_TOO_MANY_TRANSITIONS_FOR_LEGACY_CLIENT",
            UnterminatedInputLine => "ZIC021_UNTERMINATED_INPUT_LINE",
            UnterminatedQuote => "ZIC022_UNTERMINATED_QUOTE",
            SimultaneousTransition => "ZIC023_SIMULTANEOUS_TRANSITION",
            ZoneNameNonPortableByte => "ZIC024_ZONE_NAME_NONPORTABLE_BYTE",
            ZoneNameOverlengthComponent => "ZIC025_ZONE_NAME_OVERLENGTH_COMPONENT",
            ValueOverTwentyFourHours => "ZIC026_VALUE_OVER_24_HOURS",
            UnsupportedRuleType => "ZIC027_UNSUPPORTED_RULE_TYPE",
        }
    }

    /// The [`DiagnosticLayer`] this code belongs to (T13.5 — the comparison method's first axis). The
    /// `match` is exhaustive, so adding a code without classifying it is a compile error — the taxonomy
    /// stays in lock-step with `docs/zic-warning-parity.md`.
    pub fn layer(self) -> DiagnosticLayer {
        use DiagnosticCode::*;
        use DiagnosticLayer as L;
        match self {
            // Lexical / admissibility — the byte/line stream isn't well-formed `zic` text.
            InvalidFieldCount
            | AmbiguousNameAbbreviation
            | UnknownLineType
            | NulByteInInput
            | OverlongInputLine
            | UnterminatedInputLine
            | UnterminatedQuote => L::Lexical,
            // Structural — well-formed lines that don't assemble into a valid record graph.
            ContinuationWithoutZone | DuplicateZone => L::Structural,
            // Semantic — an individually invalid/unrepresentable field or unsupported construct.
            UnsupportedDirective
            | InvalidMonth
            | InvalidDayRule
            | InvalidTimeSuffix
            | UnsupportedYearType
            | UnsupportedRuleType
            | TooManyTransitions
            | UnsupportedLeapSeconds
            | InvalidValue
            | SimultaneousTransition => L::Semantic,
            // Warning — accepted but a portability/client-compat hazard.
            AbbreviationPolicyViolation
            | AbbreviationNotPosix
            | TooManyTransitionsForLegacyClient
            | ZoneNameNonPortableByte
            | ZoneNameOverlengthComponent
            | ValueOverTwentyFourHours => L::Warning,
            // Operational — caused by output materialization, not the source text.
            OutputPathTraversal => L::OperationalMaterialization,
            // Meta — tool/oracle level, not a source diagnostic.
            ReferenceZicMismatch => L::Meta,
        }
    }

    /// The [`DiagnosticSpanPrecision`] this code currently populates (T13.5 audit, made machine-checked
    /// in T13.6). Exhaustive — a new code must declare its precision or the build fails. *Records the
    /// current level, not a perfection claim* (exact token columns are a future refinement for several
    /// `Line` classes).
    pub fn span_precision(self) -> DiagnosticSpanPrecision {
        use DiagnosticCode::*;
        use DiagnosticSpanPrecision as P;
        match self {
            // Carries a related-entity cross-reference (the original definition's line).
            DuplicateZone => P::RelatedEntityLocation,
            // An output/materialization path, not a source location.
            OutputPathTraversal => P::OperationalPath,
            // Tool/oracle meta — no specific source line.
            ReferenceZicMismatch => P::FileOnly,
            // Everything else is currently file + line (column/byte-offset = future refinement).
            UnsupportedDirective
            | InvalidFieldCount
            | InvalidMonth
            | AmbiguousNameAbbreviation
            | InvalidDayRule
            | InvalidTimeSuffix
            | UnsupportedYearType
            | TooManyTransitions
            | UnsupportedLeapSeconds
            | InvalidValue
            | UnknownLineType
            | ContinuationWithoutZone
            | NulByteInInput
            | OverlongInputLine
            | AbbreviationPolicyViolation
            | AbbreviationNotPosix
            | TooManyTransitionsForLegacyClient
            | UnterminatedInputLine
            | UnterminatedQuote
            | SimultaneousTransition
            | ZoneNameNonPortableByte
            | ZoneNameOverlengthComponent
            | ValueOverTwentyFourHours
            | UnsupportedRuleType => P::Line,
        }
    }

    /// The **canonical severity** of this code (T13.6 — completes the per-code contract metadata so
    /// every code carries `(layer, span_precision, default_severity)`; exhaustive → adding a code
    /// forces a classification). The three non-fatal abbreviation/transition warnings are `Warning`;
    /// every other code is a fail-closed `Error`. *Canonical, not absolute:* the `warn-and-skip`
    /// policy may downgrade an emitted error-diagnostic to `Warning` at the instance level (see
    /// `plan::run`), and **verbosity** is the one genuinely per-*instance* axis (a `Diagnostic` cannot
    /// be constructed without a `severity` and a `verbosity`, so those are type-enforced on every
    /// instance — see [`Diagnostic`]).
    pub fn default_severity(self) -> Severity {
        use DiagnosticCode::*;
        match self {
            AbbreviationPolicyViolation
            | AbbreviationNotPosix
            | TooManyTransitionsForLegacyClient
            | ZoneNameNonPortableByte
            | ZoneNameOverlengthComponent
            | ValueOverTwentyFourHours => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A half-open column span `[start, end)` within a source line, 0-based by byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A single located diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub file: PathBuf,
    /// 1-based line number; 0 means "no specific line".
    pub line: usize,
    pub span: Option<Span>,
    pub suggestion: Option<String>,
    /// Verbosity gating (T13.5), set **per diagnostic** at emission — see [`DiagnosticVerbosity`].
    /// Defaults to [`DiagnosticVerbosity::AlwaysOn`]; the few reference-`zic` `-v`-gated warnings (e.g.
    /// the "fewer than 3 characters" abbreviation case) set [`DiagnosticVerbosity::VerboseOnly`] via
    /// [`Diagnostic::verbose_only`]. (No filter is applied yet — zic-rs surfaces everything; the
    /// default-vs-verbose split is T13.6. This field *records* the gating so it isn't lost.)
    pub verbosity: DiagnosticVerbosity,
}

impl Diagnostic {
    /// Construct an error-severity diagnostic.
    pub fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        file: &Path,
        line: usize,
    ) -> Self {
        Diagnostic {
            severity: Severity::Error,
            code,
            message: message.into(),
            file: file.to_path_buf(),
            line,
            span: None,
            suggestion: None,
            verbosity: DiagnosticVerbosity::AlwaysOn,
        }
    }

    /// Construct a warning-severity diagnostic.
    pub fn warning(
        code: DiagnosticCode,
        message: impl Into<String>,
        file: &Path,
        line: usize,
    ) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            ..Diagnostic::error(code, message, file, line)
        }
    }

    /// Attach a column span (builder style).
    pub fn with_span(mut self, start: usize, end: usize) -> Self {
        self.span = Some(Span { start, end });
        self
    }

    /// Attach a suggestion (builder style).
    pub fn with_suggestion(mut self, s: impl Into<String>) -> Self {
        self.suggestion = Some(s.into());
        self
    }

    /// Mark this diagnostic as **verbose-only** (T13.5) — surfaced only in a future `-v`/verbose mode,
    /// mirroring reference `zic`'s `noise`-gated warnings (e.g. the "fewer than 3 characters"
    /// abbreviation case). Builder style; default is [`DiagnosticVerbosity::AlwaysOn`].
    pub fn verbose_only(mut self) -> Self {
        self.verbosity = DiagnosticVerbosity::VerboseOnly;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `file:line: severity[CODE]: message` — a familiar, greppable shape.
        let loc = if self.line > 0 {
            format!("{}:{}", self.file.display(), self.line)
        } else {
            self.file.display().to_string()
        };
        write!(
            f,
            "{loc}: {}[{}]: {}",
            self.severity, self.code, self.message
        )?;
        if let Some(s) = &self.suggestion {
            write!(f, " (suggestion: {s})")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // T13.5 — the diagnostic-contract metadata. These guard the taxonomy against silent drift from
    // `docs/zic-warning-parity.md`.

    #[test]
    fn layer_maps_each_code_to_its_taxonomy_layer() {
        use DiagnosticCode as C;
        use DiagnosticLayer as L;
        assert_eq!(C::NulByteInInput.layer(), L::Lexical);
        assert_eq!(C::OverlongInputLine.layer(), L::Lexical);
        assert_eq!(C::UnknownLineType.layer(), L::Lexical);
        assert_eq!(C::InvalidFieldCount.layer(), L::Lexical);
        assert_eq!(C::ContinuationWithoutZone.layer(), L::Structural);
        assert_eq!(C::DuplicateZone.layer(), L::Structural);
        assert_eq!(C::InvalidMonth.layer(), L::Semantic);
        assert_eq!(C::UnsupportedDirective.layer(), L::Semantic);
        assert_eq!(C::AbbreviationPolicyViolation.layer(), L::Warning);
        assert_eq!(C::AbbreviationNotPosix.layer(), L::Warning);
        assert_eq!(C::TooManyTransitionsForLegacyClient.layer(), L::Warning);
        // Operational/materialization diagnostics are NOT source `zic -v` diagnostics (T13.5).
        assert_eq!(
            C::OutputPathTraversal.layer(),
            L::OperationalMaterialization
        );
        assert_eq!(C::ReferenceZicMismatch.layer(), L::Meta);
    }

    #[test]
    fn diagnostic_verbosity_defaults_always_on_and_builder_sets_verbose_only() {
        let p = Path::new("t.zi");
        let d = Diagnostic::warning(DiagnosticCode::AbbreviationPolicyViolation, "m", p, 1);
        assert_eq!(d.verbosity, DiagnosticVerbosity::AlwaysOn);
        let v = d.verbose_only();
        assert_eq!(v.verbosity, DiagnosticVerbosity::VerboseOnly);
    }

    #[test]
    fn zic020_transition_count_code_is_stable() {
        assert_eq!(
            DiagnosticCode::TooManyTransitionsForLegacyClient.as_str(),
            "ZIC020_TOO_MANY_TRANSITIONS_FOR_LEGACY_CLIENT"
        );
    }

    /// Every code carries the contract metadata `(as_str, layer, span_precision)` and the codes are
    /// distinct & append-ordered (`ZIC001`…`ZIC020`). This is the machine-enforced contract: the
    /// `match`es in `layer()`/`span_precision()` are exhaustive, so adding a code without classifying
    /// it is a compile error; this test additionally pins ordering/uniqueness (T13.6 stability policy).
    #[test]
    fn every_code_has_contract_metadata_and_codes_are_append_ordered() {
        use DiagnosticCode::*;
        const ALL: &[DiagnosticCode] = &[
            UnsupportedDirective,
            InvalidFieldCount,
            InvalidMonth,
            AmbiguousNameAbbreviation,
            InvalidDayRule,
            InvalidTimeSuffix,
            UnsupportedYearType,
            OutputPathTraversal,
            TooManyTransitions,
            UnsupportedLeapSeconds,
            ReferenceZicMismatch,
            InvalidValue,
            UnknownLineType,
            ContinuationWithoutZone,
            DuplicateZone,
            NulByteInInput,
            OverlongInputLine,
            AbbreviationPolicyViolation,
            AbbreviationNotPosix,
            TooManyTransitionsForLegacyClient,
            UnterminatedInputLine,
            UnterminatedQuote,
            SimultaneousTransition,
            ZoneNameNonPortableByte,
            ZoneNameOverlengthComponent,
            ValueOverTwentyFourHours,
            UnsupportedRuleType,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (i, code) in ALL.iter().enumerate() {
            let s = code.as_str();
            // Append-ordered numbering: the Nth code is `ZIC{N+1:03}_…`.
            assert!(
                s.starts_with(&format!("ZIC{:03}_", i + 1)),
                "code {i} = {s} is not append-ordered"
            );
            assert!(seen.insert(s), "duplicate code string {s}");
            // Per-code contract metadata is total — these would not compile if a variant were
            // unclassified (layer/span/severity are exhaustive `match`es). Verbosity is the one
            // per-instance axis, type-enforced as a required `Diagnostic` field.
            let _ = code.layer();
            let _ = code.span_precision();
            let _ = code.default_severity();
        }
        assert_eq!(seen.len(), 27, "expected ZIC001–ZIC027");
        // The three non-fatal warning classes; everything else is a fail-closed error.
        assert_eq!(
            AbbreviationPolicyViolation.default_severity(),
            Severity::Warning
        );
        assert_eq!(AbbreviationNotPosix.default_severity(), Severity::Warning);
        assert_eq!(
            TooManyTransitionsForLegacyClient.default_severity(),
            Severity::Warning
        );
        assert_eq!(UnknownLineType.default_severity(), Severity::Error);
    }

    #[test]
    fn span_precision_classifies_related_entity_and_operational() {
        use DiagnosticCode as C;
        use DiagnosticSpanPrecision as P;
        assert_eq!(C::DuplicateZone.span_precision(), P::RelatedEntityLocation);
        assert_eq!(C::OutputPathTraversal.span_precision(), P::OperationalPath);
        assert_eq!(C::ReferenceZicMismatch.span_precision(), P::FileOnly);
        assert_eq!(C::UnknownLineType.span_precision(), P::Line);
    }
}
