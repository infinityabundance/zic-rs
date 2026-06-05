//! The leap-second table model (T11) — **a distinct time model, not a transition stream.**
//!
//! ## Invariant (do not let this bleed into ordinary-zone code)
//!
//! Leap seconds are **not** local-time-type transitions:
//! * they do not select an abbreviation,
//! * they do not change `isdst`,
//! * they only contribute to the TZif **leap-second correction table**.
//!
//! These types are deliberately separate from [`crate::tzif::Transition`] /
//! [`crate::tzif::LocalTimeType`] so the distinction is visible in code. They are parsed *only* from
//! an explicit leap-source file (see [`crate::source::parse_leap_source`]); the ordinary `tzdata.zi`
//! zone-source path never produces them.

/// One leap-second table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeapSecond {
    /// The leap instant **as written**, in seconds since the 1970 epoch. For a `Stationary` leap this
    /// is the UT instant; for a `Rolling` leap it is the *local-wall* instant — the offset conversion
    /// is a later (T11.4) emission concern, so the value is stored verbatim with `rolling` recorded.
    pub trans: i64,
    /// Per-line correction: `+1` or `-1`. (Reference `zic` accumulates these into a *cumulative*
    /// table at emission via `adjleap`; that is an emission concern, not done here.)
    pub correction: i32,
    /// `true` = `Rolling` (local-wall instant), `false` = `Stationary` (UT instant).
    pub rolling: bool,
}

/// A parsed leap-second table: the entries plus an optional table-expiry instant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LeapTable {
    /// Leap entries, kept **sorted by `trans`** (insertion-sorted, mirroring `zic`'s `leapadd`).
    pub entries: Vec<LeapSecond>,
    /// The `Expires` instant (seconds since 1970), if any. At most one `Expires` line is permitted
    /// (a second is a parse error, matching reference `zic`).
    pub expires: Option<i64>,
}
