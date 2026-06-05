# T13 — close receipt (warning & diagnostic parity)

> **Status: T13 CLOSED.** The diagnostic *contract* is now **executable**, not just documented. This is
> a **documentation** receipt — no new behaviour, code, or schema beyond the sealed T13.6 surface. Last
> code work: T13.6; gate at close — `fmt` · `clippy --all-targets -D warnings` · **335 tests** · `doc`
> · **CORE.1 341/0/0** — unchanged by this doc-only receipt.
>
> **The binding rule (holds in code):** *a diagnostic claim is admitted by **class, severity, source
> origin, source location, and reference platform** — before wording is considered.* Wording is the
> last axis, ledgered only (`docs/zic-warning-parity.md`).

## Diagnostic table — ZIC001–ZIC020

Reference platform for every row: **`upstream_iana_2026b`** (the admitted diagnostic oracle; see the
reference-platform matrix below). Wording status is `ledgered` (compared last, never asserted).
Per-code metadata is **machine-checked** (`DiagnosticCode::layer()` / `span_precision()` /
`default_severity()` are exhaustive; verbosity is a required per-`Diagnostic` field).

| Code | Layer | Severity | Verbosity | Span precision | Verdict vs reference |
|------|-------|----------|-----------|----------------|----------------------|
| `ZIC001_UNSUPPORTED_DIRECTIVE` | semantic | error | always-on | line | **divergence (bucket 3)** — fail-closed where reference may compile |
| `ZIC002_INVALID_FIELD_COUNT` | lexical | error | always-on | line | class/location match |
| `ZIC003_INVALID_MONTH` | semantic | error | always-on | line | class/location match |
| `ZIC004_AMBIGUOUS_NAME_ABBREVIATION` | lexical | error | always-on | line | zic-rs-specific (reference: "unknown type") |
| `ZIC005_INVALID_DAY_RULE` | semantic | error | always-on | line | class/location match |
| `ZIC006_INVALID_TIME_SUFFIX` | semantic | error | always-on | line | class/location match |
| `ZIC007_UNSUPPORTED_YEAR_TYPE` | semantic | error | always-on | line | divergence (bucket 3) — scope |
| `ZIC008_OUTPUT_PATH_TRAVERSAL` | operational/materialization | error | always-on | operational_path | zic-rs safety (no reference equiv, bucket 3) |
| `ZIC009_TOO_MANY_TRANSITIONS` | semantic (resource) | error | always-on | line | zic-rs cap (bucket 3) |
| `ZIC010_UNSUPPORTED_LEAP_SECONDS` | semantic (scope) | error | always-on | line | reserved (superseded by T11 `-L`) |
| `ZIC011_REFERENCE_ZIC_MISMATCH` | meta | error | always-on | file_only | meta (emitted by `compare`, not parsing) |
| `ZIC012_INVALID_VALUE` | semantic | error | always-on | line | class/location match |
| `ZIC013_UNKNOWN_LINE_TYPE` | lexical | error | always-on | line | **class/location match** |
| `ZIC014_CONTINUATION_WITHOUT_ZONE` | structural | error | always-on | line | **intentional divergence** (finer than reference "unknown type") |
| `ZIC015_DUPLICATE_ZONE` | structural | error | always-on | related_entity_location | **class/location match** (+ related original line) |
| `ZIC016_NUL_INPUT_BYTE` | lexical | error | always-on | line | **class/location match** |
| `ZIC017_OVERLONG_INPUT_LINE` | lexical | error | always-on | line | match; line-cap-vs-reference reconciliation → T14 |
| `ZIC018_ABBREVIATION_POLICY_VIOLATION` | warning | warning | always-on (too-many) / **verbose-only** (fewer-than-3) | line | **class/location match** vs `zic -v` |
| `ZIC019_ABBREVIATION_NOT_POSIX` | warning | warning | always-on | line | **class/location match** vs `zic -v` |
| `ZIC020_TOO_MANY_TRANSITIONS_FOR_LEGACY_CLIENT` | warning | warning | **verbose-only** | line | emitted (`timecnt>1200`, emitted-stream; style-dependent) |

## Covered implementation

- **Class/location-first comparison harness** (`tests/diagnostic_parity.rs`) — compares zic-rs vs
  reference `zic -v` by canonical class → line → wording (last), with explicit verdicts; auto-skips the
  reference half when `zic` is absent.
- **Warning-collection channel** — structured `Severity::Warning` on `CompileReport.diagnostics`,
  non-fatal (`warning ≠ failure`), CORE.1 byte-output unchanged.
- **Verbosity filter** — `compile --verbose`/`-v`; quiet emits `AlwaysOn` only, `-v` emits both
  (matches `zic` vs `zic -v`). The report always *collects* all; only printing is gated.
- **Abbreviation policy** — direct `zic.c::checkabbr` port: single-warning precedence
  (non-POSIX > too-many > fewer-than-3), length over the conforming prefix.
- **`ZIC020` transition-count warning** — emitted (verbose-only, byte-preserving; 0 on canonical).
- **Operational/materialization layer** — `ZIC008` (and the as-yet-uncoded no-clobber / `--mode` /
  alias-map-validation operational failures) are diagnostics but **not** source `-v` diagnostics.
- **Code-stability policy + metadata totality** — append-only · never-reused · class-meaning-stable ·
  severity-change-needs-note · `(class, location, severity)` is the stable surface; every code carries
  `layer`/`span_precision`/`default_severity` (exhaustive) and every `Diagnostic` carries
  `severity`/`verbosity` (type-enforced).
- **Reference-platform diagnostic matrix** — seeded as a test (below).

### Reference-platform diagnostic matrix (seed)

*Upstream IANA is the diagnostic oracle; vendor/platform `zic` is admitted **per platform, never
inferred** — the diagnostic equivalent of the T12 release-admission doctrine.*

| reference_platform | status |
|--------------------|--------|
| `upstream_iana_2026b` | **admitted · active oracle** |
| `local_system_zic` | admitted-if-available (identity captured), else `skipped_with_reason` |
| `aix` | documentation_only (caveat: pre-1901 / 32-bit `time_t`) |
| `solaris_illumos` | documentation_only (caveat: vendor `-s` option history) |
| `bsd` · `macos` · `linux_glibc` | unavailable_on_this_host |

## Intentional divergences (recorded, not hidden)

- **`ContinuationWithoutZone` (ZIC014)** is finer than reference's "input line of unknown type" bucket
  (same location, more precise class) — harness verdict `IntentionalDivergence`.
- **Verbosity divergence:** reference gates "fewer than 3 characters" (and the transition-count
  warning) behind `-v`; zic-rs records this per-diagnostic and the `--verbose` filter reproduces it.
- **Structured diagnostics, not stderr wording:** the stable surface is `(class, location, severity)`;
  zic-rs wording differs deliberately and is only ledgered.
- **Vendor rows are not parity claims** until a platform's `zic` is captured and run; documentation-only
  caveats (AIX/Solaris) are caveats, **not** claims.

## Non-claims

- No **byte-exact `stderr` parity** (wording is ledgered, not matched).
- No **unadmitted vendor/platform parity** (only `upstream_iana_2026b` + a captured local `zic`).
- No **all-platform diagnostic parity**.
- No **full malformed-input / pathology** claim beyond the covered classes (that is T14).
- No **semantic-output proof from diagnostics** (a `diagnostic_artifact` proves the tool noticed
  something, never the compiled output's shape).

## Acceptance

> **T13.close is accepted** when zic-rs records the executable diagnostic contract through ZIC020 —
> class/location-first comparison, warning collection, verbosity filtering, transition-count warnings,
> operational/materialization diagnostics, code-stability policy, span-precision totality,
> reference-platform diagnostic matrix, known divergences, and explicit non-claims — **with no new
> behaviour beyond the sealed T13.6 surface**. *(Met: documentation only; the T13.6 gate — 335 tests,
> CORE.1 341/0/0 — is unchanged.)*

## Handoff → T14 (hostile-input & parser-edge parity)

The next credibility jump is not more diagnostic policy; it is hostile input: line-cap reconciliation
vs pinned `zic.c` (*verify before changing*), text admissibility, metamorphic source-ordering, the
pathology ledger, quote/comment edge cases, name-as-path policy (`ZoneNamePathPolicy`), and
hostile-output-tree (TOCTOU) notes/tests. T15 then owns `zdump` semantic witnesses, oracle-availability,
RFC 9636 structural validation, and the `negative_capabilities`/one-line-status JSON.
