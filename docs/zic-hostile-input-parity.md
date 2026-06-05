# Hostile-input & parser-edge parity (T14) — inventory (T14.1) · tightening (T14.2) · metamorphic (T14.3) · pathology ledger (T14.4 → `zic-pathology-ledger.md`) · name-as-path (T14.5 → `zic-zone-name-path-policy.md`)

> **T14.1 is reference-first inventory — NO behaviour change.** It pins the **input-admissibility**
> rules from the pinned reference (`zic.c` tzcode 2026b + `zic(8)`), classifies each by diagnostic
> layer (inheriting the T13 taxonomy), records zic-rs's *current* behaviour and the divergences, and
> names the implementation substeps. It changes **no compiler behaviour** — the one code addition is the
> executable admissibility *witness* (`tests/input_admissibility.rs`), a pure observation over the public
> `load_database` API.
>
> **T14.2 is the first behaviour change** — it closes the one leniency divergence T14.1 found:
> missing-final-newline (`ZIC021_UNTERMINATED_INPUT_LINE`) and unterminated-quote
> (`ZIC022_UNTERMINATED_QUOTE`) are now fatal, matching reference `zic`; the T14.1 witness flipped from
> lenient/uncoded to fatal/coded.
>
> **T14.3 returns to contract-level (NO behaviour change)** — `tests/metamorphic_ordering.rs` proves
> record-order independence (the `zic(8)` law and its continuation exception): permuting independent
> records / comments / inter-field whitespace yields byte-identical per-zone output, while source
> identity differs.
>
> **T14.4 is a classify-first pathology ledger** (`docs/zic-pathology-ledger.md`) — and it found + fixed
> a **panic on untrusted input**: "two rules for same instant" now fails closed with
> `ZIC023_SIMULTANEOUS_TRANSITION` instead of tripping a `debug_assert!`.
>
> **T14.5 is the name-as-path policy** (`docs/zic-zone-name-path-policy.md`) — the fatal `ZIC008`
> structural rules were already complete; it adds the verbose-only portability warnings `ZIC024`
> (non-benign byte) / `ZIC025` (overlength component) matching `zic -v` `namecheck`, and ledgers the
> platform-dependent cases.
>
> **T14.6 is the hostile-output-tree boundary** (`docs/zic-hostile-output-tree.md`) — a typed-status
> ledger: pre-planted file/symlink/dir at the output leaf fail closed and are never written through
> (`hard_link`/`O_EXCL`); the concurrent parent-component-swap *race* is honestly `NotClaimed` (needs
> `openat`-style hardening → T17/T20). **T14 is CLOSED** (`reports/t14-close-receipt.md`). (Gate: **360
> tests** · CORE.1 **341/0/0**.)
>
> **Category discipline (from T12/T13):** admissibility failures are **lexical** (the byte/line stream
> isn't well-formed `zic` text) — distinct from **structural**, **semantic**, and
> **operational/materialization**. A `diagnostic_artifact`, never a claim about output. Same comparison
> rule: **class · severity · location before wording**; reference platform = `upstream_iana_2026b`.

## The line-cap trap (verified against the pinned source)

The instinct was to "fix" zic-rs's 2048-byte cap down to the manpage's 511. **Pinned the actual
source instead:** reference `zic.c` 2026b defines `#define _POSIX2_LINE_MAX 2048` and reads into
`char buf[min(_POSIX2_LINE_MAX, …)]`, erroring `"line too long"` in `inputline` when
`linelen == bufsize`. So the **reference cap is 2048, and zic-rs's `MAX_LINE_LEN = 2048` already
matches** — both reject at ≥ 2048 content bytes, accept ≤ 2047. **No change.** The "511" figure is a
stale/older manpage value; the pinned 2026b source is authoritative. (This is the same
read-the-source discipline that fixed `checkabbr` in T13.4.)

## Reference admissibility rules — pinned from `zic.c::inputline` / `getfields`

| Rule | Reference `zic` behaviour (pinned) | Layer | zic-rs current | Status |
|------|-------------------------------------|-------|----------------|--------|
| **line-length cap** | `inputline`: error `line too long` at `linelen == 2048` | lexical | `MAX_LINE_LEN = 2048`, error at ≥2048 | **match** (`ZIC017_OVERLONG_INPUT_LINE`) |
| **NUL byte** | `inputline`: error `NUL input byte`, exit | lexical | rejects → `ZIC016_NUL_INPUT_BYTE` | **match** |
| **missing final newline** | `inputline`: EOF with `linelen>0` → error `unterminated line`, **exit (fatal)** | lexical | **rejects → `ZIC021_UNTERMINATED_INPUT_LINE`** (fatal; reported on the final line) | **match (T14.2)** — *was* a leniency divergence at T14.1; tightened |
| **unterminated quote** (odd `"`) | `getfields`: error `Odd number of quotation marks`, exit | lexical | rejects → **`ZIC022_UNTERMINATED_QUOTE`** (was the generic `ZIC012`) | **match (T14.2)** |
| **`#` comment** (outside quotes) | `getfields`: `#` begins a comment → rest of line ignored | lexical | comment-aware (strips `#` to EOL) | **match** |
| **quoted `#` / whitespace** | inside `"…"`, `#`/space are literal | lexical | literal inside quotes | **match** |
| **whitespace field split** | `is_space` separates fields; leading ws skipped | lexical | whitespace-delimited | **match** |
| **lone `-` field** | `getfields`: a field that is exactly `"-"` → **empty string** (reserved placeholder) | lexical | treats bare `-` as empty/placeholder | **match** |
| **too many fields** | `getfields`: error `Too many input fields`, exit | lexical/structural | per-record field-count check (`ZIC002`) | match (different shape; T14.2 audit) |
| **unknown keyword** | `byword` → `input line of unknown type` | lexical | `ZIC013_UNKNOWN_LINE_TYPE` | **match** (T13.2) |
| **continuation without zone** | folded into "unknown type" | structural | `ZIC014_CONTINUATION_WITHOUT_ZONE` (finer) | intentional divergence (T13.2) |
| **stdin `-`** | filename `-` reads standard input | operational/source-origin | **not a claimed capability** (no `--input -`) | documented non-capability → T16 (`source_origin`) |

## Headline divergence: missing final newline — RESOLVED in T14.2

At T14.1 this was the one leniency divergence (both run on `Zone Etc/UTC 0 - UTC` with **no** trailing
`\n`): reference `zic` → `"unterminated line"`, **fatal**; zic-rs → compiled it. **T14.2 tightened
zic-rs to match** (the T14.1 executable witness was written *as lenient* precisely so this flip is
deliberate, not silent):

- **reference `zic`** → `"unterminated line"`, **fatal** (exit 1, nothing written);
- **zic-rs (T14.2)** → **`ZIC021_UNTERMINATED_INPUT_LINE`**, fatal, reported on the final line.

The rule (pinned from `inputline`): *a non-empty input whose final line is not newline-terminated is
fatal* — `inputline` errors at EOF when `linelen > 0`. zic-rs computes the mirror condition
(`!text.is_empty() && !text.ends_with('\n')`) and checks it **after** the per-line NUL/overlong checks,
so those take precedence within that same final line exactly as the byte-at-a-time reference does
(`tests/.../nul_takes_precedence_over_missing_newline`). An input ending in `\n` (empty final segment)
and empty input are both fine (clean EOF). Bucket 1 — implemented parity.

## Implementation substeps

- **T14.2 — admissibility tightening ✅ DONE** (first T14 *behaviour* change; reference-pinned;
  CORE.1 341/0/0; **341 tests**). Added two append-only lexical codes through the totality path
  (`layer()`/`span_precision()`/`default_severity()` exhaustive + the contract-metadata test, now
  ZIC001–**ZIC022**): **`ZIC021_UNTERMINATED_INPUT_LINE`** (missing final newline → fatal, matching
  `inputline`) and **`ZIC022_UNTERMINATED_QUOTE`** (odd quotes; zic-rs already failed closed but under
  the generic `ZIC012` — now a dedicated class, matching `getfields`'s "Odd number of quotation marks").
  The T14.1 witness assertions **flipped** from lenient/uncoded to fatal/coded; added class/location
  fixtures to `tests/diagnostic_parity.rs` (ClassLocationMatch vs `zic -v` on line 1) + lexer unit tests
  (precedence, last-line reporting, empty-input, valid-unchanged). **Field-count / too-many-fields audit:**
  zic-rs's per-record `ZIC002_INVALID_FIELD_COUNT` already fails closed; reference's `getfields` "Too many
  input fields" is a *lexer-arity* cap (`MAX_FIELDS`) vs zic-rs's *per-record* count — a documented
  shape divergence (both reject), not a parity gap; no new code. (`UnterminatedInputLine` chosen over
  `UnterminatedLine` as the class name — it is the rule reference names "unterminated line", and avoids
  confusion with the quote class.)
- **T14.3 — metamorphic source-ordering ✅ DONE** (contract-level; NO behaviour change; CORE.1 341/0/0;
  **347 tests**). `tests/metamorphic_ordering.rs` proves the `zic(8)` law (lines order-independent
  *except* continuations) — pinned to `zic.c`'s internal re-sorts (`associate()` → `qsort(rules, rcomp)`,
  `qsort(links)`, `qsort(attypes, atcomp)`). Permitted transformations (only what reference treats as
  equivalent — **not** a general formatter): permute whole logical records (a `Zone`+continuations is
  **one** record), add/remove comments + blank lines, normalize **inter-field** whitespace (leading
  whitespace untouched — it would make a command line a continuation; quoted fields left alone). Across
  4 deterministic permutations + comment/blank-line + whitespace variants, **every zone's compiled TZif
  bytes and the `(link_name,target)` set are byte-identical** (the strongest claim — not merely
  zdump-equivalent), while the **source bytes differ** (the order-sensitive identity axis, T12.3). Two
  guard tests: a **detached continuation is rejected** (`ContinuationWithoutZone`) — the law's one
  exception is enforced, not silently reordered; and a **diagnostic's class is stable** under permutation
  even as its line *number* moves (asserted as class-equivalence, not literal line — the metamorphic
  caution). 6 new tests.
- **T14.4 — pathology ledger ✅ DONE** → **`docs/zic-pathology-ledger.md`** + executable witness
  `tests/pathology_ledger.rs` (CORE.1 341/0/0; **350 tests**). Classified the major edge classes vs
  reference 2026b (negative/large SAVE · sub-hour · `24:00` · year-zero · big-year · far-past/`min` ·
  far-future/`max` · until-on-transition · duplicate no-op era — all **matched**). **Headline:** "two
  rules for same instant" (reference fatal) was a `debug_assert!` → **panic** in debug / invalid TZif in
  release; **fixed** with `ensure_strictly_increasing()` → fail-closed **`ZIC023_SIMULTANEOUS_TRANSITION`**
  (a panic-on-untrusted-input removed). Residuals recorded: multi-era same-instant (bucket 4); two `-v`
  warning gaps ("values over 24 hours" → T15; "file name contains byte" → **T14.5**).
- **T14.5 — `ZoneNamePathPolicy` ✅ DONE** → **`docs/zic-zone-name-path-policy.md`** + executable witness
  `tests/zone_name_path_policy.rs` (CORE.1 341/0/0; **354 tests**). Five distinct axes (logical-name
  validity · materialization safety · platform constraints · reference diagnostics · zic-rs divergence).
  The fatal `ZIC008` structural policy already matched reference's `namecheck`/`componentcheck` (empty ·
  absolute · `//` · trailing-`/` · `.`/`..`) + safer divergences (leading-`-`, NUL, UTF-8-required).
  **Added the portability-warning axis** the T14.4 ledger surfaced: **`ZIC024_ZONE_NAME_NONPORTABLE_BYTE`**
  (non-benign byte; mirrors `zic -v` "contains byte", fires on `Etc/GMT+5`'s `+`) +
  **`ZIC025_ZONE_NAME_OVERLENGTH_COMPONENT`** (>14-byte component), verbose-only, over zones + links.
  Non-UTF-8 / reserved-names / case-collisions **ledgered** (bucket 3/4, platform-deferred — byte-level
  caution honoured: platform behaviour ledgered, not universalized).
- **T14.6 — hostile-output-tree (TOCTOU) ✅ DONE** → **`docs/zic-hostile-output-tree.md`** + executable
  witness `tests/hostile_output_tree.rs` (CORE.1 341/0/0; **360 tests**). Typed-status ledger
  (`Covered`/`FailClosed`/`PlatformDependent`/`NotClaimed`/`RequiresOpenatStyleHardening`). **Demonstrated
  (tested):** a pre-planted **file/symlink/dir** at the output leaf **fails closed and is never written
  through** — the default publish is `hard_link` (`O_EXCL`: `EEXIST` if the leaf exists, *including a
  symlink, without following it*); even `--force` (`rename`) *replaces* the symlink rather than writing
  through it (victim untouched, verified); a file blocking a parent dir fails closed; T9 no-partial-install
  under normal errors holds. **Honestly NOT claimed:** a parent-component **symlink-swap race mid-run** —
  std's `create_dir_all`/`open`/`rename` resolve parent symlinks at syscall time; closing it needs
  fd-relative `openat`/`O_NOFOLLOW` (forbidden by `#![forbid(unsafe_code)]` or a new dep) → deferred to
  **T17/T20**, marked `RequiresOpenatStyleHardening`, not faked.
- **TZif producer-conformance validator** (RFC 9636 structural checks over every emitted file) is
  **T14/T15** (named in the T12 close receipt §14), complementing hostile-*input* with hostile-*output*
  self-checks.

## Acceptance (T14.1)

> T14.1 is accepted when zic-rs records the reference admissibility rules for hostile/malformed input
> from pinned `zic.c` / `zic(8)`, classifies each as lexical/structural/semantic/operational, identifies
> current zic-rs divergences (the line-cap reconciliation [match, no change] and the missing-final-newline
> leniency [→ T14.2]), and names the implementation substeps **without changing behaviour**. *(Met:
> inventory + an executable witness test, no compiler behaviour change; **336 tests**, CORE.1 341/0/0.)*

## Non-claims (T14.1)

- No new admissibility *behaviour* (this is inventory; tightening is T14.2+).
- No claim of **full** hostile-input/pathology coverage (the ledger is seeded, not exhaustive).
- No **vendor/platform** admissibility parity beyond the admitted `upstream_iana_2026b` oracle
  (per the T13 reference-platform matrix; e.g. AIX's pre-1901 32-bit `time_t` boundary is a separate,
  unadmitted axis).
- Diagnostics remain `diagnostic_artifact`s — admissibility classification proves the tool *rejects*
  ill-formed input at the right layer, never anything about compiled output.
