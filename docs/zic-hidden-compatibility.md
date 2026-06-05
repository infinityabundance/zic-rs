# Hidden `zic` compatibility & operational traps

A companion to [zic-deep-semantics.md](zic-deep-semantics.md). That file answers *"how does time
compile?"* — the civil-time **state-machine** laws. This file answers a different, less glamorous but
equally important question:

> *What operational traps make a `zic` replacement **boringly safe** in production?*

These are mostly about **compatibility contracts, reader quirks, file-generation policy, parser edge
cases, and overflow/safety** — the things learned by being burned, not by reading the grammar. Most
are **honest future campaigns**, named here so they are explicit gaps, not hidden claims. The
verbose-warning *table* form is [zic-v-hazards.md](zic-v-hazards.md); this is the deeper narrative.

Status legend: **✅ done** · **▷ partial / enforced-conservatively** · **◇ future campaign (named,
not claimed)**.

## 1. Input physical format & quoting
`zic` input is newline-terminated text, **no NUL bytes**, line length **≤ 2048 bytes** (including the
newline), typically UTF-8/ASCII, with non-portable bytes confined to comments. An unquoted `#`
begins a comment, but whitespace and `#` may appear inside double quotes.
*zic-rs status:* ▷ NUL rejection + quote-aware, comment-aware field splitting in the lexer; the
2048-byte line guard is enforced. *Future:* pin quoted-`#` / quoted-whitespace / unterminated-quote
edge cases against reference `zic` (fail closed on unterminated quote). *Campaign:* parser-hardening
checklist (whitespace classes: space, FF, CR, NL, tab, VT).

## 2. Reserved `TYPE` field
A `Rule` line's third field is a reserved `-` (formerly `TYPE`); it should always be `-` for
compatibility with older `zic`.
*zic-rs status:* ▷ accept `-`. *Future:* fail closed deterministically on any other value; document
that old year-type filtering scripts are out of scope. *Campaign:* `reserved_type_field`.

## 3. Proleptic-Gregorian years (incl. year 0) & unrepresentable instants
Rule years are signed integers in the **proleptic Gregorian** calendar (year 0 precedes year 1).
Rules may describe times **not representable** as a time value; `zic` **ignores** such times so rules
stay portable across hosts — it does not fail.
*zic-rs status:* ▷ signed-year parsing; current horizon is `1900..2040`. *Future:* if the horizon
expands, *ignore* (don't fatally reject) unrepresentable rule instants where reference `zic` does.
*Campaign:* year-model + unrepresentable-instant parity.

## 4. Duplicate-instant errors
In one zone it is an **error** for two rules to take effect at the same instant, or for two zone
changes to coincide.
*zic-rs status:* ◇ not yet a distinct diagnostic. *Future:* reject deterministically (do not silently
sort). *Campaign:* `two_rules_same_instant_fail_closed`, `two_zone_changes_same_instant_fail_closed`.

## 5. `ttisstd` / `ttisut` type indicators
TZif local-time types carry **standard/wall** and **UT/local** indicators (`ttisstds`/`ttisuts` in
`zic.c`), alongside offset/isdst/abbr. A faithful writer is not merely `(time, offset, isdst, abbr)`;
the type tells conforming readers how to interpret transition times.
*zic-rs status:* ✅ **measured at parity** (campaign T8). This tzcode 2026b build (slim default)
emits **`isutcnt = isstdcnt = 0`** — i.e. *no* std/wall or UT/local indicators — and so does zic-rs,
across **all 341** canonical zones (the earlier "conservative defaults / prove later" framing was a
hypothesis; `structural-report` *refuted* the idea that we differed here — both sides write zero).
RFC 9636 permits a count of either `0` or `typecnt`; `zic` slim chooses `0`, and matching it is the
parity result. If a future `--emit-style zic-fat` ever emits the `typecnt` form, that mode must
reproduce the exact per-type indicator bytes. *Receipt:* `docs/structural-parity.md`,
`zic-rs structural-report`.

## 6. `-r` range truncation & the `-00` placeholder
`zic -r` limits output to a timestamp range; where data is omitted, `zic` uses **offset 0 and
abbreviation `-00`**. `-00` means *local time unspecified* — **not** UTC.
*zic-rs status:* ◇ no range truncation. *Future:* explicit lo/hi boundary types + `-00` handling,
with the doctrine that `-00 ≠ UTC`. *Campaign:* `T-r` range truncation.

## 7. `-R @hi` redundant-tail transitions (distinct from `-b fat`)
`-R @hi` adds redundant **trailing explicit transitions** up to `hi` even when the footer could
express them (for nonstandard readers that ignore the footer). This is **different** from `-b fat`
backward-compat bloat.
*zic-rs status:* ◇ single emission style (fat-ish explicit + footer). *Future:* keep
`slim` / `fat_compat` / `redundant_tail_until_hi` as *separate* emission styles — do not fold `-R`
into "fat." *Campaign:* `--emit-style`.

## 8. `posixrules` is a legacy landmine
`zic -p` creates a `posixrules` link; the manual calls non-`-` use obsolete and problematic (must not
be used for timestamps after 2037; must not be combined with slim output when transitions are at
standard/UT rather than local time).
*zic-rs status:* ◇ unsupported. *Doctrine:* `posixrules` is **intentionally not** a canonical-zone
conformance target — it is a legacy runtime-policy feature, not core source→TZif semantics.

## 9. Transition-count reader limits
`zic -v` warns above **1200** transitions (older clients) and the reference reader caps at **2000**;
this is **reader-compatibility**, not just memory. zic-rs separately enforces a hard
`MAX_TRANSITIONS` safety limit (`ZIC009`).
*zic-rs status:* ▷ `ZIC009` hard cap enforced. *Future:* a *soft* report — `count`,
`exceeds_1200_old_reader_warning`, `exceeds_2000_reference_reader_limit` — as a warning, not a fail.
*Campaign:* `support-report --report-reader-compat`.

## 10. Abbreviation length is a **portability** warning, not a correctness one
POSIX wants abbreviations of **3–6** chars; `zic -v` warns outside that. But a `<3`/`>6` abbreviation
is **not invalid** for `zic` — zic-rs emits it faithfully (and `zdump` warns on both ours and the
reference identically).
*zic-rs status:* ✅ emitted faithfully (e.g. `Asia/Macau` "CT", `Atlantic/Bermuda` "AT" both match
reference). *Distinction:* behaviour mismatch — *no*; portability warning — *yes*.

## 11. Filename / path safety
`zic -v` warns on output names with bytes outside `[A-Za-z-/_]`, components > 14 bytes, or
leading `-`; zone names must not contain `.`/`..` path components.
*zic-rs status:* ✅ path-traversal-proof output tree — rejects absolute paths, `.`/`..`, traversal,
leading `-` (`ZIC008`); atomic exclusive create. *Future:* surface the softer `-v` portability
warnings *separately* from the hard safety rejects. *Tests:* `tests/output_safety.rs`.

## 12. Link-to-link is valid `zic`, hazardous for old readers
`Link` targets may be zones **or other links**; links may precede targets; a chain names the same
zone. `zic -v` warns because some parsers (and `zic` itself ≤ 2022e) mishandled link-to-link — it is
**not** bad data.
*zic-rs status:* ✅ chain resolution + cycle/missing diagnostics; `--alias-map`; `support-report`
counts canonical/links/total/cycles/missing separately. *Future:* add `link_chain_count` /
`link_chain_max_depth` / `old_reader_hazard` fields. *Campaign:* `--report-link-chains`.

## 13. Leap input is a separate grammar (`Rolling` / `Stationary` / `Expires`)
Leap-second input is a **separate file context** with `Leap` (with a `Rolling`/`Stationary`
distinction) and an `Expires` line — not a list of UTC corrections, and not part of the main
`Rule`/`Zone`/`Link` dispatch.
*zic-rs status:* ✅ zone-source dispatch only; `Leap`/`Expires` are **not** accepted as zone-source
directives (`R`/`Z`/`L` zishrink prefixes are main-file only). *Future:* a separate leap-file mode
(`Leap Rolling`/`Leap Stationary`/`Expires` + truncation warnings). *Campaign:* leap-file support.

## 14. Overflow-checked count/size arithmetic
`zic.c` uses explicit checked size addition/multiplication with a `size_overflow` fatal path — it has
had to care about allocation sizing, not just semantics.
*zic-rs status:* ▷ Rust bounds-checks indexing and `#![forbid(unsafe_code)]` removes whole classes of
UB; `MAX_TRANSITIONS` bounds output. *Future:* use **checked** arithmetic for all TZif count/size
calculations and reject oversized output deterministically; add overflow tests even with artificial
fixtures. *Campaign:* count/size overflow hardening.

## 15. `minimum` is backward-compat only
`zic.c` documents the `minimum` year synonym as backward-compatibility only.
*zic-rs status:* ✅ `FROM = minimum → 1900` (T3.2a) — obsolete compatibility, **not** an infinite-past
bound. *Tests:* `minimum_lowers_to_1900`.

## 16. Reader compatibility differs by timestamp era → multi-horizon reporting
`zic -v` notes some issues affect only timestamps **before 1970** or **after early 2038**. So a zone
can be "production-post-1970 verified" even if pre-1970 behaviour is still being pinned.
*zic-rs status:* ▷ a single declared horizon (`1900..2040`) today. *Future:* report multiple
horizons — core `1900..2040`, post-1970 `1970..2038`, tail `2038..2040` — so coverage is reported
*usefully and honestly* (per [zic-deep-semantics.md](zic-deep-semantics.md) law 17, a passing
horizon is reference-agreement for that zone, not regional truth). *Campaign:* `support-report
--horizon LO,HI` / `--horizon-profile post1970`.

---

## Why this list exists

None of these are required to behaviour-match the 338 supported canonical zones over `1900..2040`
(that milestone is met). They are the **operational hardening frontier** — the difference between "a
compiler that passes the corpus" and "a compiler you would deploy." Each is named, statused, and
(where unbuilt) a declared future campaign — never a silent gap. See
[roadmap.md](roadmap.md) for sequencing and [security.md](security.md) for the path/overflow posture.
