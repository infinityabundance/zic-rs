# Structural TZif parity inventory (campaign T8)

> **Behaviour parity is the contract; structural parity is a *separate, measured* axis; byte
> parity is claimed only where a reference blob is pinned.** This document measures, zone by zone,
> how zic-rs's emitted TZif *bytes* differ from reference `zic` — the distance a "drop-in file
> replacement" claim would have to close. It does **not** weaken or restate
> [CORE.1](compatibility.md) (341/341 canonical zones behaviour-match reference `zic`/`zdump` over
> `1900..2040`), which remains the binding correctness contract.

## Why a separate axis

Two TZif files can describe the *same local time for every instant* (identical under `zdump`) yet
differ byte-for-byte: `zic`'s slim heuristics, local-time-type ordering, designation-table
packing, and explicit-transition horizon are all representational choices. CORE.1 proves
**behaviour** equivalence. T8 asks the orthogonal question — *how close is the actual file?* —
and answers it with a reproducible inventory rather than a vibe, so any future byte-parity claim
starts from measured fact.

Run it yourself:

```sh
zic-rs structural-report --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic
zic-rs structural-report --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic --format json
zic-rs structural-report --input … --reference-zic zic --zone America/Santiago   # one zone
```

The command compiles every canonical zone with zic-rs (in memory) and reference `zic` (into a
temp tree), decodes both with the same reader (`src/tzif/validate.rs`), and classifies the
difference. It needs a reference `zic` on `PATH`; it never mutates the system zoneinfo.

## The taxonomy

Each zone lands in **exactly one** class (the single differing dimension when there is one, else a
coarser bucket). Dimensions compared: TZif `version` byte, `timecnt`, `typecnt`, `charcnt`,
`isutcnt`, `isstdcnt`, `leapcnt`, and the POSIX footer string.

| Class | Meaning |
|-------|---------|
| `byte-identical` | identical bytes |
| `structurally-equivalent` | bytes differ, but every count + version + footer matches (type/abbreviation *ordering* or designation *packing* differs, invisible at the count level) |
| `slim/fat-timecnt` | only `timecnt` differs — the documented slim/fat explicit-transition window |
| `type-count` | only `typecnt` differs |
| `abbreviation-table` | only `charcnt` differs — designation-table packing (`zic` shares abbreviation *suffixes*) |
| `version-byte` | only the TZif version byte differs |
| `footer` | only the POSIX `TZ` footer differs |
| `ttisstd/ttisut` | only `isutcnt`/`isstdcnt` differ |
| `leap-count` | only `leapcnt` differs |
| `mixed/unexpected` | more than one dimension differs (the `dims` list shows which) |

## Measured inventory — `tzdata.zi` 2026b vs tzcode 2026b

341 canonical zones compared, **0 errors** (every zone compiles both ways and decodes). Counts are
against the system `zic`; the exact split of `byte-identical` vs `structurally-equivalent` vs
`slim/fat` shifts slightly with the reference build's slim window, but the **outliers below are
stable**.

```
zones compared    : 341
version+footer ok : 341 / 341   ✅ (T8-v3 closed the last two — see §3 below)
net extra transitions (ours − ref, slim/fat): ~4765

byte-identical           131
structurally-equivalent  135
slim/fat-timecnt          75   (incl. America/Santiago, Pacific/Easter, America/Adak)
abbreviation-table         0   ✅ (T8-abbrev: suffix-sharing packer — see §2)
mixed/unexpected           0   ✅
```

**Fields at full parity across all 341 zones:** `version`, `footer`, `isutcnt`, `isstdcnt`,
`leapcnt`, `typecnt`, **and `charcnt`** (after T8-abbrev). In particular **`ttisstd`/`ttisut` is
*not* a gap** — this `zic` build (slim default) emits `isutcnt = isstdcnt = 0`, and so does zic-rs.
The earlier hypothesis that we differed on the std/UT indicators was **refuted by measurement**:
both sides write zero. (This retires the speculative T8.1 sub-item.)

After T8-v3 (version, §3) and T8-abbrev (abbreviation packing, §2), the last structural axis was the
slim/fat `timecnt` window (§1) — now addressed by **T8-slim** (`--emit-style zic-slim`). Every other
dimension is at full parity across all 341 zones, and **behaviour stays 341/0/0** in every mode.

### 1. slim/fat `timecnt` — ✅ T8-slim RESOLVED (mode-specific `--emit-style`)

In the **default** emission zic-rs writes a *fatter* explicit-transition set than slim `zic`: it
expands recurring rules through `RECUR_HI` (+ the last finite one-shot year) where slim `zic` writes a
minimal anchor set and leans on the footer. Both are correct — the footer governs the open-ended tail
identically (law 14) — so the default is deliberately fat (it is the CORE.1-gated, behaviour-matched
output, and byte parity is *not* claimed for it).

**`--emit-style zic-slim`** reproduces reference `zic`'s slim output, pinned to `zic.c`'s `writezone`:
keep only transitions with `at <= TZstarttime`, where `TZstarttime` is the first transition past
`nonTZlimtime`, and `nonTZlimtime` advances only for an era boundary or a **finite** (`r_hiyear !=
ZIC_MAX`) rule transition — so a *final-era recurring* transition is footer-governed and dropped,
while a far-future *finite one-shot* row (e.g. `Asia/Gaza`'s Ramadan dates to 2086) is **kept**. The
implementation tags each transition `from_recurring`, truncates under `ZicSlim` only (the default path
is never entered — CORE.1 is structurally safe), and prunes now-unused local-time types so `typecnt`
also matches.

**Measured under `--emit-style zic-slim`** (all 341): the `slim/fat-timecnt` class collapses **75 → 1**
and `byte-identical` rises **130 → 141**; the sole residual is **`Europe/Lisbon`** (reference is fatter
by one transition — a no-op dedup `zic` emits that slim cannot invent; `zdump`-identical either way).
**Behaviour stays 341/0/0 in both modes** (the slim sweep over 1900..2040 matches reference exactly —
the footer reproduces every dropped transition). Tests: `tests/emit_style.rs`. zic-rs's *effectively
recurring-only* (shape-(a)) finals (`Europe/London`, `Europe/Moscow`, `Etc/UTC`) already emit
anchor+footer, so slim is a no-op there; only genuinely *mixed-in-era* (shape-(b)) finals
(`America/New_York`, `Pacific/Auckland`) were fat. `--emit-style zic-fat` currently aliases the
default. **Byte parity is claimed only in `zic-slim` mode, never for the default.**

### 2. abbreviation-table packing — `charcnt` — ✅ T8-abbrev RESOLVED

Formerly the `Asia/Ho_Chi_Minh` (charcnt-only) and `America/Adak` (timecnt+charcnt, "mixed")
outliers; now **fixed by porting `zic.c::addabbr` verbatim** into
`compile`/`tzif::data_block::build_designations`. `zic` shares abbreviation **suffixes** in the
designation table: when one abbreviation is a suffix of another they share a single NUL-terminated
run, with the shorter one's `tt_desigidx` pointing partway into the longer one — e.g. `HST` reuses
the tail of `AHST`, `LMT` the tail of `PLMT` (4 bytes saved each). zic-rs previously stored distinct
strings separately (its old exact-match dedup is just the `alen == clen` special case of suffix
sharing).

The packer is a faithful **two-pass** port (matching `zic`'s `writezone` comment *"Now that all
abbrevs have been added… it is safe to set INDMAP"*): pass 1 mutates the shared table per `addabbr`'s
three cases (new abbr is a suffix of an entry → no growth; an entry is a suffix of the new, longer
abbr → splice the missing prefix in; otherwise append); pass 2 resolves each type's stable offset
once the table is final (every lookup then hits the suffix case, no growth). Total `charcnt` is
**order-independent** under suffix sharing, so it matches `zic` regardless of zic-rs's type-intern
order. Every `tt_desigidx` still resolves to the correct string, so **behaviour is unchanged**
(`Asia/Ho_Chi_Minh` and `America/Adak` are `zdump`-identical to reference `zic`; full sweep
**341/0/0**). Result: the `abbreviation-table` (and `mixed`) classes are now **empty** — `charcnt`
parity across all 341 zones. Tests: `tzif::data_block` packer units
(`suffix_is_shared_regardless_of_order`, `plmt_lmt_share_like_ho_chi_minh`, `exact_duplicates_dedup`,
`non_suffix_abbrs_are_appended`, `empty_abbr_reuses_a_terminator`).

### 3. version byte — `America/Santiago`, `Pacific/Easter` — ✅ T8-v3 RESOLVED

Formerly the two `version-byte` outliers (ref **v3**, ours **v2**); now **fixed by pinning
`zic.c` rather than guessing**. The exact decision (tzcode 2026b, `outzone`):

```c
compat  = stringzone(...);            /* a TZDB-release "compat" year */
version = compat < 2013 ? '2' : '3';  /* v4 is separate, leap-truncation only */
```

and inside `stringrule`, `compat` is raised to **2013** (⇒ v3) precisely when a `weekday>=N` /
`weekday<=N` `ON` form is re-anchored onto a clean nth-weekday with a **non-zero day-shift**
(`wdayoff != 0`), **or** the folded transition time is **negative**. A folded time `>= 24h`
*alone* is only `compat = 1994` — still **v2**. So the **day-shift, not the displayed time's
range, is the trigger**:

| Zone | `ON` form | `wdayoff` | folded time | `zic` version | why |
|------|-----------|-----------|-------------|---------------|-----|
| `America/Santiago` | `Sun>=2` | 1 | `/24` (in range) | **v3** | day-shift ≠ 0 |
| `Pacific/Easter` | `Sun>=2` | 1 | `/22` (in range) | **v3** | day-shift ≠ 0 |
| `Asia/Gaza` | `Sat<=30` | 2 | `/50` | **v3** | day-shift ≠ 0 (the `/50` is incidental) |
| `America/Nuuk` | `lastSun` | 0 | `/-1` | **v3** | negative folded time |
| `Africa/Cairo` | `lastThu 24:00` | 0 | `/24` | **v2** | no shift; `≥24h` alone is only compat 1994 |

zic-rs's `posix_footer::recurring` already computes the per-rule day-shift (`date_rule`), so the
fix keys the version on `dst_shift != 0 || std_shift != 0 || dst_tod < 0 || std_tod < 0` — the
`compat >= 2013` condition transcribed directly. My earlier `tod` *range* heuristic was inspecting
the wrong variable; an attempt to bump v3 at `tod ≥ 24h` had been tried and **rejected** because it
regressed `Africa/Cairo`. Result: **version+footer parity is now 341/341**, with no other zone
flipped, and the full `1900..2040` `zdump` sweep stays **341/0/0** (the version byte never affects
`zdump`). Regression tests: `recurring_dayshift_forces_v3_even_when_time_in_range`,
`recurring_last_weekday_24h_stays_v2`.

## Doctrine (permanent)

- **Behaviour parity (CORE.1) is the contract.** Structural differences above never weaken it.
- **Structural parity is measured, not claimed.** The `structural-report` command is the receipt;
  these numbers are reproducible, not asserted.
- **Byte parity is claimed only where a reference blob is pinned** (`fixtures/expected/`:
  `Etc/UTC`, `Test/Fixed`). The `byte-identical` class above is observational, not a guarantee.
- **Convergent, behaviour-neutral correctness fixes become the default; genuine emission *policy*
  choices stay mode-gated.** The distinction (made explicit after T8-abbrev): where reference `zic`
  has a *single canonical* representation and matching it is pure convergence with zero behaviour
  change — the **version byte** (T8-v3) and **abbreviation suffix-sharing** (T8-abbrev) — zic-rs adopts
  it as the **default** (there is no reason to keep a looser output that merely diverges from `zic`).
  Where `zic` offers a genuine *policy* knob with more than one behaviour-equivalent answer — the
  **slim/fat `timecnt` window** — that ships behind an explicit `--emit-style` mode and the default is
  **never** changed. In every case behaviour parity (CORE.1, 341/0/0) is preserved and gated by the
  full sweep.
- **All three structural sub-campaigns are now done** (no silent gaps): **T8-v3 ✅** (version byte →
  341/341 version+footer); **T8-abbrev ✅** (suffix-sharing packer, `zic.c::addabbr` ported → `charcnt`
  parity, `abbreviation-table`/`mixed` classes empty); **T8-slim ✅** (`--emit-style zic-slim`
  reproduces slim `zic`, collapsing `slim/fat-timecnt` 75 → 1 — `Europe/Lisbon`, a ref-fatter no-op;
  mode-specific, the default stays the behaviour-matched fat output). **Structural parity is now a
  closed, measured story:** default = behaviour-matched; `zic-slim` = byte-close to reference (1
  enumerated residual); behaviour 341/0/0 in both.

## Relationship to the ladder

T8 is the **inventory** step of the full parity ladder (see [roadmap.md](roadmap.md)):
it quantifies how far the *writer* is from reference `zic` before CLI/leap/range modes complicate
it. It deliberately stops at measurement + classification; the emission modes that would *close*
the slim/fat and packing gaps are downstream, mode-specific, and gated by the standing 341/341
behaviour sweep.
