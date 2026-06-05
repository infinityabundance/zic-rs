# Warning & diagnostic parity (T13) — taxonomy inventory (T13.1)

> **T13.1 is reference-first inventory — no behaviour change.** It defines *how* zic-rs will compare
> its diagnostics to reference `zic -v` (by layer/class/severity/origin/location **before** exact
> wording), records the real reference surface empirically, and maps zic-rs's current diagnostics into
> the taxonomy. It does **not** rename codes, change messages, or add warnings. (T13.1 was doc-only — gate
> unchanged from T12.5d at 319 tests; **T13.2 then emitted the coded classes → 321 tests, CORE.1 341/0/0**
> — see the T13.2 section below.)
>
> **Inherits the T12 evidence-category doctrine** (`reports/t12-close-receipt.md`): a diagnostic is a
> **`diagnostic_artifact`** — it proves *the tool noticed something*, **not** that the compiled output
> has a particular semantic shape and **not** that a source byte means a particular thing. Diagnostics
> are a **compatibility surface, not UX copy.**

## Why diagnostics are a compatibility surface (not polish)

`zic -v` behaviour is something scripts, packagers, and maintainers *depend on*: which inputs warn,
which fail, and roughly where. A memory-safe replacement earns trust by **matching the reference
tool's observable diagnostic behaviour** — same layer, same class, same location, compatible severity
— not by printing prettier text. **Diagnostic-wording improvements are explicitly NOT a T13 goal.**

## The comparison method (the order that matters)

zic-rs diagnostics are compared to reference `zic` **in this priority order** — never by `stderr`
substring first:

1. **layer** — lexical → structural → semantic → warning (see below);
2. **class** — the *rule violated* (a stable name, e.g. `NulByteInInput`), never the current English text;
3. **severity** — fatal error vs non-fatal `-v` warning vs info;
4. **source_origin** — where the bytes came from (see the model below);
5. **source_location** — file + line (+ optional column/span);
6. **related entity** — the named thing the diagnostic is about (zone/rule/link name) and any
   cross-referenced location (e.g. the *original* definition for a duplicate);
7. **wording** — exact message text, compared **last** and tracked in a separate *wording ledger*
   (reference wording changes across releases and vendor builds — see "non-claims").

> **Class-naming rule:** a class names the **rule violated**, not the text printed. `InvalidSaveField`
> / `ContinuationWithoutZone` / `LinkTargetMissing` / `AbbreviationPolicyViolation` — never
> `WarningTextContainsInvalidSave`. This future-proofs the contract when reference wording changes.

## Diagnostic layers (the taxonomy)

Layering preserves the T12 categories: a **lexical/admissibility** failure (the byte stream isn't even
well-formed text) is a different kind of fact from a **semantic** timezone error. `zic(8)`/RFC 8536
constrain the *input text* itself (newline-terminated lines, a line-length cap, no NUL bytes) — those
are admissibility concerns, not timezone semantics.

| Layer | What it catches | Example rule-named classes |
|-------|-----------------|----------------------------|
| **lexical / admissibility** | the byte/line stream isn't well-formed `zic` text | `NulByteInInput` · `OverlongInputLine` · `UnterminatedQuote` · `InvalidFieldCount` · `UnknownLineType` |
| **structural** | lines are well-formed but don't assemble into a valid record graph | `ContinuationWithoutZone` · `DuplicateZone` · `UnresolvedRule` · `LinkTargetMissing` · `LinkCycle` |
| **semantic** | a field/value is individually invalid or unrepresentable | `InvalidMonthName` · `InvalidUntil` · `InvalidSaveField` · `InvalidDayRule` · `InvalidTimeSuffix` · `TransitionOverflow` |
| **warning** (`-v`) | accepted, but portability/obsolescence/reader-compat hazards | `AbbreviationPolicyViolation` · `ObsoleteSyntax` · `PortabilityHazard` · `FutureTailHazard` · `ReaderCompatHazard` |

## `source_origin` model (diagnostics need honest locations)

Reserved in the T12 close; **needed early in T13** because a diagnostic's location is only as honest as
its origin. A diagnostic for stdin must not pretend to have a path; one for a generated `.zi` must not
masquerade as user source.

```text
source_origin ∈ { path | stdin | generated | admitted_reference | policy_input }
  path   → display = "<file>",   line, optional column
  stdin  → display = "<stdin>",  line  (path is "-", per zic(8); not a real path)
  generated / admitted_reference / policy_input → display names the artifact, never a user-source path
```

Today zic-rs diagnostics carry `path` + 1-based `line` + optional byte `span` (see
`src/diagnostics.rs`). `--input -` / stdin is **not currently a claimed capability** — it is a
documented non-capability until implemented (owning milestone T16); when added, it enters this model
rather than fabricating a path.

## Reference `zic -v` surface — empirically captured (tzcode **2026b**, local build `2026b-dirty`)

Captured by running the local reference `zic -v` on crafted malformed inputs. **Wording is pinned to
this build** and is expected to drift across releases/vendors — which is exactly why wording is
compared last. Location shape is `"<file>", line N:` (quoted name, comma); zic-rs uses `<file>:N:`.

| Input | Reference `zic` message (verbatim) | Layer | Class | Severity |
|-------|-------------------------------------|-------|-------|----------|
| NUL byte in a line | `"f", line 1: NUL input byte` | lexical | `NulByteInInput` | error |
| bare `Zone` | `"f", line 1: wrong number of fields on Zone line` | lexical | `InvalidFieldCount` | error |
| unknown keyword | `"f", line 1: input line of unknown type` | lexical | `UnknownLineType` | error |
| continuation, no prior `Zone` | `"f", line 1: input line of unknown type` | lexical→structural\* | `ContinuationWithoutZone` | error |
| two `Zone A` | `"f", line 2: duplicate zone name A (file "f", line 1)` | structural | `DuplicateZone` (+ related entity `A`, + ref location) | error |
| ruleless zone using `R`/`%s` | `"f", line 1: invalid saved time` **and** `"f", line 1: %s in ruleless zone` (two diagnostics) | semantic | `InvalidSaveField` + `PercentSInRulelessZone` | error |
| `Link Nowhere Here` | `zic: Can't link …/Nowhere to …/Here: No such file or directory` | **operational** (materialization, errno) | `LinkTargetMissing` | error |
| bad `UNTIL` month | `"f", line 1: invalid month name` | semantic | `InvalidMonthName` | error |
| bad `SAVE` (`99:99`) | `"f", line 1: invalid saved time` | semantic | `InvalidSaveField` | error |
| 11-char abbreviation | `warning: "f", line 1: time zone abbreviation has too many characters (ABCDEFGHIJK)` | **warning** | `AbbreviationPolicyViolation` | warning |

\* Reference `zic` reports a no-prior-`Zone` continuation as "input line of unknown type" (it never
classifies it as a *continuation*); zic-rs may classify it more precisely as `ContinuationWithoutZone`
— a **deliberate divergence to track**, not a wording mismatch. Two observations that shape T13:
(a) one source line can yield **multiple** diagnostics (the ruleless-`%s` case); (b) **link-target
errors surface at materialization with errno text** — a different (operational) layer from source
parsing, and not a `-v` source diagnostic.

## zic-rs's current diagnostic surface (mapped into the taxonomy)

zic-rs already emits **structured, coded** diagnostics (`src/diagnostics.rs`: `DiagnosticCode` +
`Severity` + file/line/span/suggestion; rendered `file:line: severity[CODE]: message`). Mapping the
existing 12 codes onto the layers:

| Code | Layer | Notes vs reference |
|------|-------|--------------------|
| `ZIC002_INVALID_FIELD_COUNT` | lexical | ↔ `wrong number of fields` |
| `ZIC004_AMBIGUOUS_NAME_ABBREVIATION` | lexical | zic-rs-specific (prefix-matching of `Rule`/`Zone`/`Link`); reference says `input line of unknown type` |
| `ZIC003_INVALID_MONTH` | semantic | ↔ `invalid month name` |
| `ZIC005_INVALID_DAY_RULE` | semantic | ↔ `ON`-field errors |
| `ZIC006_INVALID_TIME_SUFFIX` | semantic | ↔ `AT`/`SAVE` suffix |
| `ZIC012_INVALID_VALUE` | semantic | offsets/names/values |
| `ZIC013_UNKNOWN_LINE_TYPE` | lexical | **T13.2** — unrecognised keyword in command position ↔ reference `input line of unknown type` |
| `ZIC014_CONTINUATION_WITHOUT_ZONE` | structural | **T13.2** — indented stray continuation; finer than reference (intentional divergence) |
| `ZIC015_DUPLICATE_ZONE` | structural | **T13.2** — ↔ reference `duplicate zone name N`; carries the original line (detection newly added) |
| `ZIC016_NUL_INPUT_BYTE` | lexical | **T13.2** — ↔ reference `NUL input byte` (recoded from `ZIC012`) |
| `ZIC017_OVERLONG_INPUT_LINE` | lexical | **T13.2** — line-length cap (recoded from `ZIC012`); cap-vs-reference reconciliation is T14 |
| `ZIC018_ABBREVIATION_POLICY_VIOLATION` | warning (non-fatal) | **T13.3/T13.4** — abbreviation **length** outside 3–6 (`> 6` "too many"; `< 3` "fewer than 3", the latter `-v`-gated in reference); the first warning-severity class, collected (not failed) |
| `ZIC019_ABBREVIATION_NOT_POSIX` | warning (non-fatal) | **T13.4** — abbreviation has a non-`[A-Za-z0-9+-]` character ↔ reference `time zone abbreviation differs from POSIX standard`; a distinct rule (lowercase is fine) |
| `ZIC001_UNSUPPORTED_DIRECTIVE` | semantic (scope) | **intentional divergence**: zic-rs *fails closed* on a declared-unsupported construct where reference `zic` may compile it — a bucket-3 safety divergence, not a parity gap (see `docs/differences-from-reference-zic.md`). **T13.2 unconflated this**: a genuinely *unknown* keyword is now `ZIC013`, not `ZIC001` |
| `ZIC007_UNSUPPORTED_YEAR_TYPE` | semantic (scope) | same posture |
| `ZIC009_TOO_MANY_TRANSITIONS` | semantic (resource) | zic-rs resource cap (`MAX_TRANSITIONS`); reference has no identical cap |
| `ZIC010_UNSUPPORTED_LEAP_SECONDS` | semantic (scope) | superseded by T11 (`-L` now supported); code retained as reserved |
| `ZIC008_OUTPUT_PATH_TRAVERSAL` | operational (output safety) | zic-rs safety reject; no reference equivalent (bucket-3 divergence) |
| `ZIC011_REFERENCE_ZIC_MISMATCH` | meta (oracle) | emitted by `compare`, not by parsing |

**Gaps (status after T13.4):** **all closed** for the inventoried set. `UnknownLineType` (ZIC013),
`ContinuationWithoutZone` (ZIC014), `DuplicateZone` (ZIC015), `NulByteInInput` (ZIC016),
`OverlongInputLine` (ZIC017) emit as coded *error* diagnostics (T13.2); the abbreviation warnings emit
through the non-fatal channel (T13.3/T13.4): **`AbbreviationPolicyViolation` (ZIC018)** — length
outside 3–6 — and **`AbbreviationNotPosix` (ZIC019)** — non-POSIX characters. The abbreviation checks
are a **direct port of `zic.c::checkabbr`** (one warning per abbreviation by precedence; see T13.4
below). Further `-v` classes (e.g. "values over 24 hours", "pre-1994/2007 clients") are parse-time /
version-tail observations needing machinery beyond the finished `TzifData` → **T14/T15**.

## T13.2 — emitted coded classes + comparison harness + wording ledger

T13.2 made the comparison **architecture real** (not stderr-substring matching) and emitted the
missing coded classes where the parser/lexer already had the information.

**Code changes (additive; fail-closed posture unchanged; CORE.1 341/0/0):**

- `src/diagnostics.rs` — five new codes: `ZIC013_UNKNOWN_LINE_TYPE` · `ZIC014_CONTINUATION_WITHOUT_ZONE`
  · `ZIC015_DUPLICATE_ZONE` · `ZIC016_NUL_INPUT_BYTE` · `ZIC017_OVERLONG_INPUT_LINE`. (Codes only ever
  append; existing codes never shift meaning.)
- `src/source/parser.rs` — the `record_keyword == None` path was split **by column** (the existing
  `Field.col`): an indented line (`col > 0`) is `ContinuationWithoutZone`; a command-position line
  (`col 0`) is `UnknownLineType`. This **unconflates** the generic `UnsupportedDirective` (ZIC001),
  which now means *recognised-but-deliberately-unsupported* only. Added **duplicate-zone detection** (a
  name → first-line map, seeded from prior files so cross-file dups are caught) → `DuplicateZone`.
- `src/source/lexer.rs` — NUL and overlong-line recoded from the generic `InvalidValue` to
  `NulByteInInput` / `OverlongInputLine`.

**Comparison harness** (`tests/diagnostic_parity.rs`) — compares zic-rs vs reference `zic -v`
**by canonical class then source line, before wording**, emitting the verdict vocabulary
(`ClassLocationMatch` · `ClassMatchLocationDiff` · `IntentionalDivergence` · `LayerMatchClassGap` ·
`ReferenceOnly` · `ZicRsOnly`). zic-rs's own class/line is always asserted; the reference half
**auto-skips** when `zic` is absent (oracle-availability). Verdicts on the current fixtures (against
tzcode 2026b):

| Fixture | zic-rs class | reference class | line | verdict |
|---------|--------------|-----------------|------|---------|
| unknown keyword (`Frobnicate …`) | `UnknownLineType` | `UnknownLineType` | 1 | **ClassLocationMatch** |
| stray continuation (indented `1:00 …`) | `ContinuationWithoutZone` | `UnknownLineType` | 1 | **IntentionalDivergence** (zic-rs is finer; documented) |
| duplicate `Zone A` | `DuplicateZone` | `DuplicateZone` | 2 | **ClassLocationMatch** |
| NUL byte | `NulByteInInput` | `NulByteInInput` | 1 | **ClassLocationMatch** |
| bare `Zone` | `InvalidFieldCount` | `InvalidFieldCount` | 1 | **ClassLocationMatch** |
| 7-char abbreviation (`… ABCDEFG`) — **warning** (T13.3) | `AbbreviationPolicyViolation` | `AbbreviationPolicyViolation` | 1 | **ClassLocationMatch** |

### T13.3 — warning-collection channel (the first non-fatal class)

A diagnostic does not have to be fatal. Reference `zic` *warns* (always-on, non-fatal) for an
abbreviation longer than the portable maximum and **continues**; a replacement that turned that into a
hard error would be wrong. T13.3 adds the first **warning** path:

- **Channel:** warnings are collected into `CompileReport.diagnostics` with `Severity::Warning`
  (already printed to stderr by the CLI, like any diagnostic) — **structured artifacts, never
  bolted-on stderr text**. `plan::run` appends them; nothing fails, the exit status is unchanged.
- **`AbbreviationPolicyViolation` (ZIC018):** `collect_abbreviation_warnings` inspects each compiled
  zone's finished `TzifData.types` and warns once per distinct abbreviation longer than
  **`MAX_PORTABLE_ABBR_LEN = 6`** (pinned empirically: reference is silent at ≤ 6, warns at ≥ 7),
  located at the zone's source line. **Pure observation over `data`** — it never mutates the compiled
  bytes, type table, or exit status.
- **CORE.1 untouched:** the canonical zones in `tzdata.zi` 2026b have no abbreviation longer than 6, so
  the sweep emits **zero** abbreviation warnings and the behaviour/byte output is unchanged
  (`warning_collection_does_not_change_core1_output` asserts the compiled bytes are byte-identical to
  `compile_zone_to_bytes`). **`warning collection ≠ compile failure.`**
- **Tests** (`tests/diagnostic_parity.rs`): `abbreviation_too_long_emits_warning_not_error` (7-char →
  one warning + still compiles; 6-char → silent), `abbreviation_policy_violation_matches_reference_class_location`
  (ClassLocationMatch vs reference, auto-skips without `zic`), `warning_collection_does_not_change_core1_output`,
  `warning_wording_recorded_not_optimized`, `reference_absent_auto_skips_warning_harness`. **326 tests.**

### T13.4 — widened abbreviation surface, ported from `zic.c::checkabbr`

T13.4 widens the abbreviation warning surface to the **full** reference rule, and does so by directly
porting `zic.c::checkabbr` so the *class* matches exactly. Three subtleties (pinned against tzcode
2026b, verified by reading the source):

- **Length 3–6** (extends ZIC018): `< 3` → "fewer than 3 characters"; `> 6` → "too many characters".
- **POSIX character set** (new **ZIC019** `AbbreviationNotPosix`): any character outside
  `[A-Za-z0-9+-]` → "differs from POSIX standard". **Lowercase is fine** (alphanumeric); only the
  *set*, not case, is checked.
- **Exactly one warning per abbreviation, by precedence.** `checkabbr` walks the leading conforming
  run and assigns a single `mp` through sequential `if`s, so the **last** match wins:
  **non-POSIX > too-many > fewer-than-3**. So `"A_"` (2 chars + `_`) warns **only** `NotPosix`, never
  *also* "fewer than 3" — an initial independent-checks version got this wrong and the harness caught
  it. Length is measured over the **conforming prefix** (`cp - string`), which equals the full length
  for a conforming abbreviation and is moot for a non-conforming one (POSIX wins).
- **Verbosity divergence (honest):** in reference `zic`, **"fewer than 3" is `noise`(`-v`)-gated**;
  "too many" and "differs from POSIX" are always-on. zic-rs has **no verbosity tiers** — it always
  collects all three (its diagnostic set corresponds to `zic -v`). This is a *verbosity* divergence,
  **not** a class/location one; the harness therefore compares against `zic -v`. Recorded, not hidden.

Harness fixtures (`abbreviation_warnings_by_class_and_location`): `too_long` → `AbbreviationPolicyViolation`,
`too_short` → `AbbreviationPolicyViolation`, `non_posix` → `AbbreviationNotPosix`,
`non_posix_wins_over_short` (`A_`) → `AbbreviationNotPosix` *only* (precedence) — each ClassLocationMatch
vs `zic -v`. **327 tests.**

### Wording ledger (compared LAST — recorded, not optimised)

Wording is **not** asserted (it drifts across releases/vendors); recorded here for transparency.

| Class | zic-rs wording | reference `zic` wording (tzcode 2026b) | status |
|-------|----------------|-----------------------------------------|--------|
| `UnknownLineType` | `input line of unknown type beginning with "X"` | `input line of unknown type` | wording_diff_only (zic-rs adds the token) |
| `ContinuationWithoutZone` | `continuation line "X" has no open zone to continue` | *(folded into "input line of unknown type")* | intentional_divergence |
| `DuplicateZone` | `duplicate zone name "A" (originally defined at line N)` | `duplicate zone name A (file "f", line N)` | wording_diff_only (same facts, different shape) |
| `NulByteInInput` | `line contains a NUL byte` | `NUL input byte` | wording_diff_only |
| `InvalidFieldCount` | *(zic-rs message)* | `wrong number of fields on Zone line` | wording_diff_only |
| `AbbreviationPolicyViolation` (too many) | `time zone abbreviation "X" has too many characters (N > 6)` | `time zone abbreviation has too many characters (X)` | wording_diff_only (zic-rs names the count + threshold) |
| `AbbreviationPolicyViolation` (fewer than 3) | `time zone abbreviation "X" has fewer than 3 characters (N)` | `time zone abbreviation has fewer than 3 characters (X)` *(`-v`-gated)* | wording_diff_only; **verbosity divergence** (zic-rs always-on, reference `-v`-only) |
| `AbbreviationNotPosix` | `time zone abbreviation "X" differs from the POSIX standard (non-alphanumeric, non-+/- character)` | `time zone abbreviation differs from POSIX standard (X)` | wording_diff_only |

**Wording improvements are not a T13 goal** — class/location compatibility is. Narrowing wording (if
ever) is a separate, late, opt-in pass.

## T13.5 — diagnostic contract completion

T13.5 completes the **diagnostic contract** — the parts that make T13's claims auditable and bounded.
These are diagnostic-contract items, so they live **in T13**, not deferred to T16. Where it falls out
naturally they are *machine-checkable* (accessors + tests), not just prose.

> **The binding rule (the whole contract in one line):** *a diagnostic claim is admitted by **class,
> severity, source origin, source location, and reference platform** — **before wording is
> considered**.* (Wording is the last axis, ledgered only.)

### 1. Diagnostic layers — now machine-checked (+ the operational/materialization layer)

The taxonomy gains a sixth layer and a code-level accessor (`DiagnosticCode::layer()`), so a code can
never be added without classifying it (the `match` is exhaustive → compile error otherwise):

| Layer | Codes |
|-------|-------|
| **lexical/admissibility** | `ZIC002` field-count · `ZIC004` ambiguous-keyword · `ZIC013` unknown-line · `ZIC016` NUL · `ZIC017` overlong |
| **structural** | `ZIC014` continuation-without-zone · `ZIC015` duplicate-zone |
| **semantic** | `ZIC001` unsupported-directive · `ZIC003` month · `ZIC005` day-rule · `ZIC006` time-suffix · `ZIC007` year-type · `ZIC009` too-many-transitions(cap) · `ZIC010` leap · `ZIC012` value |
| **warning** (non-fatal) | `ZIC018` abbrev-length · `ZIC019` abbrev-POSIX · `ZIC020` transition-count · `ZIC024` name-nonportable-byte · `ZIC025` name-overlength-component · `ZIC026` value-over-24h (all verbose-only except `ZIC018`/`ZIC019` too-many/POSIX) |
| **operational/materialization** | `ZIC008` output-path-traversal *(and the as-yet-uncoded no-clobber / no-create-dirs / mode-failure / alias-map-validation operational diagnostics — **operational, not source `zic -v` diagnostics**)* |
| **meta** | `ZIC011` reference-`zic`-mismatch (emitted by `compare`, not parsing) |

**Operational/materialization diagnostics are diagnostics, but not source `-v` diagnostics** — they
are caused by output materialization (link-target-missing surfaces here with errno, path traversal,
no-clobber, `--no-create-dirs`, `--mode`, `AliasMap::validate`). Recording them in their own layer is
how a reviewer sees they were *placed*, not missed.

### 2. Verbosity model (quiet vs `zic -v`)

Reference `zic` gates some warnings behind `noise` (`-v`); T13.4 found this empirically. The model is
**per-diagnostic, not per-code** — `checkabbr` gates "fewer than 3" behind `-v` while "too many" /
"differs from POSIX" (same `ZIC018`/`ZIC019` family) are always-on. So `Diagnostic` carries a
`DiagnosticVerbosity` (`AlwaysOn` | `VerboseOnly`), set at emission (`.verbose_only()`):

| Warning | Verbosity (vs reference) |
|---------|--------------------------|
| `ZIC018` "too many characters" | `AlwaysOn` |
| `ZIC018` "fewer than 3 characters" | **`VerboseOnly`** (reference `-v`-gated) |
| `ZIC019` "differs from POSIX standard" | `AlwaysOn` |
| `ZIC020` transition-count | `VerboseOnly` (reference `-v`-gated) |

**Status (T13.6 ✅):** the filter is **implemented** — `compile --verbose`/`-v`. The report still
*collects* every diagnostic (programmatic consumers + the comparison harness see all); the CLI prints
`AlwaysOn` always and `VerboseOnly` only under `-v`, so **default output matches `zic` and `-v` matches
`zic -v`**. It never affects compiled output, exit status, or what is collected — only what is printed.

### 3. Transition-count warning surface (`ZIC020`)

Reference `zic` (`-v`/`noise`-gated, on the *emitted* stream) warns: **`1200 < timecnt`** →
"pre-2014 clients may mishandle more than 1200 transition times"; **`TZ_MAX_TIMES < timecnt`** →
"reference clients mishandle more than N transition times" (and `TZ_MAX_LEAPS < leapcnt` for leaps).
**`ZIC020_TOO_MANY_TRANSITIONS_FOR_LEGACY_CLIENT`** is coded + classified (warning layer,
`VerboseOnly`) and, **as of T13.6, emitted** when a zone's emitted `timecnt > 1200` — a pure
observation over `TzifData.transitions.len()` (byte-output preserving; **0 on canonical `tzdata.zi`**).
*Style note:* the count is the emitted count, so the fat default can cross the threshold where
reference's slim default would not; it is `VerboseOnly` (quiet by default) and concerns the *emitted*
stream, so this is consistent. The `TZ_MAX_TIMES`/`TZ_MAX_LEAPS` higher-threshold variants stay
reserved (a deeper TZif/conformance concern → T15).

### 4. Diagnostic-code stability policy

The `ZIC` codes are a **contract**:

- **Append-only.** New conditions get new numbers; gaps are reserved. (ZIC013–ZIC020 appended; ZIC001–ZIC012 never shifted.)
- **Never reused** — a retired number stays retired.
- **Class meaning is stable**; the class names the **rule violated**, not the current English message.
- **Wording may change** (compared last; ledgered).
- A **severity change** (warning↔error, or a verbosity-gating change) needs an **explicit compatibility note**, never a silent flip.
- The **stable comparison surface** is the structured triple **(class, location, severity)** — what scripts and the harness match on; message text is *not* part of it.

*(The consumer-facing restatement of this is T19/`TRUST.md`; here it is the rule T13 follows.)*

### 5. Source-span precision policy + audit

Locations are **not all equal**. Precision levels: `file_only` · `line` · `line_column` · `span` ·
`related_entity_location` · `operational_path` · `policy_input_location`. Audit of the current classes:

| Code | Current precision | Target |
|------|-------------------|--------|
| `ZIC013` unknown-line · `ZIC017` overlong · `ZIC018`/`ZIC019` abbrev | `line` (abbrev = zone source line) | `line` ok; column = future |
| `ZIC014` continuation-without-zone | `line` (+`Field.col` available) | `line_column` (future) |
| `ZIC015` duplicate-zone | `line` + **`related_entity_location`** (original definition line) ✅ | — |
| `ZIC016` NUL · `ZIC002` field-count | `line` | `line`/byte-offset (future) |
| `ZIC008` path-traversal | `operational_path` | — |

`related_entity_location` is in the model (duplicate-zone); **full per-class column/span precision is
not yet populated** — recorded as a refinement, not claimed complete.

### 6. Reference-platform diagnostic matrix (vendor/platform — T13-owned)

**Upstream IANA / tzcode is the diagnostic oracle.** Vendor/platform `zic` builds differ by platform
constraints (e.g. AIX's pre-1901 32-bit `time_t` limit), obsolete/extra options (e.g. Solaris `-s`),
local patches, installed tzdata, libc assumptions, and docs — and tzdb NEWS shows `zic`'s own
`-v`/manpage behaviour shifts across releases. So vendor parity is a **separate oracle axis**.

**Doctrine (mirrors T12.5a.3's release-admission matrix, on a platform axis): vendor/platform
diagnostic parity is supported through admission, never by assumption.** *Support every vendor
eventually, by admission; never claim every vendor by default.* T13 **owns** this diagnostic matrix
(broader release ecology is still T16); each admitted row records `zic` binary identity · `--version` ·
OS/platform · tzdb release/source hash · doc source · diagnostic fixtures run · known divergences ·
oracle status. **Seed:**

| reference_platform | status |
|--------------------|--------|
| `upstream_iana_2026b` | **admitted · active diagnostic oracle** |
| `linux_glibc` · `bsd` · `macos` · `aix` · `solaris_illumos` · `vendor_other` | pending |

No vendor/platform behaviour is claimed until that platform is admitted and tested (class/location
first, vendor wording last). Admitting further platforms (running their `zic` over the fixtures) is
**T13.6 / T16** as binaries become available.

## Non-claims (T13.1, still in force after T13.2/T13.3/T13.4/T13.5)

T13.1 does **not** claim:

- byte-exact `stderr` parity, or matched exact wording;
- that *all* `zic` warnings/errors are matched;
- parity across all platforms, locales, or reference-`zic` versions (the surface above is one build:
  `2026b-dirty`; **upstream IANA is the oracle**, vendor builds are a separate axis — RFC `reference_platform` doctrine);
- that a diagnostic proves anything about the *compiled output* (a `diagnostic_artifact` proves only
  that the tool noticed something).

**Honest claim:** *T13.1 establishes a diagnostic-parity taxonomy by layer/class/severity/origin/
location and compares diagnostics in that order before exact wording; it records the reference `zic -v`
surface for the pinned 2026b build and maps zic-rs's current coded diagnostics onto it.*

## Handoff (rest of T13, and reserved tooling)

- **T13.2 ✅** — emitted the missing coded classes (ZIC013–017) where the parser/lexer had the info,
  unconflated `UnknownLineType` from `UnsupportedDirective`, added duplicate-zone detection, and built
  the class/location-first comparison harness (`tests/diagnostic_parity.rs`) + the wording ledger above.
- **T13.3 ✅** — added the **warning-collection channel** (`CompileReport.diagnostics` +
  `Severity::Warning`, printed by the CLI) and the first warning class **`AbbreviationPolicyViolation`
  (ZIC018)** (abbreviation `> 6` chars; pure observation over the finished `TzifData`, CORE.1 untouched,
  byte output unchanged); widened the harness to warning-severity (ClassLocationMatch vs reference) +
  the wording ledger. **warning collection ≠ compile failure.**
- **T13.4 ✅** — widened the abbreviation surface to the full `checkabbr` rule: extended ZIC018 length
  to `< 3`, added **ZIC019 `AbbreviationNotPosix`**, and matched `zic`'s **single-warning precedence**
  (non-POSIX > too-many > fewer-than-3) + the **`-v`-gating of "fewer than 3"** (recorded as a verbosity
  divergence). 327 tests; CORE.1 untouched.
- **T13.5 ✅** — diagnostic-contract completion (the section above): machine-checked **layer** accessor
  + the **operational/materialization** layer; the **verbosity model** (`DiagnosticVerbosity`, per-diagnostic,
  fewer-than-3 = `VerboseOnly`); the **`ZIC020`** transition-count surface (coded + classified, emission
  T13.6); the **diagnostic-code stability policy**; the **span-precision** levels + audit; and the
  **reference-platform diagnostic matrix** (upstream IANA = oracle; vendors admitted, never assumed).
- **T13.6 ✅** — implemented the mechanically-available pieces: the **verbosity filter** is real
  (`compile --verbose`/`-v`; quiet prints `AlwaysOn` only, `-v` prints `VerboseOnly` too — matching
  `zic` vs `zic -v`); **`ZIC020` is emitted** (transition count > 1200, `VerboseOnly`, byte-output
  preserving — 0 on canonical `tzdata.zi`); `DiagnosticCode::span_precision()` makes the span audit
  **machine-checked** (exhaustive, like `layer()`); a contract-metadata totality test pins ZIC001–**ZIC026**
  append-ordering + per-code layer/span; and the **reference-platform diagnostic matrix is seeded** as a
  test (upstream admitted · local `zic` captured-or-`skipped_with_reason` · vendor rows
  documentation-only/unavailable, never admitted-by-assumption). **335 tests.**
- **T13.close ✅** — the closure receipt **`reports/t13-close-receipt.md`** (ZIC001–020 table · covered
  implementation · intentional divergences · non-claims). **T13 CLOSED.** Further `-v` classes:
  **"values over 24 hours" → ✅ `ZIC026`** (T15.5-remainder; a parse-time magnitude warning over
  STDOFF/SAVE/AT/UNTIL, verbose-only, pinned to `zic.c::gethms`); "pre-1994/2007 clients" → T15
  version-tail (still reserved — needs version-history machinery beyond the finished `TzifData`).
- **Reserved for T14** (per the T12 close): the input-text **admissibility layer** (511/2048 line-cap
  reconciliation against pinned `zic.c` — *verify before changing*, the cap is currently 2048; NUL;
  newline; PPCS encoding), the **metamorphic source-ordering** suite, and the **pathology ledger**.
- **Reserved for T15**: `zdump` **semantic-witness** tests, the **oracle-availability** policy, and the
  one-line machine status + `negative_capabilities` array.
