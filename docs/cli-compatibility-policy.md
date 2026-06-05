# CLI compatibility policy (T17.6)

> **Doctrine.** *A CLI exit code reports **command execution status**, not necessarily the presence or
> absence of differences. **Diagnostic / report** commands may exit `0` while reporting problems;
> **gate** commands may exit non-zero when the subject fails admission.* Reading "exit 0" as "nothing to
> see" is a misuse this policy names and bounds (cf. `RISK.DIFF.1`, `RISK.REPORT.1` in
> `risk-register.md`). Policy only — **no behaviour change**; it classifies the existing surface.

## The exit-status contract (campaign T9.2, stable)

A deliberately small, script-friendly taxonomy (pinned by `tests/cli_operational_parity.rs`; the *wording*
of diagnostics may evolve, the *classification* does not):

| Code | Meaning |
|---|---|
| `0` | success — the command executed (a *report* command exiting 0 means it ran and produced its report, **not** that it found nothing) |
| `1` | an **operational / compiler / config** failure (parse error · unsupported construct · `ZIC008` output-path traversal · no-clobber without `--force` · missing/unreadable input · reference tool absent where required · malformed `--horizon` · a **non-admitted** vendor receipt). Diagnostic on stderr. |
| `2` | a **CLI usage** error (`clap`: unknown option, missing a structurally-required arg). Emitted by `clap` before the library runs. |

`--help` / `--version` print and exit `0`.

## Command classification — gate / diagnosis / witness / admission / convenience

The load-bearing distinction: **what may NOT be inferred from a `0` exit.**

| Command | Class | Exit semantics | Must **not** infer from exit 0 |
|---|---|---|---|
| `compile` | **gate** | `0` = every selected zone compiled + materialised; `1` = a fatal (unsupported/traversal/clobber/no-partial-install) | — (a gate: 0 *does* mean the compile succeeded for the selected set, within the declared subset) |
| `compare` | **gate-ish** (oracle) | `0` = ran + compared; `1` = reference tool absent / operational failure | that behaviour *matched* — read the report; absence of the oracle is a `1`, not a silent pass |
| `support-report` | **diagnosis/report** | `0` = report produced; `1` = operational failure | that all zones are supported — read `conformance_status` + the buckets |
| `structural-report` | **diagnosis/report** | `0` = report produced | structural parity — read the `ParityClass` rows |
| `semantic-report` | **diagnosis/report** | `0` = report produced | behaviour parity — read `witnesses[]` + `oracle_mode` (may be `unavailable`) |
| `tzif-validate` | **diagnosis/report** | `0` = validation **ran** | conformance — read the five typed verdicts (a `violation` verdict still exits 0; it is a *report*, not a gate) |
| `aux-table-validate` | **diagnosis/report** | `0` = validation ran | tables are valid — read `ZoneTableStructuralVerdict` |
| `vendor-oracle-sample` | **convenience** | `0` = emitted the schema sample | anything about a real platform (it is a schema example) |
| `vendor-oracle-admit` | **admission/gate** | `0` = receipt **admitted**; `1` = **not admitted** (by rule) *or* a parse failure (distinct stderr) | that the platform is "good" — admission is a typed, rule-based verdict; a parse failure ≠ an inadmissible receipt |
| `release-diff` | **witness/report** | `0` = the diff ran (differences are the **output**); `1` = operational failure only (bad `--horizon`, unreadable source) | that the releases are identical — read the `ReleaseChangeKind` rows + `oracle_mode`; **`behaviour_unassessed` ≠ unchanged** |
| `doctor` | **diagnosis** | **always `0`** (a diagnosis, not a gate) | that the host tools are present/correct — read each `ToolStatus`; an absent/`unsupported`/`unreadable` tool is reported, not an error |
| `size-report` | **diagnosis/report** | `0` = the tree was measured; `1` = `--out` is not a readable dir (config error) | that the bundle is *approved* for any runtime, or that a `tzif_files` entry is a zone vs a copy-mode link — it reports the tree on disk; `bundle_hash` is a determinism witness, not an attestation |
| `explain` | **convenience/diagnostic** | `0` = trace produced | — |
| `supported-syntax` | **convenience** | `0` = printed the supported-syntax summary | — |

**The three anchors, restated:** `doctor` = diagnosis (always 0) · `release-diff` = witness (0 even with
differences; only operational failure is 1) · `compile` / `vendor-oracle-admit` = gate (non-zero when the
subject fails). A report command's `0` means *"I ran"*, never *"all clear"*.

## Machine-readable output policy

- Commands with `--format json` emit **deterministic** JSON (sorted keys where applicable, fixed field
  order, LF, no timestamps, no host-`now`/locale dependence) carrying a `schema` id governed by
  [`schema-compatibility-policy.md`](schema-compatibility-policy.md).
- Text (`--format text`) output is **human-facing and may evolve in wording**; it is **not** a stable
  contract. Scripts must consume `--format json`, not parse text.
- stderr carries diagnostics (wording may evolve); the *exit code* + the JSON *schema fields* are the
  stable surfaces.

## Command + flag stability

- **All commands above are stable** (none is currently labelled *experimental*). A new command is added
  additively; an existing command's *name* and *exit class* are stable. Pre-1.0, a command may be marked
  *experimental* in its `--help` if its contract is not yet frozen — none is today.
- **Flags are append-only** in spirit: a new flag is additive with a documented default; removing/renaming
  a flag, or changing a flag's meaning, is a breaking change (held for a major version once consumers
  exist). The deliberate CLI-shape divergences from reference `zic` (subcommands + required `--out`, no
  implicit system install) are documented in `docs/zic-operational-parity.md` /
  `docs/differences-from-reference-zic.md`, not re-litigated here.
- The exit-status *classification* (0/1/2) is the frozen contract; diagnostic *wording* is explicitly not.

## Non-claims

- This policy is about the **CLI/exit/output contract**, not about *what zic-rs proves* (that is CORE.1 +
  the reports + the risk register). A stable exit contract does not imply broad `zic` parity.
- It does not claim argv/exit/stderr parity *with reference `zic`* — that is the separate drop-in
  compatibility question (a future `docs/drop-in-compatibility-contract.md`, T19); this doc governs
  zic-rs's *own* CLI contract.

See also: [`schema-compatibility-policy.md`](schema-compatibility-policy.md) · `risk-register.md`
(`RISK.DIFF.1` unknown-as-unchanged · `RISK.REPORT.1` report-as-attestation) · `docs/zic-operational-parity.md`
(the T9 exit-status taxonomy).
