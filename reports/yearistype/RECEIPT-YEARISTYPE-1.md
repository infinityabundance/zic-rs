# RECEIPT — YEARISTYPE.1 (historical Rule TYPE / `-y` replay support) — 2026-06-05

> **Claim wording (binding):** *YEARISTYPE.1 does not claim current reference `zic` still supports `-y` or
> Rule `TYPE` yearistype scripts. It adds explicit historical-source replay support for stable tzdata
> releases whose Rule `TYPE` fields require the old yearistype ecology, and verifies behaviour against an
> admitted historical reference `zic` oracle.*

## The gap it closes

TZDB-ATLAS.2 left exactly one historical wall: the **66 pre-2000f stable tzdata releases** that current
reference `zic` **cannot build**, because tzcode **2020a removed `zic -y` and the Rule `TYPE` (yearistype)
ecology**. These are the atlas `reference_zic` band — *not* a zic-rs divergence (the reference itself can't
build them). Empirically, the **only** `TYPE` predicates in the whole archive are **`even` and `odd`**, and
**only** in `australasia` (the `AS` rules for Australia/Adelaide & Broken_Hill — the 1993–94-era name was
`Australia/South`). No `uspres`/`nonpres` appears in any stable release.

## What was anchored (acceptance #1 — admitted historical oracle)

The `TYPE` semantics are **not** in `zic.c` — `yearistype(year,type)` *shells out* to a `yearistype`
script. So the semantics were anchored to the **actual script** every affected release shipped:

| artifact | identity |
|---|---|
| `yearistype.sh` **v7.4** | sha256 `86cfb6b1f05acffce45322ffdaafc5cb035d459da6e75ca0d2ddd5f8efad1702` (identical 1997a→2000e; the 93c variant's `even`/`odd` cases are byte-identical) |
| oracle `zic` source | `tzcode2019c.tar.gz` sha256 `f6ebd3668e02d5ed223d3b7b1947561bf2d2da2f4bd1db61efefd9e06c167ed4` (the last era with `-y`; `TYPE` is script-delegated, so the semantics are invariant across tzcode versions) |
| built oracle binary | `zic (oracle) 2019c`, sha256 `595f7b9e897e52abb0986a088a6fcde88e67f5f6dfe298ab86fba0e584ffa1cc` (built `cc -w -o zic_oracle zic.c` with a `version.h` stub) |
| oracle command | `zic_oracle -y <yearistype-v7.4> -d <out> <region files>` |

The four predicates, read **directly from the script's `case` globs** (not from memory) and confirmed
against the oracle on `Australia/Adelaide` 1990–1994:

```text
even    decimal year ends {0,2,4,6,8}   ⇔ year % 2 == 0
odd     decimal year ends {1,3,5,7,9}   ⇔ year % 2 != 0
uspres  *[02468][048] | *[13579][26]    ⇔ year % 4 == 0   (US presidential-election year)
nonpres negation of uspres              ⇔ year % 4 != 0
wild    any other type                  → exit 1 (false)  → a hard error in either mode
```

## What was implemented (acceptance #2, #4 — bounded, opt-in, no script execution)

A new **`--legacy-yearistype`** flag (composable with `--legacy-latin1` via the typed
[`LegacySource`] bundle). The four predicates are **internal deterministic functions** — zic-rs **never
executes the historical shell script** (the safety rule). Threading: a typed `YearType` enum
(`model::YearType`) on `RuleRecord`; `parse_year_type` gates the `TYPE` field; `rule_active_in` adds
`&& rule.year_type.includes(year)` (mirroring `zic`'s `r_todo = … && yearistype(year,type)`).

- **Default (modern) stays UTF-8/`-`-only and fails closed:** any non-`-` TYPE → **`ZIC027`** (new code).
  This is a bucket-3 hardening — the parser previously *ignored* the field, which would silently
  mis-compile a year-parity rule (firing it every year). All 139 Rule lines in the test corpus use `-`,
  so the modern path and **CORE.1 are byte-unchanged**.
- **`--legacy-yearistype` admits** `even`/`odd`/`uspres`/`nonpres`; an unknown ("wild") type, or a
  *typed single year*, is still `ZIC027` (mirrors `zic`'s `yearistype.sh` exit-1 and "typed single year").

## Result (acceptance #3, #6 — 66 releases, oracle-compared)

`bash reports/yearistype/gauntlet.sh` (oracle `zic -y` vs zic-rs `--legacy-yearistype`, per-fixture `zdump`
over 1980–2037; the even/odd zone is **Australia/Adelaide**, or **Australia/South** in the 1993–94 era —
**not** Lord Howe, which uses the un-typed `LH` rules):

| verdict | releases | meaning |
|---|--:|---|
| **match** | **55** | the even/odd zone builds in both **and every fixture's `zdump` is byte-identical to the historical oracle** |
| **deferred-perpetual-footer** | **11** | 93b–94f: the even/odd rules were **perpetual** (`1990 max even/odd`), so the recurring tail is **not POSIX-footer-expressible** (a TZ string cannot encode year-parity). zic-rs **accepts the TYPE** (no `ZIC027`) then defers the footer — a **separate, pre-existing** feature, **not** a yearistype failure |
| **zic-rs-divergent** | **0** | — |

**354 / 354 fixture comparisons match · 0 divergent.** The yearistype admission feature is verified
byte-identical to the historical oracle across all 55 releases where the even/odd zone is POSIX-clean.

### The honest split (acceptance #6 — classify honestly)

The 11 `deferred-perpetual-footer` releases are a **genuine, separate finding**, not a yearistype bug:
before 1995 the `AS` even/odd rules were perpetual (`1990 max even` / `1990 max odd`), giving **two
perpetual standard rules** that alternate by year parity. The historical oracle renders this by **explicit
expansion to the v2 horizon + an empty footer** (`…CST\0\n\n` — no TZ string). zic-rs's footer synthesiser
requires one POSIX-expressible perpetual DST+standard pair and refuses (`ZIC001`). This is the same
recurring-footer deferred class RELEASE-LADDER.1 noted, here triggered by perpetual year-parity rules. By
1995 the rules were reformed (`1995 max - Mar lastSun`; even/odd became finite 1990–1994), and all such
releases compile. **YEARISTYPE.1 closes the Rule-TYPE-admission half; the non-POSIX perpetual-footer half
is honestly deferred** (`docs/differences-from-reference-zic.md`, bucket 4).

## Tests (acceptance #5) — 515 total (+8)

`tests/yearistype.rs` (4): predicate vs `yearistype.sh` v7.4 (even/odd/uspres/nonpres + token round-trip);
default mode rejects with `ZIC027`; legacy mode admits; **even/odd fire by year parity** (1990→Mar 18 …
1994→Mar 20, matching the oracle). `src/source/parser.rs` (4): default rejects · legacy admits even/odd ·
**wild type rejected** · **typed single year rejected**. The `ZIC001`–`ZIC027` totality test pins the new code.

## Non-claims

- **Does not claim current reference `zic` supports `-y`** — it doesn't (tzcode 2020a removed it). This is
  *historical-source replay*, compared against an **admitted historical oracle**, not current-reference parity.
- zic-rs **never executes** the historical `yearistype` shell script; the predicates are internal functions.
- Only `even`/`odd` occur in the stable archive; `uspres`/`nonpres` are implemented for fidelity to the
  script but **not exercised by any admitted release**.
- The 11 perpetual-footer releases are **not** claimed closed (a separate deferred feature); verified
  against the oracle on the bounded fixture set over 1980–2037, not all zones / all time / byte parity.

## Gate

`--legacy-yearistype` off by default → modern path byte-unchanged: **CORE.1 341/0/0**, 515 tests,
`fmt`/`clippy -D warnings` clean, doc-staleness green. Reproduce: `bash reports/yearistype/gauntlet.sh`
(build the oracle first: fetch `tzcode2019c.tar.gz`, `cc -w -o zic_oracle zic.c` with a `version.h` stub).

[`LegacySource`]: ../../src/source/mod.rs
