//! The parsed, still-textual *semantic model* of a tzdata source: the records the parser
//! produces and the compiler consumes.
//!
//! These types mirror the three tzdata line kinds (`Rule`, `Zone`, `Link`) plus the
//! continuation structure of zones. They are intentionally close to the source — value
//! interpretation (calendar math, transition generation) happens later, in `compile`.

pub mod calendar;
pub mod leap;
pub mod time;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use leap::{LeapSecond, LeapTable};
pub use time::{Offset, Save, TimeOfDay, TimeRef};

/// Where a record came from, for diagnostics.
#[derive(Debug, Clone)]
pub struct Origin {
    pub file: PathBuf,
    pub line: usize,
}

impl Origin {
    pub fn new(file: &Path, line: usize) -> Self {
        Origin {
            file: file.to_path_buf(),
            line,
        }
    }
}

/// A `Rule` line. Stored verbatim-ish in T1 (only validated, not yet expanded); the
/// transition compiler in T2 consumes the typed fields.
#[derive(Debug, Clone)]
pub struct RuleRecord {
    pub name: String,
    /// `FROM` year.
    pub from: i32,
    /// `TO` year (after resolving `only`/`maximum`).
    pub to: YearBound,
    pub in_month: u8,
    pub on: calendar::OnDay,
    pub at: TimeOfDay,
    pub save: Save,
    /// `LETTER/S` — the variable part of the abbreviation (`-` becomes empty).
    pub letter: String,
    /// The historical `TYPE` field (the 5th field). Modern tzdata is always [`YearType::All`]
    /// (`-`); the year-parity predicates are admitted only under `--legacy-yearistype`
    /// (YEARISTYPE.1). The rule fires in a year only if `year_type.includes(year)`.
    pub year_type: YearType,
    pub origin: Origin,
}

/// The historical Rule `TYPE` field (tzcode pre-2020a; the `-y`/`yearistype` ecology, removed in
/// tzcode 2020a). Modern tzdata uses only `-` ([`YearType::All`]). The non-trivial predicates are
/// **anchored to `yearistype.sh` v7.4** (the script `zic` shelled out to; the four cases are the
/// only ones that script defines) and confirmed against an old `zic` oracle on `Australia/Adelaide`
/// 1990–1994:
///
/// - `even`    — the decimal year ends in `{0,2,4,6,8}`  ⇔ `year % 2 == 0`
/// - `odd`     — the decimal year ends in `{1,3,5,7,9}`  ⇔ `year % 2 != 0`
/// - `uspres`  — a U.S. presidential-election year       ⇔ `year % 4 == 0`
/// - `nonpres` — *not* a presidential-election year      ⇔ `year % 4 != 0`
///
/// Only `even`/`odd` actually appear in the stable tzdata archive (the `AS` rules for
/// Australia/Adelaide & Broken_Hill, 1990–1994); `uspres`/`nonpres` are recognised for fidelity to
/// the script but are not exercised by any admitted release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum YearType {
    /// `-` — applies to every year in `[FROM, TO]` (the only modern value).
    #[default]
    All,
    /// `even` — even years.
    Even,
    /// `odd` — odd years.
    Odd,
    /// `uspres` — U.S. presidential-election years (divisible by 4).
    Uspres,
    /// `nonpres` — non-presidential-election years.
    Nonpres,
}

impl YearType {
    /// True iff a rule carrying this type fires in `year` — mirrors `zic`'s
    /// `r_todo = … && yearistype(year, type)` (the script returns true / exit 0 for a matching year).
    /// `rem_euclid` keeps the parity well-defined for negative years (the admitted band is all
    /// positive, but the predicate stays total).
    pub fn includes(self, year: i32) -> bool {
        match self {
            YearType::All => true,
            YearType::Even => year.rem_euclid(2) == 0,
            YearType::Odd => year.rem_euclid(2) != 0,
            YearType::Uspres => year.rem_euclid(4) == 0,
            YearType::Nonpres => year.rem_euclid(4) != 0,
        }
    }

    /// The canonical source token for this type (`-`/`even`/`odd`/`uspres`/`nonpres`).
    pub fn as_str(self) -> &'static str {
        match self {
            YearType::All => "-",
            YearType::Even => "even",
            YearType::Odd => "odd",
            YearType::Uspres => "uspres",
            YearType::Nonpres => "nonpres",
        }
    }

    /// Parse a historical `TYPE` field token. `-` is always [`YearType::All`]; the four predicates
    /// are recognised here. Returns `None` for an unknown ("wild") type — the caller decides whether
    /// that is a hard error (it always is: reference `yearistype.sh` exits 1 on a wild type).
    pub fn from_field(token: &str) -> Option<YearType> {
        match token {
            "-" => Some(YearType::All),
            "even" => Some(YearType::Even),
            "odd" => Some(YearType::Odd),
            "uspres" => Some(YearType::Uspres),
            "nonpres" => Some(YearType::Nonpres),
            _ => None,
        }
    }
}

/// Upper bound of a rule's year range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearBound {
    /// A concrete year.
    Year(i32),
    /// `maximum` — extends indefinitely.
    Max,
}

/// The `RULES` column of a zone era.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneRules {
    /// `-` : standard time throughout this era.
    None,
    /// An inline saving (the column held a time like `1:00`), not a named ruleset.
    Save(Save),
    /// A named ruleset (`Rule NAME ...`).
    Named(String),
}

/// One era of a zone (one `Zone`/continuation line up to its `UNTIL`).
#[derive(Debug, Clone)]
pub struct ZoneEra {
    /// Standard UT offset for the era (`STDOFF`), seconds east of UTC.
    pub stdoff: Offset,
    pub rules: ZoneRules,
    /// Abbreviation template (`FORMAT`).
    pub format: String,
    /// `UNTIL` instant ending this era; `None` for the final era.
    pub until: Option<Until>,
    pub origin: Origin,
}

/// The `UNTIL` field: a partially-specified wall/standard/UT instant.
#[derive(Debug, Clone)]
pub struct Until {
    pub year: i32,
    pub month: u8,
    pub day: calendar::OnDay,
    pub time: TimeOfDay,
}

/// A `Zone` plus its continuation lines.
#[derive(Debug, Clone)]
pub struct ZoneRecord {
    pub name: String,
    pub eras: Vec<ZoneEra>,
    pub origin: Origin,
}

/// A `Link` line: `LINK-NAME` is an alias for `TARGET`.
#[derive(Debug, Clone)]
pub struct LinkRecord {
    pub target: String,
    pub link_name: String,
    pub origin: Origin,
}

/// The whole parsed source: zones, links, and rules keyed by name.
#[derive(Debug, Default, Clone)]
pub struct Database {
    pub zones: Vec<ZoneRecord>,
    pub links: Vec<LinkRecord>,
    pub rules: BTreeMap<String, Vec<RuleRecord>>,
}

impl Database {
    /// Find a zone by exact name.
    pub fn zone(&self, name: &str) -> Option<&ZoneRecord> {
        self.zones.iter().find(|z| z.name == name)
    }
}
