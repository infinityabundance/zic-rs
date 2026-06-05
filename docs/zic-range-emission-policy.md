# Range & emission policy — reference `zic`'s `-b`, `-R`, `-r` (campaign T10)

> **Three knobs that look related but are not.** The single most important thing in T10 is to **not**
> let `-b`, `-R`, and `-r` collapse into one "emission options" blob. They are three separate
> contracts with three separate mechanisms. This doc pins each from `zic.c` (tzcode 2026b) and tracks
> zic-rs's status against it. None of these knobs is the **TZif version** (which is content-driven,
> T8-v3) and none changes canonical-zone **behaviour** within the declared horizon (CORE.1).

Status legend: **✅ implemented** · **◐ partial** · **▷ deferred (milestone)**.

## The three mechanisms, pinned from `zic.c`

| Knob | What it controls | `zic.c` mechanism | zic-rs |
|------|------------------|-------------------|--------|
| **`-b {slim\|fat}`** | **emission bloat** — whether otherwise-redundant transitions are *kept* (`fat`) or *dropped* (`slim`) | `bloat = -1` (slim) / `+1` (fat); `want_bloat() = 0 <= bloat`; `ZIC_BLOAT_DEFAULT = "slim"`. Drives whether pass-1 explicit transitions are emitted. | ✅ **T10.2** — `--bloat`/`-b`, an alias onto `--emit-style` |
| **`-R @hi`** | **redundant tail** — keep transitions that slim would drop, out to `@hi`, for *old readers* | `redundant_time = max(redundant_time, @hi)`; widens `limitrange(... max(hi_time, redundant_time-…))`. **Distinct from `-b`.** | ✅ **T10.3** — `--redundant-until` |
| **`-r @lo/@hi`** | **range truncation** — restrict emitted timestamps to `[lo, hi]` (Unix seconds) | `timerange_option`: `lo_time = max(lo, min)`, `hi_time = min(hi, max)`, `hi -= 1`; validates `hi < lo`. Truncated-away leading type → the **`-00`** "local time **unspecified**" placeholder. | ✅ **T10.4** — `--range`/`-r` (all three declared profiles **341/341**; T10.4f cleared the open-ended residual) |

### Why they must stay separate

- `-b` is about **redundancy**, `-r` is about **range**, `-R` is about **how far redundancy is kept
  for compatibility**. `zic` even composes them: `limitrange(rangeall, lo_time, max(hi_time,
  redundant_time - (ZIC_MIN < redundant_time)))` — `hi_time` (from `-r`) and `redundant_time` (from
  `-R`) are *different inputs* to the same clamp.
- **`-b` ≠ TZif version.** Slim/fat is the explicit-transition count; the version byte (`2`/`3`) is
  content-driven (T8-v3). A slim file can still be v3 and vice-versa.
- **`-r`'s `-00` is not UTC.** The placeholder means *local time is unspecified before the range*, not
  "UTC offset zero". Getting this wrong produces output that looks plausible and is wrong — which is
  why `-r` is implemented **last** (T10.4), after the lower-risk knobs.

## T10.2 — `-b {slim|fat}` ✅ (alias onto `--emit-style`)

zic-rs already built the slim/fat emission machinery in **T8-slim** (`EmitStyle {Default|ZicSlim|ZicFat}`;
`--emit-style`). T10.2 adds the reference-`zic` spelling as a **thin alias**, no new emission logic:

- **`--bloat <slim|fat>` / `-b`** → `slim` maps to `--emit-style zic-slim`, `fat` to `zic-fat`.
  `-b slim` produces **byte-identical** output to `--emit-style zic-slim` (pinned by test).
- **Reconciliation.** `--emit-style` is the native surface; `-b` is the alias. When only one is given
  it decides. When **both** are given they must agree — a conflicting pair (e.g. `-b fat
  --emit-style zic-slim`) is a config error (exit 1, nothing written), mirroring `zic`'s own
  *"incompatible -b options"* check. `-b` against a left-at-`default` `--emit-style` simply wins.
- **Default unchanged.** `zic`'s default is `slim`; zic-rs keeps a **behaviour-matched fat-style
  default** and only changes emission when asked — this is recorded as an *intentional divergence in
  default*, not a behaviour difference (both styles are behaviour `341/0/0`).

Classification (four-bucket): `--emit-style zic-slim` / `-b slim` is an **explicit compatibility
mode**; the **fat-style default** is an **intentional safer/clarity divergence in default** (more
explicit transitions, never wrong); behaviour parity holds in both.

Pinned by `tests/cli_operational_parity.rs::{bloat_alias_matches_emit_style,bloat_conflicts_with_emit_style_exits_one}`
and `src/cli.rs` `cli::tests::{bloat_alias_maps_onto_emit_style,bloat_conflicting_with_emit_style_is_error}`;
the slim-vs-fat *difference* itself is proven over real recurring zones in `tests/emit_style.rs`.
Sweep **341/0/0** (emission policy, not the engine).

## T10.3 — `-R @hi` redundant tail ✅ (`--redundant-until`)

An explicit redundant-tail policy that keeps slim-droppable transitions out to a given `@hi` instant,
**without** touching `-b`. It reuses the T8-slim truncation seam (`finalize_emit_style`) parameterised
by an upper instant: `keep_at_max = max(TZstarttime, redundant_until)` — exactly `zic`'s
`max(TZstarttime, redundant_time)`. It only *widens* what slim keeps; it never narrows, never changes
behaviour (the kept transitions are **redundant** with the footer — identical offsets/abbreviations),
and never the TZif version.

- **`--redundant-until @<seconds>` / `-R`** — the bound is `@`-prefixed Unix seconds (matching `zic`'s
  `redundant_time_option`, which requires the `@`). A missing `@` or non-integer body is a config error
  (exit 1, nothing written); a missing value is a clap usage error (exit 2).
- **Independent of `-b`.** It only affects slim emission — under the fat-style default everything is
  already kept, so `-R` is a no-op there. A `-R` value never implies fat, and `-b fat` never implies
  any `-R`. Threaded as `EmitOptions { style, redundant_until }`; `EmitStyle` converts in with
  `redundant_until: None`, so non-`-R` callers are unchanged.
- **No-op on zones without a footer-governed tail** (fixed-offset zones): slim == slim+`-R`.
- **Interaction with the future `-r` (T10.4):** `zic` clamps to `max(hi_time, redundant_time-…)` — the
  `-r` upper bound and the `-R` redundant bound are *different inputs* to one clamp. When `-r` lands,
  `-R` continues to govern *redundant* retention while `-r` governs the *representable range*; they do
  not merge.

**Receipts.** Default sweep **341/0/0**; `-b slim` sweep **341/0/0**; **`-b slim -R @hi` sweep
341/0/0** (behaviour unchanged — redundant transitions agree with the footer). Library tests prove
slim < slim+`-R`@hi < fat in `timecnt` with footer/version unchanged. Pinned by
`tests/emit_style.rs::{redundant_until_widens_slim_without_reaching_fat,redundant_until_no_op_on_fixed_zone,redundant_until_is_no_op_under_fat_default}`,
`tests/cli_operational_parity.rs::{redundant_without_at_prefix_exits_one,redundant_missing_value_exits_two}`,
and `cli::tests::redundant_until_requires_at_prefix`.

## T10.4 — `-r @lo/@hi` truncation ◐ (last, highest-risk; split into substeps)

Range truncation in Unix seconds with the **`-00` unspecified-local-time** placeholder. This is the
one T10 knob that can silently corrupt *representable* time semantics, so it is split into substeps
and pinned more slowly than the previous three.

### T10.4a — reference pin ✅ (this is the ground truth, do not deviate)

Pinned from `zic.c` (tzcode 2026b):

- **Parse** (`timerange_option`): grammar `[@lo][/@hi]`, each part `@`-prefixed Unix seconds. `lo`
  defaults `min_time`, `hi` defaults `max_time`. An explicit `hi` is decremented (`hi -= 1`) unless it
  overflowed. Rejects `*hi_end` junk, `hi < lo`, `max_time < lo`, `hi < min_time`. Resolved:
  `lo_time = max(lo, min_time)`, `hi_time = min(hi, max_time)`.
- **Clamp** (`limitrange`): drop transitions `< lo_time` — remembering `defaulttype` = the type in
  effect at `lo_time` — and drop transitions `> hi_time + 1`.
- **The `-00` trap** (`unspecifiedtype = addtype(0, "-00", false, false, false)`): when the start is
  truncated (`locut = thismin < lo_time && lo_time <= thismax`), the **leading default type becomes a
  synthetic type with offset 0 and abbreviation `-00`**, meaning *local time is **unspecified*** before
  `lo_time` — **not** "UTC". A `pretrans` transition at `lo_time` switches from `-00` to the first
  real in-range type. Symmetrically, if the end is truncated (`hicut`), a transition at `hi_time + 1`
  switches **back** to the `-00` unspecified type.
- **Empirical anchor** (reference `zic -r @946684800` on `America/New_York`, via `zdump -v`):
  `… Fri Dec 31 23:59:59 1999 UT = … -00 isdst=0 gmtoff=0` then `Sat Jan 1 00:00:00 2000 UT = … EST
  gmtoff=-18000`. Everything before the cut is `-00` (unspecified), not EST. This is the behaviour any
  zic-rs implementation must reproduce; a naïve "drop transitions < lo" that left the real leading
  type would *wrongly* claim the pre-`lo` offset is known.

> **Doctrine for `-r` (explicit):** `-r` is **not** just filtering transitions — it changes the
> *representable interval* and may introduce a **leading (and trailing) `-00` local-time-unspecified
> type**. `-r` (range) and `-R` (redundant retention) both feed `limitrange` but are **different
> inputs to the clamp** (`max(hi_time, redundant_time - …)`); they do not merge.

### T10.4b — parse ✅ (`--range`/`-r`)

`--range <[@lo][/@hi]>` / `-r` parses + validates via `parse_range` (unit-tested): accepts `@lo`,
`@lo/@hi`, and `/@hi`; rejects empty, missing-`@`, non-integer, trailing-junk, and `hi < lo` (config
error, exit 1). Stored as `CompileConfig.range: Option<RangeSpec>` and applied by T10.4d (this was a
fail-closed placeholder during T10.4b; the truncation now lands — see T10.4d/e/f below).

### T10.4c — structural microcases ✅ (reference-pinned; not implementation tests yet)

Reference-pinned 4 small zones from real `zic -r`, capturing exact TZif structure (counts, types incl.
`-00`, transitions, footer, version) + normalized `zdump` — the **ground truth T10.4d must reproduce**.
Raw bytes in [`fixtures/range/reference/`](../fixtures/range/reference/) (hashed); full decode in
[range-truncation-microcases.md](range-truncation-microcases.md). Invariants confirmed:

- **`America/New_York`** `@946684800` — leading `-00`, real `EST` at `lo`, footer `EST5EDT,M3.2.0,M11.1.0`
  **kept** (hi unbounded). v2, timecnt 16, typecnt 3.
- **`Etc/UTC`** `@946684800/@1577836800` — even a fixed zone gains leading **and** trailing `-00`; the
  trailing transition lands at the **parsed `@hi`**; footer **emptied**. v2, timecnt 2.
- **`Europe/London`** `@157766400` — multi-era historical truncation; leading `-00`, real `GMT`/`BST`.
  v2, timecnt 44.
- **`Asia/Gaza`** `@1577836800` — start-truncation composes with the **law-10 v3 footer** (version +
  footer unchanged). v3, timecnt 195.

Started with these microcases, **not** the 341 matrix (that is T10.4e).

### T10.4d — implementation ✅ (a post-emission range *shaper*; range **before** slim)

The compiler still generates the **same semantic transition stream first**; range shaping is a
distinct, final pass over `(types, transitions, footer)`:

1. **Determine the prevailing type at `lo`** — ⚠ the load-bearing step: it is the type *in effect at
   the instant `lo`*, **not** the first transition after `lo` (the previous transition's type carries
   into `lo`). Get the prevailing type, then synthesise the boundary.
2. Insert/retain a transition **at exactly `lo`** from the synthetic `-00` type → that prevailing real
   type. (`-00` becomes type 0 / the default-before-first-transition type.)
3. Drop representational transitions outside `[lo, hi]`.
4. If `hi` is bounded, insert a transition **at the parsed `@hi`** back to `-00` (`zic`'s `hi -= 1`
   then trailing transition at `hi + 1` nets to the parsed `@hi`).
5. **Clear the footer** when `hi` is bounded; **preserve** it when `hi` is unbounded.
6. Keep the TZif **version content-driven** (unchanged by `-r`).
7. **Do not merge with `-R`.** `EmitOptions` gains a **distinct `range` field** —
   `EmitOptions { style, redundant_until, range }` — never overloading `redundant_until`. `-r` and
   `-R` are different inputs to the clamp.

`Rolling`-leap + `-r` is unsupported (matches `zic`); see
[reference-zic-semantics.md](reference-zic-semantics.md).

**Implemented** as `apply_range_to_stream` (`src/compile/transitions.rs`), threaded as a distinct
`EmitOptions.range` field (never overloading `redundant_until`). The **ordering matters**: range runs
on the **fat** stream *before* slim — matching `zic`'s `limitrange`-then-`bloat`. (A first cut ran
slim-then-range and was wrong when `lo` fell past the footer take-over — the prevailing type at `lo`
had been dropped; the summer-`lo` London case caught it.) For "shape (a)" recurring-only final eras
(which normally emit just an anchor), `-r` **forces the full general expansion** so the stream covers
`lo`; this only affects `-r` compiles (default output unchanged → CORE.1 intact).

**Verified:** all five [microcases](range-truncation-microcases.md) are **zdump-identical** to
reference (and structurally identical — counts/footer/version/`-00`); the previously-broken
summer-`lo` corner (London, Lisbon) now matches. Tests: `tests/emit_style.rs::range_*` (mechanics +
the reference-fixture comparison) and `tests/cli_operational_parity.rs::range_*`.

> **Honest structural note (behaviourally null):** under `-r`, the **final** explicit transition may
> sit at a different slim footer-take-over boundary than reference (e.g. London keeps the 1996 spring
> where reference keeps a 1996 anchor). Behaviour is identical (the POSIX footer reproduces the tail;
> zdump matches) and counts match — this is the same slim-boundary residual the project already does
> not claim byte-parity on.

### T10.4e — all-zone gate ✅ (all three declared profiles 341/341)

Default **341/0/0** unchanged (re-verified). `-r` gated over **declared range profiles** (the claim is
scoped to these profiles, never implied for *all possible* ranges):

| Profile | Form | Result |
|---------|------|--------|
| `bounded-2000-2038` | `@946684800/@2145916800` | ✅ **341/341** zdump-identical to reference |
| `post-2000` | `@946684800` (lo-only) | ✅ **341/341** |
| `post-1970` | `@0` (lo-only) | ✅ **341/341** |

### T10.4f — open-ended-`-r` residual: diagnosed and cleared ✅

The earlier open-ended-`-r` residual (`Antarctica/Macquarie` @post-2000; + `Europe/{Bucharest,Riga,
Sofia}` @post-1970) was verified against the pinned source — and the diagnosis overturned the first hypothesis:

- **Not** an expansion leak. The `fat -r @0` stream handled Macquarie's 2010–11 constant-`AEDT` one-off
  *correctly* (no 2010 fall-back). The bug was in the **slim take-over point** under the (then) forced
  shape-(b) path: the redundant era-boundary transition that should anchor the footer take-over got
  de-duped, so slim truncated ~5 transitions too early (last explicit at 2007 vs reference's 2010-12-31),
  and the recurring **footer then wrongly projected a 2010 fall-back**.
- **Fix (smallest, leak-proof by construction):** stop forcing shape-(b) under `-r`; keep the correct
  **shape-(a)** path (which preserves the prior/interior-era transitions and the boundary anchor) and,
  *only* under `-r`, expand **this era's own** recurring rules forward from the anchor
  (`from_recurring=true`) so range truncation can read the prevailing type at any `lo` in the recurring
  tail and the slim take-over stays anchored. No other era is re-projected, so an interior special era
  can never be disturbed. The no-`-r` output is byte-unchanged (gated on `range.is_some()`) → CORE.1
  intact. (`src/compile/transitions.rs`, the shape-(a) branch.)
- **Result:** **all three declared profiles → 341/341**; default sweep **341/0/0**; the 5 microcases
  still match; `Europe/London` summer-`lo` correct. Pinned by a Macquarie open-ended-`-r` regression
  fixture (`fixtures/range/reference/macquarie.lo-1970.tzif`) in
  `tests/emit_style.rs::range_microcases_match_reference_fixtures` (it would have failed pre-fix:
  ours=77 was a *non*-prefix of reference=82).

> **Behaviourally-null structural note (retained):** under `-r`, ours may keep a *few* extra explicit
> transitions at the slim footer-takeover boundary (e.g. `Europe/London` 45 vs reference 44 — one extra
> 1996 spring) — reference's transitions are always an exact **prefix** of ours, behaviour is identical
> (zdump), and the footer reproduces the tail. Same class as the `Europe/Lisbon` slim residual; we do
> not claim `timecnt` byte-parity at that boundary.

## Doctrine

Pin first · keep the three contracts separate · implement narrowly (lowest-risk first: `-b` → `-R` →
`-r`) · classify each into the four buckets · sweep after. No default safety regressions; CORE.1 is
the standing gate. See [differences-from-reference-zic.md](differences-from-reference-zic.md) for the
consolidated map and [zic-operational-parity.md](zic-operational-parity.md) for the CLI flag matrix.
