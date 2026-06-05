//! Source front-end: turning tzdata text into a typed [`Database`](crate::model::Database).
//!
//! Pipeline within this module:
//!
//! ```text
//! bytes --lexer::tokenize--> Vec<Line>  --parser::parse_into--> Database
//! ```
//!
//! * [`lexer`] handles field splitting, comments, quoting, and input-hardening limits.
//! * [`names`] resolves month/weekday/keyword abbreviations the way `zic` does.
//! * [`records`] holds the lexical [`Line`](records::Line)/[`Field`](records::Field) types.
//! * [`parser`] builds the typed `Rule`/`Zone`/`Link` records.

pub mod leap;
pub mod lexer;
pub mod names;
pub mod parser;
pub mod records;

pub use leap::parse_leap_source;
pub use parser::{parse_into, parse_into_with};

/// Opt-in **historical-source replay** modes — each admits a bounded, era-specific source shape that
/// the modern default refuses, *without* weakening the modern contract. They are explicit flags, never
/// the default; composable (`--legacy-latin1 --legacy-yearistype`). One typed bundle so adding a mode
/// is a field, not another positional `bool`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LegacySource {
    /// LEGACY-SOURCE.1 — admit non-UTF-8 (Latin-1) bytes **only inside `#` comments** of pre-2013
    /// historical tzdb source (semantics-bearing fields still fail closed).
    pub latin1: bool,
    /// YEARISTYPE.1 — admit the historical Rule `TYPE` predicates (`even`/`odd`/`uspres`/`nonpres`)
    /// of pre-2000f source (the `-y`/`yearistype` ecology removed from reference `zic` in tzcode
    /// 2020a). The default rejects any non-`-` TYPE.
    pub yearistype: bool,
}
