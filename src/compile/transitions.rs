//! Transition generation for rule-driven zones — the heart of the compiler.
//!
//! Two entry points:
//! * [`compile_rule_zone`] — a **single-era** zone `Zone … STDOFF RULES FORMAT` whose `RULES`
//!   names a rule set: expand the rules across the years they apply, producing the UT
//!   transition instants and the local-time-types they switch between, plus the POSIX footer
//!   for the open-ended future (fixed for finite rule sets, recurring for `TO = maximum`).
//! * [`compile_multi_era`] — a **multi-era** zone (`UNTIL` continuations): the same per-era
//!   machinery, plus the hard cross-era stitching (see the section comment lower in this file
//!   for the five traps), the final-recurring-era anchor, **effective-in-era** rule
//!   classification (a final era whose finite rows all end before it starts is recurring-only),
//!   and **inline-save** eras (`EraKind::InlineSave`: a `RULES` clock value → a constant
//!   `STDOFF + SAVE` type). Single-era inline-save lives in `compile::compile_inline_save`.
//!
//! Behaviour is validated against reference `zic` via the `zdump` oracle; the explicit-
//! transition representation may be "fatter" than `zic`'s slim emission (zdump-equivalent —
//! see `docs/reference-zic-semantics.md` §6).
//!
//! ## The one genuinely tricky part: wall/standard/UT → UT conversion
//!
//! A rule's `AT` time is a *local* time expressed against one of three clocks:
//!
//! * **wall** (`w`, the default): the local clock people read, i.e. `STDOFF + save`, where
//!   `save` is whatever is in effect **just before** this transition. This is the subtle
//!   bit — the conversion depends on the *previous* rule's saving, so transitions must be
//!   processed in chronological order while tracking the running `save`.
//! * **standard** (`s`): standard local time, i.e. `STDOFF` (independent of DST).
//! * **universal** (`u`/`g`/`z`): already UT.
//!
//! So for a wall-clock `AT`:  `ut = local_seconds - STDOFF - save_prev`.
//! For standard:              `ut = local_seconds - STDOFF`.
//! For universal:             `ut = local_seconds`.
//!
//! ## Finite vs recurring
//!
//! * **Finite** rule sets (every `TO` a concrete year): expand across `[min FROM, max TO]`.
//!   Afterwards the zone stays in its last state forever → a **fixed** POSIX footer.
//! * **Recurring** rule sets (a `TO = maximum` rule): the infinite tail is described by a
//!   **recurring** POSIX footer (e.g. `EST5EDT,M3.2.0,M11.1.0`). We still emit explicit
//!   transitions across `[min FROM, RECUR_HI]` so readers without footer support, and
//!   `zdump` within that window, see real transitions; the footer covers beyond. We only
//!   accept recurring sets we can describe *exactly* (one perpetual standard rule + one
//!   perpetual DST rule, with POSIX-expressible day forms); anything else fails closed.

use crate::compile::posix_footer::{self, Recurring};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::error::{Error, Result};
use crate::model::calendar::{days_from_civil, resolve_on_day, year_of_unix};
use crate::model::time::{TimeOfDay, TimeRef};
use crate::model::{Database, RuleRecord, Save, Until, YearBound, ZoneEra, ZoneRecord, ZoneRules};
use crate::tzif::{LocalTimeType, Transition, TzifData};

use super::abbreviations::render;

/// Defence against pathological rule expansion until the configurable limit is wired in.
const MAX_TRANSITIONS: usize = 100_000;

/// Last year for which we emit *explicit* transitions for a recurring rule set; the POSIX
/// footer governs everything after. (Comfortably past the 32-bit horizon; the footer makes
/// the exact cut-over immaterial to behaviour.)
const RECUR_HI: i32 = 2037;

/// A well-formed transition stream is **strictly increasing** in `at` (TZif requires it, and `zdump`
/// reads it that way). If two rules expand to the same instant — reference `zic`'s "two rules for same
/// instant", which it treats as a fatal error — we **fail closed** with a coded diagnostic instead of
/// panicking (the prior `debug_assert!`) or, worse, emitting a non-monotonic TZif in a release build.
/// (The 341 canonical zones are all strictly increasing, so this never fires on valid data; it only
/// catches the degenerate/hostile same-instant conflict the T14.4 ledger surfaced.)
fn ensure_strictly_increasing(transitions: &[Transition], zone: &ZoneRecord) -> Result<()> {
    if transitions.windows(2).any(|w| w[0].at >= w[1].at) {
        return Err(Error::from(Diagnostic::error(
            DiagnosticCode::SimultaneousTransition,
            "two rules produce a transition at the same instant",
            &zone.origin.file,
            zone.origin.line,
        )));
    }
    Ok(())
}

/// Compile a single-era zone whose `RULES` field names the given `rules` set.
pub fn compile_rule_zone(
    zone: &ZoneRecord,
    era: &ZoneEra,
    rules: &[RuleRecord],
    opts: crate::EmitOptions,
) -> Result<TzifData> {
    let stdoff = era.stdoff.0;
    if rules.is_empty() {
        return Err(unsupported(zone, "named rule set has no rules"));
    }
    // (`FROM = minimum` is coerced to 1900 at parse time, exactly as reference `zic` does for
    // that obsolete keyword — so there is no unbounded-past expansion to guard against here.)

    let has_recurring = rules.iter().any(|r| matches!(r.to, YearBound::Max));
    let lo = rules.iter().map(|r| r.from).min().expect("non-empty");
    let hi = if has_recurring {
        RECUR_HI
    } else {
        rules
            .iter()
            .filter_map(|r| match r.to {
                YearBound::Year(y) => Some(y),
                YearBound::Max => None,
            })
            .max()
            .expect("finite set has a concrete TO")
    };

    // 1. Expand every (rule, year) activation in the span, ordered by naive-local instant.
    let activations = expand_activations(zone, rules, lo, hi)?;

    if activations.len() > MAX_TRANSITIONS {
        return Err(located(
            zone,
            DiagnosticCode::TooManyTransitions,
            format!(
                "zone produces {} transitions, exceeding the limit of {MAX_TRANSITIONS}",
                activations.len()
            ),
        ));
    }

    // 3. Intern the initial (pre-first-transition) type: standard time, save 0, with the
    //    rule set's standard LETTER. Interning it first makes it type index 0, which is also
    //    what a reader uses before the first transition (first non-DST type).
    let mut types: Vec<LocalTimeType> = Vec::new();
    let std_letter = standard_letter(rules);
    let initial = LocalTimeType {
        utoff: stdoff,
        is_dst: false,
        abbr: render(&era.format, &std_letter, false, stdoff),
    };
    let _ = intern(&mut types, initial);

    // 4. Walk activations in order, converting each to a UT instant using the prevailing
    //    save, recording the transition + (interned) type.
    let mut transitions: Vec<Transition> = Vec::new();
    // Parallel to `transitions`: whether each came from a recurring (`TO = maximum`) rule. Used only
    // by the `zic-slim` finalize below; on the default path it is harmless bookkeeping.
    let mut from_recurring: Vec<bool> = Vec::new();
    let mut save_prev: i32 = 0;
    let mut last_type_index: u8 = 0;
    for act in &activations {
        let ut = match act.at_ref {
            TimeRef::Wall => act.local_seconds - stdoff as i64 - save_prev as i64,
            TimeRef::Standard => act.local_seconds - stdoff as i64,
            TimeRef::Universal => act.local_seconds,
        };
        let utoff = stdoff + act.save;
        let ty = LocalTimeType {
            utoff,
            is_dst: act.is_dst,
            abbr: render(&era.format, &act.letter, act.is_dst, utoff),
        };
        let type_index = intern(&mut types, ty);
        transitions.push(Transition { at: ut, type_index });
        from_recurring.push(act.from_recurring);
        last_type_index = type_index;
        save_prev = act.save;
    }

    // 5. Footer. Recurring sets get an exact recurring POSIX string (or fail closed);
    //    finite sets settle permanently into the last transition's state → fixed footer.
    let (mut footer, version) = if has_recurring {
        match build_recurring_footer(zone, era, stdoff, rules) {
            Ok(fv) => fv,
            // PERPETUAL-EXPANSION.1: the recurring tail is not POSIX-footer-expressible (the live
            // case: perpetual year-parity, `1990 max even/odd`). Explicit transitions already cover
            // `[lo, RECUR_HI]`; under `--legacy-empty-footer` fall back to an **empty footer** (frozen
            // beyond) instead of failing closed — matching the historical oracle's behaviour over the
            // declared window. Off by default → footer-synthesising zones never reach here.
            Err(_) if opts.allow_empty_footer_on_legacy_nonposix_recurrence => {
                (String::new(), b'2')
            }
            Err(e) => return Err(e),
        }
    } else {
        let tail = &types[last_type_index as usize];
        (posix_footer::fixed_offset(&tail.abbr, tail.utoff), b'2')
    };

    // T10.4d: range truncation (`-r`) shapes the **fat** stream **before** slim — matching reference
    // `zic`'s order (limitrange then bloat). Doing it after slim would be wrong when `lo` falls past
    // the footer-governed tail that slim drops (the prevailing type at `lo` would be lost).
    if let Some(range) = opts.range {
        apply_range_to_stream(
            &mut types,
            &mut transitions,
            &mut from_recurring,
            &mut footer,
            range,
        );
    }
    // T8-slim: optionally truncate the footer-governed recurring tail (no-op for the default).
    finalize_emit_style(opts, &mut types, &mut transitions, &from_recurring);

    ensure_strictly_increasing(&transitions, zone)?;

    Ok(TzifData {
        types,
        transitions,
        footer,
        version,
        leaps: Vec::new(),
    })
}

/// One expanded rule activation, before UT conversion.
struct Activation {
    local_seconds: i64,
    at_ref: TimeRef,
    save: i32,
    is_dst: bool,
    letter: String,
    /// Whether the source rule is recurring (`TO = maximum`). Used only by the `--emit-style
    /// zic-slim` truncation (T8-slim): a transition from a recurring rule is *footer-governed* and
    /// may be dropped past the anchor; a finite-rule (or era-boundary) transition is not. Mirrors
    /// `zic.c`'s `nonTZlimtime` rule (`rp->r_hiyear != ZIC_MAX`). Ignored by the default emission.
    from_recurring: bool,
}

/// Is `rule` in effect during `year`?
///
/// The window is `[FROM, TO]` **and** the historical `TYPE` predicate (`year_type`) — mirroring
/// `zic`'s `r_todo = year >= loyear && year <= hiyear && yearistype(year, type)`. For modern data the
/// type is always [`YearType::All`](crate::model::YearType::All), so the extra clause is a no-op; it
/// only filters the `even`/`odd`/… rules admitted under `--legacy-yearistype` (YEARISTYPE.1).
fn rule_active_in(rule: &RuleRecord, year: i32) -> bool {
    let to = match rule.to {
        YearBound::Year(y) => y,
        YearBound::Max => i32::MAX,
    };
    rule.from <= year && year <= to && rule.year_type.includes(year)
}

/// The LETTER for standard time: that of a standard-time (non-DST) rule, else empty.
fn standard_letter(rules: &[RuleRecord]) -> String {
    rules
        .iter()
        .find(|r| !r.save.is_dst)
        .map(|r| r.letter.clone())
        .unwrap_or_default()
}

/// Build the recurring POSIX footer for a `TO = maximum` rule set, or fail closed.
///
/// We require exactly one perpetual standard rule and one perpetual DST rule, with day forms
/// POSIX can express. The `AT` times are converted to **local wall** time here, since this
/// is where the prevailing offsets are known: at DST onset the prior save is 0; at DST end
/// the prior save is the DST amount.
/// Returns `(footer, version)` — the POSIX `TZ` footer and the TZif version byte it requires
/// (`b'2'`, or `b'3'` when the footer needs an out-of-`0..24h` transition time, e.g. Asia/Gaza).
fn build_recurring_footer(
    zone: &ZoneRecord,
    era: &ZoneEra,
    stdoff: i32,
    rules: &[RuleRecord],
) -> Result<(String, u8)> {
    let perpetual: Vec<&RuleRecord> = rules
        .iter()
        .filter(|r| matches!(r.to, YearBound::Max))
        .collect();
    let dst_rules: Vec<&&RuleRecord> = perpetual.iter().filter(|r| r.save.is_dst).collect();
    let std_rules: Vec<&&RuleRecord> = perpetual.iter().filter(|r| !r.save.is_dst).collect();
    if dst_rules.len() != 1 || std_rules.len() != 1 {
        return Err(unsupported(
            zone,
            "recurring footer needs exactly one perpetual DST rule and one standard rule",
        ));
    }
    let dst = dst_rules[0];
    let std = std_rules[0];

    let dst_save = dst.save.seconds;
    let std_utoff = stdoff;
    let dst_utoff = stdoff + dst_save;

    let std_abbr = render(&era.format, &std.letter, false, std_utoff);
    let dst_abbr = render(&era.format, &dst.letter, true, dst_utoff);

    // DST onset: prior save is 0. DST end: prior save is the DST amount.
    let dst_at_wall = at_to_wall(dst.at, stdoff, 0);
    let std_at_wall = at_to_wall(std.at, stdoff, dst_save);

    let rec = Recurring {
        std_abbr: &std_abbr,
        std_utoff,
        dst_abbr: &dst_abbr,
        dst_utoff,
        dst_month: dst.in_month,
        dst_on: dst.on,
        dst_at_wall,
        std_month: std.in_month,
        std_on: std.on,
        std_at_wall,
    };
    posix_footer::recurring(&rec).ok_or_else(|| {
        unsupported(
            zone,
            "recurring rule day form is not POSIX-expressible (only nth/last weekday) — \
             refusing rather than emitting an approximate footer",
        )
    })
}

/// Convert an `AT` time to local wall seconds, given the standard offset and the `save` in
/// effect just before the transition. (Wall is already wall; standard adds the prior save;
/// universal adds the full prevailing offset `stdoff + save_before`.)
fn at_to_wall(at: TimeOfDay, stdoff: i32, save_before: i32) -> i32 {
    match at.reference {
        TimeRef::Wall => at.seconds,
        TimeRef::Standard => at.seconds + save_before,
        TimeRef::Universal => at.seconds + stdoff + save_before,
    }
}

/// Intern a local-time-type, returning its index (reusing an identical existing type).
fn intern(types: &mut Vec<LocalTimeType>, ty: LocalTimeType) -> u8 {
    if let Some(i) = types.iter().position(|t| *t == ty) {
        return i as u8;
    }
    types.push(ty);
    (types.len() - 1) as u8
}

/// Expand every `(rule, year)` activation of `rules` over `[lo, hi]` into [`Activation`]s,
/// ordered by naive-local instant (month/day order within a year; chronological across
/// years). The naive sort key ignores `save`; callers compute the *actual* UT instant using
/// the running `save`. Shared by the single-era and multi-era compilers.
fn expand_activations(
    zone: &ZoneRecord,
    rules: &[RuleRecord],
    lo: i32,
    hi: i32,
) -> Result<Vec<Activation>> {
    let mut activations: Vec<Activation> = Vec::new();
    for year in lo..=hi {
        for rule in rules {
            if !rule_active_in(rule, year) {
                continue;
            }
            let day = resolve_on_day(rule.on, year, rule.in_month)
                .map_err(|(code, msg)| located(zone, code, msg))?;
            let local_seconds = days_from_civil(day.year, day.month, day.day as u8) * 86400
                + rule.at.seconds as i64;
            activations.push(Activation {
                local_seconds,
                at_ref: rule.at.reference,
                save: rule.save.seconds,
                is_dst: rule.save.is_dst,
                letter: rule.letter.clone(),
                from_recurring: matches!(rule.to, YearBound::Max),
            });
        }
    }
    activations.sort_by_key(|a| a.local_seconds);
    Ok(activations)
}

// ===========================================================================================
// Multi-era zones (milestone T3.1): `Zone` lines with `UNTIL` continuations.
// ===========================================================================================
//
// A multi-era zone is a chain of eras, each active on a half-open UT interval
// `[start, until)`. Compiling it is the hard part of `zic` because state crosses era
// boundaries. The five traps we must get exactly right (each proven by the `zdump` oracle):
//
//   1. An era's `UNTIL` is converted to UT using **that era's** `stdoff` and the `save`
//      prevailing *at the until moment* — not the next era's offset, and not save 0 if DST
//      is active there. (Off-by-one-hour trap.)
//   2. State carries across boundaries: a `-` era resets `save` to 0 and uses its literal
//      abbreviation; a ruled era is seeded in standard time and evolved by its own rules
//      (pre-start activations evolve the state used for the boundary transition).
//   3. The pre-first-transition type is the first era's standard/initial state (interned as
//      type 0, which is what a reader uses before the first transition).
//   4. The footer comes from the **final** era only.
//   5. A boundary transition is dropped only when it is a true no-op (identical
//      `(utoff, is_dst, abbr)` to the prevailing type), never when it is a real change —
//      e.g. EDT→AST at an equal `utoff` but different DST flag/abbreviation must be kept.

/// The running local-time state while walking an era: the DST `save`, its daylight flag, and
/// the abbreviation `LETTER` for `%s` rendering.
#[derive(Clone)]
struct RunState {
    save: i32,
    is_dst: bool,
    letter: String,
}

/// Compile a multi-era zone (two or more `Zone`/continuation lines) into TZif data.
pub fn compile_multi_era(
    zone: &ZoneRecord,
    db: &Database,
    opts: crate::EmitOptions,
) -> Result<TzifData> {
    let mut types: Vec<LocalTimeType> = Vec::new();
    let mut transitions: Vec<Transition> = Vec::new();
    // Parallel to `transitions` (kept in lockstep via `push`'s return): whether each came from a
    // recurring (`TO = maximum`) rule. Read only by `finalize_emit_style` (zic-slim); inert default.
    let mut from_recurring: Vec<bool> = Vec::new();
    let mut last_ut = i64::MIN;

    // --- Trap #3: the pre-first-transition type is the first era's standard/initial state. ---
    let first = &zone.eras[0];
    let (first_rules, first_kind) = resolve_era_rules(zone, db, first)?;
    validate_fixed_era_format(zone, first, &first_kind)?;
    let initial = era_type(
        first.stdoff.0,
        &seed_run(&first_kind, first_rules),
        &first.format,
    );
    let mut prevailing = intern(&mut types, initial);

    // `start_ut` is the UT instant at which the current era begins; `None` means -infinity
    // (the first era). It is set to each era's computed `UNTIL` instant for the next era.
    let mut start_ut: Option<i64> = None;
    // The previous era's `UNTIL` as a *naive local time + clock reference* — i.e. the boundary in
    // the clock frame it was written in, before any offset conversion. Carried so the new era can
    // recognise a rule activation that names the *same standard-clock instant* as the boundary
    // even when the two eras' standard offsets differ (see the absorption logic below).
    let mut boundary_until: Option<(i64, TimeRef)> = None;

    for era in &zone.eras {
        let stdoff = era.stdoff.0;
        let (rules, kind) = resolve_era_rules(zone, db, era)?;
        validate_fixed_era_format(zone, era, &kind)?;
        // A *fixed* era (literal or inline-save) has no rules to expand and emits a single
        // constant type; only a `Ruled` era walks activations.
        let is_fixed = !matches!(kind, EraKind::Ruled);
        // (`FROM = minimum` was coerced to 1900 at parse time — reference `zic`'s obsolete-keyword
        // behaviour — so no unbounded-past guard is needed.)
        let recurring = rules.iter().any(|r| matches!(r.to, YearBound::Max));

        // Seed the running state for this era: standard time for a literal/ruled era, or the
        // constant saving for an inline-save era (its `is_dst` carries the suffix/zero heuristic).
        // A ruled era is then evolved by its own activations below; a fixed era keeps the seed.
        let mut run = seed_run(&kind, rules);

        // T5 #4 — a ruled era inherits the rule set's *prevailing* state at the era start, even when
        // the activation that set it predates this era's local expansion window. A rule's
        // `save`/`is_dst`/`letter` persist until the rule *itself* changes them; an intervening era
        // that used a different rule (or none) does **not** reset the named rule's timeline. Without
        // this, an era that resumes a rule whose last change was years earlier is wrongly seeded
        // with *standard* state. Forced by real zones:
        //   • America/Phoenix — the 1944-04-01 `-7 US M%sT` era must start in War time (MWT): Rule
        //     US set War (+1) in **1942** and does not clear it until 1945.
        //   • Atlantic/Bermuda — the 1930 `-4 Be A%sT` era renders `AST`, not `AT`: Rule Be's last
        //     pre-1930 activation (**1918**) carries `LETTER = S`.
        //   • Asia/Manila (DST from **1941**), Europe/Brussels (DST from **1940**) — likewise.
        // Exact-or-fail: scan the rule set's *actual* FROM range (not an arbitrary `start_year-1`
        // window) up to the era start and seed from the latest activation at-or-before it, carrying
        // `save` + `letter` together via the same w/s/u `convert_at` the walk below uses. If no
        // activation precedes the era start, the standard seed above stands.
        if !is_fixed {
            if let Some(s) = start_ut {
                let from_lo = rules
                    .iter()
                    .map(|r| r.from)
                    .min()
                    .unwrap_or_else(|| year_of_unix(s));
                for act in &expand_activations(zone, rules, from_lo, year_of_unix(s) + 1)? {
                    let ut = convert_at(act.local_seconds, stdoff, run.save, act.at_ref);
                    if ut <= s {
                        run = act.into();
                    } else {
                        break;
                    }
                }
            }
        }

        // --- Final recurring era: anchor + footer, no explicit per-year expansion. ---
        //
        // A continuation era whose rule set recurs (`TO=maximum`) is represented by `zic` as a
        // single anchor transition at the era start plus the recurring POSIX footer; `zdump`
        // then projects the recurrence forward from that anchor. (Pinned by experiment — see
        // docs/reference-zic-semantics.md: reference `Test/FR`/`Test/Q` each have exactly ONE
        // explicit transition, and DST appears from the era start even when the rule `FROM` is
        // later.) We therefore emit the era-start anchor — **forced**, never de-duplicated,
        // because it is precisely what bounds where the footer begins projecting; without it
        // `zdump` would project the recurrence backwards over all earlier time — and stop.
        if era.until.is_none() && recurring {
            // Classify the final era by its **effective in-era activations**, not by raw
            // membership. A finite rule (`TO = <year>`) whose whole span ends before this era
            // begins never fires *inside* the era — it governed an earlier era and is already
            // accounted for there. Two shapes result:
            //
            //   (a) effectively **recurring-only** — no finite rule fires in or after the era
            //       start. `zic` represents this as a single era-start anchor + the recurring
            //       footer (pinned: `Test/FR`/`Test/Q` have exactly ONE explicit transition, and
            //       DST projects from the era start even when the rule `FROM` is later). We emit
            //       that anchor — **forced**, never de-duplicated — and the footer, and return.
            //       (Europe/London: final EU era from 1996, Rule E's finite rows end by 1995.)
            //
            //   (b) genuinely **mixed-in-era** — finite rules *do* still fire inside the era,
            //       alongside the recurring ones (America/New_York: final era from 1967, Rule US
            //       has finite DST 1967..2006 + recurring 2007..max). We fall through to the
            //       general expansion below, which emits the finite history explicitly and the
            //       recurring tail through `RECUR_HI`, then builds the recurring footer from the
            //       perpetual rows — identical to the single-era mixed case (`Test/Mixed`), and
            //       zdump-equivalent to `zic` (the explicit transitions cover the era from its
            //       start, so there is no FROM-after-start gap that would need the (a) anchor).
            //
            // PRECISION NOTE: the classification is *year-level* — a finite rule counts as
            // "in-era" if its `TO` year is >= the era's start year. This is coarse but safe in
            // the only way that matters: it can mis-route a borderline finite rule (one ending
            // in the era's start year but whose last activation precedes the boundary) from (a)
            // to (b), and path (b) still produces correct output — it just emits one more
            // explicit transition than the minimal (a) form. It never admits an unsupported
            // shape. A future instant-level classifier could tighten the (a)/(b) split.
            let era_start_year = start_ut.map(year_of_unix);
            let finite_active_in_era = rules.iter().any(|r| match r.to {
                YearBound::Year(y) => match era_start_year {
                    Some(sy) => y >= sy,
                    None => true, // era starts at -infinity: every finite row is in-era
                },
                YearBound::Max => false,
            });
            // Shape (a): an effectively recurring-only final era → a single era-start anchor + the
            // recurring footer (its tail lives in the footer, not as explicit transitions). We keep
            // this path under `-r` too — it correctly preserves the prior/interior-era transitions
            // (e.g. Antarctica/Macquarie's 2010–11 inline-save one-off) and the boundary anchor — and
            // *additionally* expand **this era's own** recurring tail forward when `-r` is set, so
            // range truncation has the explicit transitions it needs (T10.4f). An earlier attempt
            // routed `-r` through the general path (b); that collapsed the slim take-over onto the
            // footer and dropped the one-off-era transitions (footer then wrongly projected a 2010
            // fall-back). Expanding only this era avoids re-projecting any other era.
            if !finite_active_in_era {
                // Shape (a): effectively recurring-only → forced era-start anchor + footer.
                if let Some(s) = start_ut {
                    // Seed the boundary state from the latest rule activation at-or-before the
                    // era start — the same `ut <= s` rule the general path uses (lines below).
                    // This is load-bearing when a recurring rule fires *exactly* at the era
                    // boundary: Europe/Lisbon's final era `0 E WE%sT` begins 1996-03-31 01:00u,
                    // which is precisely Rule E's last-Sunday-of-March spring-forward, so the
                    // anchor must be WEST (isdst, +1h), not the era's standard WET. Without this
                    // the anchor seeds standard time and `zdump` shows WET at the boundary while
                    // reference `zic` shows WEST. (Europe/London's shape-(a) era starts in winter,
                    // so its anchor is standard either way — which is why this surfaced only now.)
                    let sy = year_of_unix(s);
                    for act in &expand_activations(zone, rules, sy - 1, sy + 1)? {
                        let ut = convert_at(act.local_seconds, stdoff, run.save, act.at_ref);
                        if ut <= s {
                            run = act.into();
                        } else {
                            break;
                        }
                    }
                    if s > last_ut {
                        let ty = era_type(stdoff, &run, &era.format);
                        let idx = intern(&mut types, ty);
                        transitions.push(Transition {
                            at: s,
                            type_index: idx,
                        });
                        from_recurring.push(false); // the anchor itself is the footer take-over point
                                                    // Keep `prevailing`/`last_ut` consistent with the anchor so the optional `-r`
                                                    // forward expansion below de-dups/orders against it correctly. Harmless for the
                                                    // default path (it builds the footer and returns immediately).
                        prevailing = idx;
                        last_ut = s;
                    }
                }
                // Default build: a single anchor + footer is *already slim*, so no
                // `finalize_emit_style` truncation is needed — every emit style returns the same set.
                let (mut footer, version) = build_recurring_footer(zone, era, stdoff, rules)?;
                // Under `-r` only: expand **this era's own** recurring rules forward from the anchor
                // so range truncation can read the prevailing type at any `lo` in the recurring tail
                // and the slim take-over stays anchored on the era boundary. Prior/interior eras are
                // never touched. Then shape (range before slim, as everywhere). The no-`-r` output
                // above is unchanged → CORE.1 intact (T10.4f).
                if let Some(range) = opts.range {
                    if let Some(s) = start_ut {
                        let sy = year_of_unix(s);
                        for act in &expand_activations(zone, rules, sy - 1, RECUR_HI)? {
                            let ut = convert_at(act.local_seconds, stdoff, run.save, act.at_ref);
                            // Activations at/before the anchor re-seed the run state (the anchor
                            // already represents `s`); only those strictly after `s` are emitted.
                            run = act.into();
                            if ut <= s {
                                continue;
                            }
                            let ty = era_type(stdoff, &run, &era.format);
                            if push(
                                &mut types,
                                &mut transitions,
                                &mut prevailing,
                                &mut last_ut,
                                ut,
                                ty,
                            ) {
                                from_recurring.push(true);
                            }
                        }
                    }
                    apply_range_to_stream(
                        &mut types,
                        &mut transitions,
                        &mut from_recurring,
                        &mut footer,
                        range,
                    );
                    finalize_emit_style(opts, &mut types, &mut transitions, &from_recurring);
                }
                ensure_strictly_increasing(&transitions, zone)?;
                return Ok(TzifData {
                    types,
                    transitions,
                    footer,
                    version,
                    leaps: Vec::new(),
                });
            }
            // Shape (b): fall through to the general expansion path below.
        }

        // `UNTIL`, resolved to a naive-local second count + clock reference (if present).
        let until_local = match &era.until {
            Some(u) => Some(until_local_seconds(zone, u)?),
            None => None,
        };

        // Year span to expand: from the era's start year (a year of margin) to its end. For a
        // recurring final era, expand through RECUR_HI and let the footer cover the tail.
        let lo = match start_ut {
            Some(s) => year_of_unix(s) - 1,
            None => rules.iter().map(|r| r.from).min().unwrap_or(0),
        };
        // The max year among *finite* (`TO = <year>`) rows. One-shot rows can extend far past
        // `RECUR_HI` and are **not** representable by the perpetual footer (e.g. Asia/Gaza's
        // Ramadan-dated `R P … o` rows out to 2086 vs the footer's perpetual `Sat<=30`), so they
        // must be emitted explicitly; the footer covers only the tail after the last finite row.
        let last_finite = rules.iter().filter_map(|r| match r.to {
            YearBound::Year(y) => Some(y),
            YearBound::Max => None,
        });
        let hi = match &until_local {
            Some((ul, _)) => year_of_unix(*ul) + 1,
            None if recurring => RECUR_HI.max(last_finite.max().map_or(RECUR_HI, |y| y + 1)),
            None => last_finite
                .max()
                .unwrap_or_else(|| year_of_unix(start_ut.unwrap_or(0))),
        };
        let acts = if is_fixed {
            Vec::new()
        } else {
            expand_activations(zone, rules, lo, hi)?
        };

        // Walk this era's activations, emitting transitions within `[start_ut, until_ut)`.
        // `boundary_done` tracks whether the era-start transition has been emitted (the first
        // era needs none — its initial type already covers all prior time).
        let mut boundary_done = start_ut.is_none();
        for act in &acts {
            // UT instant uses the `save` prevailing *just before* this activation.
            let ut = convert_at(act.local_seconds, stdoff, run.save, act.at_ref);
            // Stop at the era end: an activation at-or-after the era-end *instant* belongs to the
            // next era. T5 #5 — compare in **UT**, not naive local seconds, because the `UNTIL` and
            // the rule `AT` may use different w/s/u clock references. Europe/Simferopol ends
            // `2 E EE%sT 2014 Mar 30 2` (wall) while Rule E springs `Mar lastSu 1u` (universal):
            // naively `01:00 < 02:00` keeps the spring, but in UT the spring (00:00→01:00u) is past
            // the boundary (Mar 30 00:00 UT) and belongs to the next era. For a *same-reference* era
            // this is algebraically identical to the old `act.local_seconds >= ul` (the stdoff+save
            // cancel), so only mixed-reference boundaries change. Breaking here (before `run` is
            // advanced) also keeps the prevailing save for the `UNTIL` conversion correct when a
            // rule transition coincides with the boundary — Europe/Warsaw's Rule c fall-back
            // (`S M>=15 2s` = wall 03:00 while DST) lands exactly on `1 c CE%sT 1918 S 16 3`.
            if let Some((ul, uref)) = until_local {
                if ut >= convert_at(ul, stdoff, run.save, uref) {
                    break;
                }
            }

            if let Some(s) = start_ut {
                // An activation that lands *at* the era boundary seeds the boundary state rather
                // than being emitted as a separate transition. Two ways it can land at `s`:
                //
                //   (i)  `ut <= s` — its converted UT instant is at-or-before the boundary. The
                //        `<= s` (not `< s`) is load-bearing for a rule that activates *exactly* at
                //        the boundary in UT — e.g. New Zealand's `1946 Jan 1 0 0 S` rule lands on
                //        the +11:30→+12:00 boundary and flips the standard LETTER `M`→`S`
                //        (NZMT→NZST); it must seed the boundary, not be dropped behind it. (`%s`/the
                //        standard LETTER is *temporal* state: a `SAVE=0` rule can change the
                //        standard abbreviation.)
                //
                //   (ii) it is *clock-coincident* with the boundary: it names the **same civil
                //        instant** as the previous era's `UNTIL`. This is needed when the new era's
                //        standard offset is *smaller* than the old era's, so the activation's `ut` —
                //        computed above with the new (smaller) stdoff — lands AFTER `s` (computed at
                //        the prior era's UNTIL with the old, larger stdoff) even though both name the
                //        *same instant*. Example: Europe/Moscow 1991 — era ends `3 R MSK/MSD 1991
                //        Mar 31 2s`, the next era `2 R EE%sT` springs `lastSun Mar 02:00s`; both are
                //        "Mar 31 02:00 standard", so reference `zic` makes the boundary itself the
                //        spring (→ EEST, isdst=1), not a standard boundary + a separate spring an hour
                //        later. T5 #5 generalises the test: the `UNTIL` and the rule `AT` may use
                //        *different* w/s/u references that still resolve to the same instant under the
                //        prevailing save — Asia/Tashkent ends `…1991 Mar 31 2` (**wall**) while Rule R
                //        springs `Mar lastSu 2s` (**standard**), and at the boundary `save == 0`, so
                //        wall 02:00 ≡ standard 02:00. We therefore require the same local seconds AND
                //        that the two references resolve to the **same UT** under the prevailing
                //        `(stdoff, save)` — *exact* equality after deterministic normalisation, never
                //        a "within N minutes" tolerance (which would absorb ordinary transitions).
                let boundary_coincident = match boundary_until {
                    Some((bl, br)) => {
                        act.local_seconds == bl
                            && convert_at(bl, stdoff, run.save, br)
                                == convert_at(bl, stdoff, run.save, act.at_ref)
                    }
                    None => false,
                };
                if ut <= s || boundary_coincident {
                    run = act.into();
                    continue;
                }
                if !boundary_done {
                    // Trap #2/#5: emit the boundary transition at the era start using the
                    // state as of `start_ut` (deduped if it is a no-op vs. the prevailing).
                    let ty = era_type(stdoff, &run, &era.format);
                    if push(
                        &mut types,
                        &mut transitions,
                        &mut prevailing,
                        &mut last_ut,
                        s,
                        ty,
                    ) {
                        from_recurring.push(false); // era boundary — not footer-governed
                    }
                    boundary_done = true;
                }
            }
            run = act.into();
            let ty = era_type(stdoff, &run, &era.format);
            if push(
                &mut types,
                &mut transitions,
                &mut prevailing,
                &mut last_ut,
                ut,
                ty,
            ) {
                from_recurring.push(act.from_recurring);
            }
        }
        // A boundary with no in-range activations (a `-` era, or a ruled era whose rules all
        // fall outside the window) still needs its era-start transition.
        if let Some(s) = start_ut {
            if !boundary_done {
                let ty = era_type(stdoff, &run, &era.format);
                if push(
                    &mut types,
                    &mut transitions,
                    &mut prevailing,
                    &mut last_ut,
                    s,
                    ty,
                ) {
                    from_recurring.push(false); // era boundary — not footer-governed
                }
            }
        }

        match until_local {
            // Trap #1: convert UNTIL with this era's stdoff and the save now prevailing.
            // Also remember the *unconverted* boundary (naive local + ref) so the next era can
            // detect a clock-coincident activation despite an offset change (absorption case (ii)).
            Some((ul, uref)) => {
                start_ut = Some(convert_at(ul, stdoff, run.save, uref));
                boundary_until = Some((ul, uref));
            }
            None => {
                // Trap #4: the final era owns the footer. A recurring final era gets the exact
                // recurring POSIX string; any other final era (literal, inline-save, or a
                // finite ruled set settled into its last state) gets a fixed footer from the
                // *prevailing* type — which already carries the era's effective offset and
                // abbreviation (for inline-save, `stdoff + save` rendered via `%z`/literal).
                let (mut footer, version) = if recurring {
                    match build_recurring_footer(zone, era, stdoff, rules) {
                        Ok(fv) => fv,
                        // PERPETUAL-EXPANSION.1 (shape-b / mixed-in-era): the general path above
                        // already expanded explicit transitions through `RECUR_HI`. If the recurring
                        // tail is not POSIX-footer-expressible (perpetual year-parity) and
                        // `--legacy-empty-footer` is set, emit an **empty footer** (frozen beyond)
                        // rather than failing closed. Off by default → footer-synthesising zones never
                        // reach here.
                        Err(_) if opts.allow_empty_footer_on_legacy_nonposix_recurrence => {
                            (String::new(), b'2')
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    let t = &types[prevailing as usize];
                    (posix_footer::fixed_offset(&t.abbr, t.utoff), b'2')
                };
                // T10.4d: range (`-r`) shapes the fat stream **before** slim (see compile_rule_zone).
                if let Some(range) = opts.range {
                    apply_range_to_stream(
                        &mut types,
                        &mut transitions,
                        &mut from_recurring,
                        &mut footer,
                        range,
                    );
                }
                // T8-slim: optionally truncate the footer-governed recurring tail (no-op default).
                finalize_emit_style(opts, &mut types, &mut transitions, &from_recurring);
                ensure_strictly_increasing(&transitions, zone)?;
                return Ok(TzifData {
                    types,
                    transitions,
                    footer,
                    version,
                    leaps: Vec::new(),
                });
            }
        }
    }

    // The parser guarantees the last era has no UNTIL, so the loop returns above.
    Err(Error::message(
        "multi-era zone has no terminating era (internal invariant violated)",
    ))
}

impl From<&Activation> for RunState {
    fn from(a: &Activation) -> Self {
        RunState {
            save: a.save,
            is_dst: a.is_dst,
            letter: a.letter.clone(),
        }
    }
}

/// What an era's `RULES` field denotes.
///
/// * `Literal` — `RULES = -`: a fixed-offset era with a literal abbreviation (no DST).
/// * `InlineSave` — `RULES = <clock value>` (e.g. `0:30`): a fixed era at `STDOFF + SAVE` with a
///   constant saving (real tzdata: Hong Kong/India wartime). Carries the parsed [`Save`] (whose
///   `is_dst` already encodes the `s`/`d` suffix or the no-suffix `seconds != 0` heuristic).
/// * `Ruled` — `RULES = <name>`: a named rule set, expanded across its years.
enum EraKind {
    Literal,
    InlineSave(Save),
    Ruled,
}

/// Resolve an era's `RULES` field to its rule slice (empty for the fixed kinds) and its
/// [`EraKind`]. Fails closed only on an unknown rule-set name; inline savings are now supported.
fn resolve_era_rules<'a>(
    zone: &ZoneRecord,
    db: &'a Database,
    era: &ZoneEra,
) -> Result<(&'a [RuleRecord], EraKind)> {
    match &era.rules {
        ZoneRules::None => Ok((&[], EraKind::Literal)),
        ZoneRules::Save(save) => Ok((&[], EraKind::InlineSave(*save))),
        ZoneRules::Named(name) => {
            let rules = db
                .rules
                .get(name)
                .ok_or_else(|| unsupported(zone, format!("unknown rule set {name:?}")))?;
            Ok((rules.as_slice(), EraKind::Ruled))
        }
    }
}

/// The seed running state for an era before any rule activation is applied: a literal era starts
/// in standard time with an empty letter; an inline-save era starts at the constant saving (and
/// its daylight flag); a ruled era starts in standard time with the rule set's standard LETTER.
fn seed_run(kind: &EraKind, rules: &[RuleRecord]) -> RunState {
    match kind {
        EraKind::Literal => RunState {
            save: 0,
            is_dst: false,
            letter: String::new(),
        },
        EraKind::InlineSave(s) => RunState {
            save: s.seconds,
            is_dst: s.is_dst,
            letter: String::new(),
        },
        EraKind::Ruled => RunState {
            save: 0,
            is_dst: false,
            letter: standard_letter(rules),
        },
    }
}

/// Validate the `FORMAT` (and, for inline save, the save value) of a *fixed* era — one with no
/// rule context to supply a `%s` LETTER. Fails closed on the constructs not yet pinned against
/// reference `zic`. (A `Ruled` era needs no check; its rules provide the abbreviation context.)
fn validate_fixed_era_format(zone: &ZoneRecord, era: &ZoneEra, kind: &EraKind) -> Result<()> {
    let fmt = &era.format;
    match kind {
        EraKind::Literal => {
            // `%z` is allowed on a no-rules era (renders the era's numeric standard offset, as
            // reference `zic` does); `%s` (no LETTER) and the `STD/DST` slash form (no DST
            // context) are not. The `Literal` `era_type` path renders through `render`, so once
            // validated here a `%z` FORMAT produces the numeric abbreviation automatically.
            if fmt.contains("%s") || fmt.contains('/') {
                return Err(unsupported(
                    zone,
                    format!("FORMAT {fmt:?}: %s / STD-DST slash need rule context but the era has no rules"),
                ));
            }
        }
        EraKind::InlineSave(_s) => {
            // Inline save supports literal and `%z` FORMAT (pinned: Hong Kong `HKWT`, Jakarta
            // `%z`→`+0720`). `SAVE` is **signed**: a *negative* inline save is valid — the effective
            // offset is `STDOFF + SAVE` and `is_dst = (seconds != 0)`, exactly as reference `zic`
            // renders Europe/Prague's `1 -1 GMT` era (→ `GMT`, isdst=1, gmtoff 0). `%s` has no
            // LETTER; the `STD/DST` slash form is still not pinned. (Law 7 — signed SAVE.)
            if fmt.contains("%s") {
                return Err(unsupported(
                    zone,
                    format!("inline-save FORMAT {fmt:?}: %s has no LETTER in an inline-save era"),
                ));
            }
            if fmt.contains('/') {
                return Err(unsupported(
                    zone,
                    format!("inline-save FORMAT {fmt:?}: STD/DST slash form is not supported yet"),
                ));
            }
        }
        EraKind::Ruled => {}
    }
    Ok(())
}

/// Build the local-time-type for an era given its running state. The abbreviation is rendered
/// from `FORMAT` via [`render`], which handles literal text, `%z` (from the *total* offset
/// `stdoff + save`), `%s` (LETTER substitution) and the `STD/DST` slash form uniformly.
fn era_type(stdoff: i32, run: &RunState, format: &str) -> LocalTimeType {
    let utoff = stdoff + run.save;
    LocalTimeType {
        utoff,
        is_dst: run.is_dst,
        abbr: render(format, &run.letter, run.is_dst, utoff),
    }
}

/// Convert a naive-local second count to a UT instant for the given clock reference.
fn convert_at(local: i64, stdoff: i32, save: i32, reference: TimeRef) -> i64 {
    match reference {
        TimeRef::Wall => local - stdoff as i64 - save as i64,
        TimeRef::Standard => local - stdoff as i64,
        TimeRef::Universal => local,
    }
}

/// Resolve an `UNTIL` field to `(naive_local_seconds, clock_reference)`.
fn until_local_seconds(zone: &ZoneRecord, until: &Until) -> Result<(i64, TimeRef)> {
    let day = resolve_on_day(until.day, until.year, until.month)
        .map_err(|(code, msg)| located(zone, code, msg))?;
    let local =
        days_from_civil(day.year, day.month, day.day as u8) * 86400 + until.time.seconds as i64;
    Ok((local, until.time.reference))
}

/// Push a transition with de-duplication and a strict-monotonicity guard. Skips a transition
/// that would switch to the already-prevailing type (a no-op) or that is not strictly later
/// than the previous one (defensive; well-formed input does not trigger it).
/// Append a transition unless it is a no-op or out of order. Returns `true` iff a transition was
/// actually appended — callers tracking a parallel attribute (the T8-slim `from_recurring` flag)
/// use this to stay in exact lockstep with `transitions`.
fn push(
    types: &mut Vec<LocalTimeType>,
    transitions: &mut Vec<Transition>,
    prevailing: &mut u8,
    last_ut: &mut i64,
    at: i64,
    ty: LocalTimeType,
) -> bool {
    let idx = intern(types, ty);
    if idx == *prevailing {
        return false; // no-op: same local-time behaviour as before this instant
    }
    if at <= *last_ut {
        return false; // keep transitions strictly increasing
    }
    transitions.push(Transition {
        at,
        type_index: idx,
    });
    *prevailing = idx;
    *last_ut = at;
    true
}

/// Apply the [`crate::EmitStyle`] to a fully-built `(types, transitions)` pair, in lockstep with a
/// `from_recurring` flag per transition. **A no-op for [`crate::EmitStyle::Default`]/`ZicFat`** — the
/// behaviour-matched output is returned untouched, so CORE.1 is structurally unaffected.
///
/// For [`crate::EmitStyle::ZicSlim`] it reproduces `zic.c`'s slim truncation: keep only transitions
/// with `at <= TZstarttime`, where `TZstarttime` is the first transition past `nonTZlimtime`, and
/// `nonTZlimtime` is the latest transition **not** governed by the footer (i.e. from an era boundary
/// or a finite rule — `!from_recurring`). The footer reproduces everything dropped. Unreferenced
/// local-time types are then pruned so `typecnt` also matches slim `zic`.
fn finalize_emit_style(
    opts: crate::EmitOptions,
    types: &mut Vec<LocalTimeType>,
    transitions: &mut Vec<Transition>,
    from_recurring: &[bool],
) {
    if !matches!(opts.style, crate::EmitStyle::ZicSlim) {
        return;
    }
    debug_assert_eq!(transitions.len(), from_recurring.len());
    // nonTZlimtime: the latest transition the footer does NOT govern (boundary / finite rule).
    let non_tz = transitions
        .iter()
        .zip(from_recurring)
        .filter(|(_, &rec)| !rec)
        .map(|(t, _)| t.at)
        .max();
    // TZstarttime: the first transition the footer DOES take over. If every transition is
    // footer-governed (a pure recurring zone), keep just the first as the anchor.
    let tz_start = match non_tz {
        Some(n) => transitions.iter().map(|t| t.at).filter(|&a| a > n).min(),
        None => transitions.first().map(|t| t.at),
    };
    let Some(mut keep_at_max) = tz_start else {
        return; // nothing past the last non-footer transition → already slim
    };
    // T10.3 (`-R @hi`): keep the otherwise-droppable, footer-governed explicit transitions out to
    // `redundant_until` as well — `zic`'s `keep_at_max = max(TZstarttime, redundant_time)`. These are
    // *redundant* with the footer (identical offsets/abbreviations), so behaviour is unchanged; they
    // only help readers that ignore the footer. Independent of slim/fat — it only widens what slim
    // keeps, never narrows, and capping at what we expanded (`RECUR_HI`) is harmless.
    if let Some(hi) = opts.redundant_until {
        keep_at_max = keep_at_max.max(hi);
    }
    let before = transitions.len();
    transitions.retain(|t| t.at <= keep_at_max);
    if transitions.len() != before {
        prune_types(types, transitions);
    }
}

/// The synthetic `-00` "local time **unspecified**" type (reference `zic`'s
/// `addtype(0, "-00", false, false, false)`): UT offset 0, not DST. It is **distinct** from any real
/// offset-0 type (e.g. `UTC`) because its abbreviation is `-00` — the invariant that `-00` never
/// masquerades as a real offset (T10.4d).
fn unspecified_type() -> LocalTimeType {
    LocalTimeType {
        utoff: 0,
        is_dst: false,
        abbr: "-00".to_string(),
    }
}

/// Apply reference `zic`'s `-r '[@lo][/@hi]'` range truncation to a **fully-built** `TzifData`
/// (T10.4d). This is a deliberate **post-emission shaper**: the compiler emits the same semantic
/// transition stream first (and any `-b`/`-R` shaping has already run); this pass only reshapes the
/// *representable interval*. It is independent of `-R` (a different clamp input) and never merges with
/// it. See `docs/range-truncation-microcases.md` for the pinned ground truth.
///
/// Rules (each pinned by a reference fixture):
/// * **start cut (`lo`)** — the leading default becomes a synthetic `-00` (type 0); a transition at
///   *exactly* `lo` switches to the **prevailing real type at `lo`** (the type in effect *at* `lo` —
///   the last transition `<= lo`, or the default — **not** the first transition after `lo`).
/// * **end cut (`hi`)** — a transition at *exactly* the parsed `@hi` switches **back** to `-00`, and
///   the **footer is cleared** (no proleptic rule past a bounded end). An unbounded end keeps the
///   footer.
/// * unreferenced types are pruned (matching reference `typecnt`); the TZif version is left
///   content-driven (unchanged here).
pub(crate) fn apply_range(data: &mut TzifData, range: crate::RangeSpec) {
    // Fixed-offset / inline-save zones have no footer-governed recurring tail, so `from_recurring`
    // is all-false and the later slim pass is a no-op — a throwaway vector suffices.
    let mut from_recurring = vec![false; data.transitions.len()];
    apply_range_to_stream(
        &mut data.types,
        &mut data.transitions,
        &mut from_recurring,
        &mut data.footer,
        range,
    );
}

/// Core range shaper, operating on the raw stream so it can run **on the fat stream before slim**
/// (the rule/era paths) *and* on a finished [`TzifData`] (fixed/inline paths). It reshapes
/// `from_recurring` in lockstep with `transitions` so the subsequent slim pass stays correct.
fn apply_range_to_stream(
    types: &mut Vec<LocalTimeType>,
    transitions: &mut Vec<Transition>,
    from_recurring: &mut Vec<bool>,
    footer: &mut String,
    range: crate::RangeSpec,
) {
    debug_assert_eq!(transitions.len(), from_recurring.len());

    // --- start truncation (locut): leading `-00` (type 0) + a boundary transition at `lo`. ---
    if let Some(lo) = range.lo {
        // Prevailing type *at* lo — last transition with `at <= lo`, else the default (type 0).
        // NOT the first transition after `lo`.
        let prevailing_old = transitions
            .iter()
            .rev()
            .find(|t| t.at <= lo)
            .map(|t| t.type_index)
            .unwrap_or(0);
        // Make `-00` the new type 0 (the before-first-transition default); shift type indices +1.
        let mut new_types = Vec::with_capacity(types.len() + 1);
        new_types.push(unspecified_type());
        new_types.append(types);
        *types = new_types;
        let prevailing = prevailing_old + 1;

        let mut kept_t = vec![Transition {
            at: lo,
            type_index: prevailing,
        }];
        let mut kept_fr = vec![false]; // the synthetic boundary is not footer-governed
        for (t, &fr) in transitions.iter().zip(from_recurring.iter()) {
            if t.at > lo {
                kept_t.push(Transition {
                    at: t.at,
                    type_index: t.type_index + 1,
                });
                kept_fr.push(fr);
            }
        }
        *transitions = kept_t;
        *from_recurring = kept_fr;
    }

    // --- end truncation (hicut): boundary transition at `hi` back to `-00`, footer cleared. ---
    if let Some(hi) = range.hi {
        let unspec_idx = if range.lo.is_some() {
            0u8 // `-00` is already type 0 from the start-cut above
        } else {
            types.push(unspecified_type());
            (types.len() - 1) as u8
        };
        let mut kept_t = Vec::new();
        let mut kept_fr = Vec::new();
        for (t, &fr) in transitions.iter().zip(from_recurring.iter()) {
            if t.at < hi {
                kept_t.push(*t);
                kept_fr.push(fr);
            }
        }
        kept_t.push(Transition {
            at: hi,
            type_index: unspec_idx,
        });
        kept_fr.push(false);
        *transitions = kept_t;
        *from_recurring = kept_fr;
        footer.clear();
    }

    if range.lo.is_some() || range.hi.is_some() {
        // `prune_types` only drops/remaps *types* (transition count unchanged), so `from_recurring`
        // stays aligned.
        prune_types(types, transitions);
        debug_assert!(
            types.iter().any(|t| t.abbr == "-00"),
            "range truncation must retain the `-00` unspecified type"
        );
    }
}

/// Drop local-time types no longer referenced by any transition (after a slim truncation),
/// remapping `type_index` and preserving order. Type 0 (the pre-first-transition / default type) is
/// always kept — readers use it before the first transition.
fn prune_types(types: &mut Vec<LocalTimeType>, transitions: &mut [Transition]) {
    let mut used = vec![false; types.len()];
    if !used.is_empty() {
        used[0] = true;
    }
    for t in transitions.iter() {
        used[t.type_index as usize] = true;
    }
    let mut remap = vec![0u8; types.len()];
    let mut kept: Vec<LocalTimeType> = Vec::new();
    for (i, ty) in types.iter().enumerate() {
        if used[i] {
            remap[i] = kept.len() as u8;
            kept.push(ty.clone());
        }
    }
    for t in transitions.iter_mut() {
        t.type_index = remap[t.type_index as usize];
    }
    *types = kept;
}

fn unsupported(zone: &ZoneRecord, msg: impl Into<String>) -> Error {
    located(zone, DiagnosticCode::UnsupportedDirective, msg)
}

fn located(zone: &ZoneRecord, code: DiagnosticCode, msg: impl Into<String>) -> Error {
    Error::from(Diagnostic::error(
        code,
        msg,
        &zone.origin.file,
        zone.origin.line,
    ))
}
