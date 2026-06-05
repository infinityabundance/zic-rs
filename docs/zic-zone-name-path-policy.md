# `ZoneNamePathPolicy` — zone/link name → output path admissibility (T14.5)

> A zone/link **name** is untrusted input that becomes an **output path** under `--out`
> (`Link ../../etc/passwd evil` is a thing an attacker writes). This is its own boundary — *name-as-path*
> — distinct from input-text admissibility (T14.1–T14.2) and the time-law pathologies (T14.4). Pinned
> from reference `zic`'s `namecheck`/`componentcheck` (tzcode 2026b) and made executable in
> `tests/zone_name_path_policy.rs`. CORE.1 341/0/0 unchanged (real names are all benign).

## Five axes — kept distinct (the senior point: these are not one axis)

1. **Logical-name validity** — is the *name* a well-formed `zic` name at all (non-empty, no traversal)?
2. **Output-path materialization safety** — can it be written under `--out` without escaping the root?
3. **Platform-specific path constraints** — separators, reserved names, case-folding (host-dependent).
4. **Reference `zic` diagnostics** — what reference rejects (fatal) vs `-v`-warns (portable).
5. **zic-rs safer divergence** — where zic-rs is deliberately stricter than reference.

## The policy table

| Case | Reference `zic` (`namecheck`/`componentcheck`, pinned) | zic-rs | Axis | Code | Bucket | Status |
|------|--------------------------------------------------------|--------|------|------|--------|--------|
| **Empty name** | fatal "empty file name" | reject | 1·2 | `ZIC008` | 1 | ✅ matched (fatal) |
| **Absolute** (leading `/`) | fatal "begins with '/'" | reject | 2 | `ZIC008` | 1 | ✅ matched |
| **`//`** (empty interior component) | fatal "contains '//'" | reject | 1 | `ZIC008` | 1 | ✅ matched |
| **Trailing `/`** | fatal "ends with '/'" | reject | 1 | `ZIC008` | 1 | ✅ matched |
| **`.` component** | fatal "contains '.' component" | reject | 2 | `ZIC008` | 1 | ✅ matched |
| **`..` component** (traversal) | fatal "contains '..' component" | reject | 2 | `ZIC008` | 1 | ✅ matched |
| **NUL byte** | (caught earlier as "NUL input byte") | reject | 1·2 | `ZIC008`/`ZIC016` | 1 | ✅ matched |
| **Leading-`-` component** | **`-v` warning** "component contains leading '-'" (accepts) | **reject** (fail-closed) | 5 | `ZIC008` | **3** | intentional safer divergence (looks like a CLI flag / tooling foot-gun) |
| **Non-benign byte** (digit · `+` · `.` · space · punctuation · control/high) | **`-v` warning** "contains byte '%c'" / "\\%o" (accepts) | **`ZIC024`** verbose-only warning (accepts) | 4 | `ZIC024` | 1 | ✅ matched (T14.5) — was silent before |
| **Overlength component** (> 14 bytes) | **`-v` warning** "overlength component" (accepts) | **`ZIC025`** verbose-only warning (accepts) | 4 | `ZIC025` | 1 | ✅ matched (T14.5) — was silent before |
| **Non-UTF-8 / high bytes** | byte-oriented; `-v`-warns (octal form), accepts | **rejected at the lexer** (source must be UTF-8 → `ZIC012`) | 3·5 | `ZIC012` | **3** | safer divergence: zic-rs requires UTF-8 source; reference is byte-oriented. Ledgered, not "matched" |
| **Backslash / platform separator** | treated as an ordinary non-benign byte (`-v` warn); `/` is the only separator | same — `\` is a non-benign byte → `ZIC024`; never a separator | 3·4 | `ZIC024` | 1 | ✅ matched (no Windows-separator special-casing — `/` only) |
| **Reserved device names** (`CON`, `NUL`, `AUX`, …) | no special handling (POSIX tool) | no special handling | 3 | — | 4 | **ledgered** (a Windows-materialization concern; deferred — out of scope until a Windows install path is claimed) |
| **Case-insensitive collisions** (`America/New_York` vs `america/new_york`) | no special handling; relies on the filesystem | no special handling; relies on the filesystem | 3 | — | 4 | **ledgered** (case-folding is filesystem-dependent; not enforced — recorded, not inferred) |

## What T14.5 changed (narrow, reference-pinned)

The hard *structural* policy (`safe_relative_path` → `ZIC008`) was already complete and matched
reference's fatal cases (plus the leading-`-`/NUL safer divergences). T14.5 added the **portability
warning axis** the T14.4 ledger surfaced — two append-only, verbose-only codes through the diagnostic
totality path (`layer()` = Warning · `span_precision()` = Line · `default_severity()` = Warning;
ZIC001–**ZIC025**):

* **`ZIC024_ZONE_NAME_NONPORTABLE_BYTE`** — a name byte outside the benign set (ASCII letters, `-`, `_`;
  `/` separator). Mirrors `zic -v`'s "contains byte". Fires on real names like `Etc/GMT+5` (the `+`),
  exactly as `zic -v` does — verified against reference.
* **`ZIC025_ZONE_NAME_OVERLENGTH_COMPONENT`** — a component > 14 bytes. Mirrors `zic -v`'s "overlength
  component".

Collected by `collect_zone_name_warnings` in `plan::run` over every compiled zone name **and** every
materialized link name (reference runs `namecheck` on both `ZF_NAME` and `LF_LINKNAME`); located at the
defining source line; **verbose-only** (zic-rs has no quiet mode, so the report always collects them but
the CLI prints them only under `--verbose` — matching `zic` vs `zic -v`). One warning of each kind per
name (first offender), bounded to avoid per-byte spam. **Read-only** over the name: never changes output
bytes or exit status → **CORE.1 341/0/0 unchanged** (canonical names are all benign except the `Etc/GMT±N`
family, which now warn under `--verbose` exactly as reference does).

## Byte-level caution (honest scoping)

Reference `zic` is C and **byte-oriented**: it warns on non-benign/high bytes but does not require UTF-8
and does not special-case platform separators or reserved names. zic-rs's policy is **typed and safer**
(UTF-8-required at the lexer, leading-`-` rejected), and reference's byte-level name warnings are
**pinned** (`ZIC024`/`ZIC025`). Platform-specific path behaviour (Windows reserved names, case-folding,
`\` semantics) is **ledgered, not inferred** — it is deferred until a concrete Windows/non-Unix install
path is claimed (cross-link `docs/platform-portability.md`), never assumed from Linux/UTF-8 defaults.

## Acceptance (T14.5)

> Met: the `ZoneNamePathPolicy` is recorded (this doc) and enforced — fatal structural rules
> (`ZIC008`) and the two new verbose-only portability warnings (`ZIC024`/`ZIC025`) — with absolute /
> traversal / empty / `//` / trailing-`/` / leading-`-` / NUL classified, non-benign-byte and overlength
> warnings added (matching reference `zic -v`), non-UTF-8 and case-collision and reserved-name cases
> ledgered with their bucket, and CORE.1 341/0/0 preserved. Executable witness:
> `tests/zone_name_path_policy.rs`.

## Non-claims (T14.5)

* No **Windows/non-Unix** name-materialization parity (reserved names, `\`, case-folding) — ledgered,
  deferred, not claimed.
* No claim to reproduce reference's **per-byte** warning multiplicity — zic-rs emits one `ZIC024`/`ZIC025`
  per name (first offender), bounded; class/location parity, not count parity.
* zic-rs is **stricter** than reference on leading-`-` and non-UTF-8 (bucket 3) — that is a deliberate
  safer divergence, not parity.
