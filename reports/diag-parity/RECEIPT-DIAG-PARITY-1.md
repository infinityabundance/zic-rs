# RECEIPT — DIAG-PARITY.1 / HOSTILE-EQUIV.1 (failure-mode parity) — 2026-06-05

> **Claim wording (binding):** *DIAG-PARITY.1 does not claim byte-for-byte diagnostic wording parity. It
> classifies malformed-source and hostile-input behaviour by diagnostic class, severity, location, exit
> status, and output side effects, and records whether zic-rs matches, safely diverges, or intentionally
> refuses.*

## Method

Run a corpus of **17 malformed/hostile** fixtures through **both** reference `zic` (host, tzcode 2026b) and
zic-rs, and compare the **failure mode** — diagnostic **class** · **severity** (error vs warning) · **exit
status** · **output side effects** (partial files) — *never* the exact wording (wording drifts across
releases/vendors; classes are pinned to the rule violated). Corpus: `reports/diag-parity/corpus/`. Reproduce:
`bash reports/diag-parity/gauntlet.sh` → `reports/diag-parity/diag-parity.tsv`.

The corpus spans every diagnostic layer:

- **lexical/admissibility:** NUL byte · overlong line · unterminated line (missing final `\n`) ·
  unterminated quote · wrong field count
- **structural:** unknown line type · continuation-without-zone · duplicate zone
- **semantic:** invalid month · invalid day rule · invalid time suffix · `yearistype` TYPE (default) ·
  two-rules-same-instant · value over 24h
- **operational/safety:** path-traversal zone name · (partial-output probe)
- **warnings:** abbreviation too long · value over 24h

## Result — 17 fixtures

| verdict | count | meaning |
|---|--:|---|
| **class_exit_match** | **14** | same diagnostic class **and** same exit status |
| **intentional_divergence** | **1** | zic-rs classifies *more precisely* (documented) |
| **safer_no_partial** | **1** | same error class, but zic-rs leaves **no partial output** where reference wrote a file |
| **warning_match** | **1** | both accept with a warning (exit 0) |
| **divergent** | **0** | — |

**zic-rs matches reference `zic`'s failure CLASS and exit status on 14/17 hostile fixtures; the 3 non-exact
cases are all documented, intentional, and safer. Zero unexplained divergences.**

### The 3 non-exact cases (all named/safer/intentional)

| fixture | reference `zic` | zic-rs | why |
|---|---|---|---|
| `continuation_without_zone` | exit 1 · "input line of unknown type" (generic) | exit 1 · **`ZIC014` ContinuationWithoutZone** (finer) | zic-rs classifies a continuation-shaped stray line *more precisely* than reference, which folds it into "unknown line type" — the documented **T13.2** intentional divergence (finer, never wrong) |
| `simultaneous_transition` | exit 1 · "two rules for same instant" · **writes a partial file** | exit 1 · `ZIC023` SimultaneousTransition · **no partial output** | same class + exit; zic-rs's **compile-all-to-memory-then-write** leaves no partial install where reference writes as it goes — the documented **T9.3** safer divergence (a fatal never leaves a half-built tree) |
| `value_over_24h` | exit 0 · warning | exit 0 · `ZIC026` warning | both accept-with-warning; severity + exit match |

## What this proves (the failure-mode contract)

- **Same class, same exit:** for the lexical/structural/semantic error fixtures (NUL, overlong, unterminated
  line/quote, field count, unknown line, duplicate zone, invalid month/day/time, `yearistype`, path
  traversal), zic-rs fails with the **same diagnostic class and the same exit status** as reference `zic`.
- **Warnings stay warnings:** the warning fixtures (abbreviation too long, value over 24h) **compile with a
  warning and exit 0** in both — zic-rs does not over-escalate a warning to an error.
- **Divergences are safer + named:** every deviation is one of the project's two documented safer
  divergences — *finer classification* (T13.2, `ContinuationWithoutZone`) or *no-partial-output* (T9.3) —
  not a class mismatch or an exit-status mismatch.

## Non-claims

- **No byte-for-byte wording parity** — compared by class/severity/exit/side-effects, never verbatim stderr
  (wording drifts across releases/vendors; the per-class wording ledger is `docs/zic-warning-parity.md`).
- Reference = current host `zic` 2026b; a bounded corpus of 17 fixtures, not every malformed input.
- A `warning_match` / `class_exit_match` is a *failure-mode* statement (how it fails), not a claim about the
  success path (that is CORE.1 / the parity matrix).
- Partial-output behaviour is platform/filesystem-bounded; the `simultaneous_transition` case demonstrates
  the no-partial guarantee on this host (reference wrote 1 file, zic-rs wrote 0).

## Gate

Docs/report only — no `src/` change (the unit-level `tests/diagnostic_parity.rs` is unchanged); CORE.1
341/0/0 + 519 tests unaffected; doc-staleness green. Cross-linked from STATUS · the atlas ·
`docs/zic-warning-parity.md` (T13) · `docs/zic-hostile-input-parity.md` (T14).
