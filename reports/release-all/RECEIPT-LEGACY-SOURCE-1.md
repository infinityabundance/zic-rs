# RECEIPT — LEGACY-SOURCE.1 (bounded Latin-1 tzdb source admission) — 2026-06-05

> **Claim wording (binding):** *LEGACY-SOURCE.1 does not weaken the modern UTF-8 source contract. It adds
> bounded historical-source replay support for admitted Latin-1-era tzdb releases and verifies behaviour
> against reference `zic` on those releases.*

## The gap it closes

TZDB-ATLAS.1 named exactly one zic-rs-side historical compatibility band: **2008a–2012j →
`source-shape-incompatible`**, because those tzdb releases carry **non-UTF-8 (Latin-1)** bytes and zic-rs
fail-closes on non-UTF-8 (`ZIC012`, the T14 admissibility decision). Verified: the bad bytes are **only in
`#` comments** (accented author/place names, e.g. *"La Nación"* — `southamerica`, byte `0xf3`); `file`
reports `southamerica` as *ISO-8859 text* in this band and *UTF-8 text* from ~2013. No semantics-bearing
field is non-UTF-8.

## The fix (bounded, opt-in, guard-railed)

A new **`--legacy-latin1`** flag (`src/cli.rs`), threaded `tokenize_with` → `parse_into_with` →
`load_database_with` (each a `*_with` variant; the existing UTF-8 entry points are unchanged wrappers, so
**no caller/test churn**). When set and the bytes are not valid UTF-8:

- **`reject_non_utf8_outside_comments`** scans line-by-line: a non-UTF-8 byte is admitted **only** if it
  lies at/after the line's first `#` (inside a comment). A non-UTF-8 byte in a **semantics-bearing field**
  is **refused** (a typed error), even in legacy mode.
- The source is then decoded as **ISO-8859-1** (each byte → `U+0000..=U+00FF`, lossless). Because the lexer
  strips comments, **the compiled output is unaffected** — only comment text differs. (So there is no lossy
  *output*; the decode is an explicit, opt-in interpretation of comment bytes.)

The **modern contract is preserved**: with the flag off (the default), invalid UTF-8 is still a hard `ZIC012`.

## Result (acceptance #3, #4)

Re-ran the **entire blocked band (60 releases, 2008a–2012j)** with `--legacy-latin1`
(`LEGACY=1 bash reports/release-all/data-gauntlet.sh`, verified per-fixture vs reference `zic`):

| | before (UTF-8 only) | after (`--legacy-latin1`) |
|---|---|---|
| releases that compile | 0 / 60 | **60 / 60** |
| releases behaviour-match (0 fixture divergence) | 0 / 60 | **60 / 60** |
| fixture `zdump` comparisons | — | **420 / 420 match, 0 divergent** |

**All 60 source-shape-incompatible releases move to `match`** under legacy replay — the atlas band closes.
The upstream behaviour tally becomes (legacy mode): **210 match · 66 reference-build-incompatible (the
`yearistype` *reference* limit, unchanged) · 0 source-shape-incompatible · 0 divergent.**

## Regression tests (acceptance #6)

`src/source/lexer.rs`: `legacy_latin1_admits_non_utf8_in_comments` (legacy admits comment Latin-1 + the
record parses; default rejects) · `modern_invalid_utf8_fails_closed` (the modern contract preserved) ·
`legacy_rejects_non_utf8_outside_comment` (semantics-bearing non-UTF-8 refused even in legacy mode).

## Non-claims

- **Does not weaken the modern UTF-8 contract** (default unchanged; flag is explicit opt-in for admitted
  historical replay only).
- Admits Latin-1 **only inside `#` comments**; never in a semantics-bearing field (refused with a typed error).
- Verified against reference `zic` on the bounded fixture set over 1900..2037; not a claim of full
  old-release admission.

## Gate

`--legacy-latin1` off by default → the modern compile path is byte-unchanged: **CORE.1 341/0/0**, 507 tests,
`fmt`/`clippy -D warnings` clean, doc-staleness green. Reproduce: `LEGACY=1 bash reports/release-all/data-gauntlet.sh`.
