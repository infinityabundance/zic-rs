# Operational (CLI / filesystem / install-mode) parity with reference `zic`

> **Runnable companion:** this doc is the *inventory + rationale*; the **receipt-backed, executed** version is
> **[`docs/zic-operational-parity-matrix.md`](zic-operational-parity-matrix.md)** (ZIC-MATRIX.1) — every flag
> run on both compilers with a strict verdict (match / class-parity / intentional-divergence /
> unsupported-by-design / deferred / divergent).

> **Campaign T9 — inventory first (T9.1).** This is the *read-only* contract inventory: every
> reference-`zic` flag mapped to its zic-rs equivalent and classified, with the safety rationale for
> each deliberate divergence. It changes **no behaviour** — it is the map that keeps T9.2–T9.5 from
> becoming a swamp. Doctrine: **CLI parity ≠ parser parity; filesystem parity ≠ behaviour parity;
> install-mode parity must never weaken output safety.**

## The shape difference (deliberate, documented)

Reference `zic` is a **single-shot** compiler — one invocation, short flags, compiles every zone in
the input file(s), and (by default) **installs into the system zoneinfo tree**. zic-rs is a
**subcommand** tool — `compile` / `compare` / `explain` / `supported-syntax` / `support-report` /
`structural-report` — with long flags, an explicit **required `--out`** (there is *no* implicit
`/usr/share/zoneinfo` default), and a producer/verifier surface beyond what `zic` offers.

This is an **intentional divergence**, not an oversight: the required `--out` and the no-system-install
default are a safety posture (a packager opts in to a staging dir; nothing is ever written to system
paths implicitly). Operational *parity* therefore means "every `zic` capability is reachable, with
equal-or-safer defaults," not "byte-identical argv."

## Reference-`zic` flag inventory (tzcode 2026b)

Status legend: **✅ implemented** · **◐ partial** · **▷ deferred** (with the milestone that lands it) ·
**⚠ intentionally different** (deliberate divergence, documented).

| `zic` flag | Purpose | zic-rs equivalent | Status |
|------------|---------|-------------------|--------|
| `--version` | print version | `--version` (clap) | ✅ |
| `--help` | usage | `--help` (clap), `supported-syntax` | ✅ |
| `-v` | verbose / portability warnings | warnings to stderr; no `-v` taxonomy yet | ▷ deferred → **T13** (warning parity) |
| `-b {slim\|fat}` | emission bloat | `compile -b {slim\|fat}` / `--bloat` (alias onto `--emit-style`) | ✅ (T8-slim + **T10.2**) ⚠ default is fat-style, behaviour-matched; `-b slim` == `--emit-style zic-slim` (byte-identical); conflicting `-b`+`--emit-style` → error |
| `-d directory` | output dir | `compile --out <dir>` (**required**) | ✅ ⚠ no implicit system default (safety) |
| `-D` | do **not** create missing parent dirs | `compile --no-create-dirs` / `-D` | ✅ (T9.3) |
| `-l localtime` | make `localtime` a link to a zone | `compile --localtime <zone>` / `-l` | ✅ (T9.4) ⚠ link written **only under `--out`** (target must also be selected) |
| `-L leapseconds` | compile with a leap-seconds file | `compile --leapseconds <file>` / `-L` | ✅ (T11) ⚠ opt-in `right/` build profile; leap grammar is leap-source-only, never the default |
| `-m mode` | file mode for created files | `compile --mode <octal>` / `-m` | ✅ (T9.5) ⚠ **octal subset, Unix-only** (symbolic chmod exprs not parsed; applies to TZif + copied links, not symlinks) |
| `-p posixrules` | create the legacy `posixrules` link | — | ⚠ intentionally out-of-scope (legacy — reference `zic` itself warns *"-p is obsolete and likely ineffective"*; never a zone failure) → see **T9.4** below |
| `-r '[@lo][/@hi]'` | truncate to a timestamp range (`-00` placeholder) | `compile --range '[@lo][/@hi]'` / `-r` | ✅ (T10.4) ⚠ all three declared profiles 341/341; claim scoped to declared profiles (see range-emission doc) |
| `-R @hi` | redundant trailing transitions to `@hi` | `compile --redundant-until @<sec>` / `-R` | ✅ (T10.3) ⚠ slim-only, independent of `-b`, behaviour-preserving |
| `-t localtime-link` | alternate `localtime` path/name | `compile --localtime-name <name>` / `-t` | ✅ (T9.4) ⚠ **safe relative name under `--out` only** (not an arbitrary/system path) |
| `-u 'owner[:group]'` | owner/group for created files | — | ⚠ deferred (privileged, Unix-only install metadata — documented roadmap; see **T9.5** below) |
| `[ filename … ]` | input source files | `--input <path>…` (files **or** dirs) | ✅ ⚠ also accepts directories |

## zic-rs-specific surface (producer / verifier — beyond `zic`)

These have no `zic` counterpart; they are the producer/verifier role (Rust-ecosystem + reviewer
personas, T16):

- **Zone selection** — `compile --zone <z>` / `--zones <file>` / `--all-supported` (`zic` always
  compiles every zone in the input). Enables minimal bundles (T21).
- **`--force`** — opt-in clobber; the default is **atomic no-clobber** (a safety addition over `zic`,
  which overwrites).
- **`--unsupported {error|skip}`** — fail-closed (default) vs warn-and-skip.
- **`--link-mode {copy|symlink}`**, **`--transition-limit`**, **`--alias-map`**, **`--manifest`**,
  **`--emit-style`**.
- **Verifier subcommands** — `compare` (zdump/structural oracle), `support-report`, `structural-report`
  (the conformance engine; `zic` has no equivalent).

## Safety boundary (the load-bearing rule)

Hard safety rejects are **never** relaxed for `zic` compatibility, and are kept separate from soft
portability behaviour:

- **Path traversal / unsafe names** → `ZIC008`, always rejected (absolute, `.`/`..`, traversal,
  leading `-`, NUL) — *hard reject*, regardless of what `zic -v` would merely warn about.
- **No implicit system install** — output only under explicit `--out`.
- **Atomic, no-clobber-by-default** writes; cleanup-on-error.

Where reference `zic` permits a filesystem-dangerous action in a historical/operational context,
zic-rs chooses the safer default and records here whether that is a **deliberate divergence** (e.g.
required `--out`, no-clobber default) or a future **mode-gated** compatibility option.

## Exit-status contract (T9.2 ✅)

The success/failure **classification** is a stable CLI contract (diagnostic *wording* may evolve; the
*codes* must not surprise scripts). Kept deliberately small:

| Code | Meaning | Examples |
|------|---------|----------|
| `0` | success | `--help`, `--version`, `supported-syntax`, a valid `compile` |
| `1` | operational / compiler / config failure | parse error · unsupported construct (`ZIC001`) · output-path traversal (`ZIC008`) · output exists without `--force` · missing/unreadable `--input` file · **missing `--out`** (no implicit default) · reference `zic` absent for `compare`/`structural-report` |
| `2` | CLI usage error (clap) | unknown option · missing a structurally-required arg (**`--input`**) |

Note the deliberate asymmetry: **`--input`** is clap-required (its absence is a usage error → `2`),
while **`--out`** is validated in the command (no implicit system default → a config error → `1`, with
a domain message). Both are non-zero; the split reflects that `--out`'s absence is a *safety* decision,
not a syntax slip. `--help`/`--version` exit `0` even though clap models them as "errors." Pinned by
**`tests/cli_operational_parity.rs`** (12 cases across the matrix).

## Filesystem materialization (T9.3 ✅)

- **No partial install after a fatal — `intentional safer divergence` (bucket 3).** `compile` now runs
  in two phases: (1) compile **every** selected zone to memory (serialise its bytes, pre-validate its
  output path); (2) materialise only if *all* succeeded. A fatal in phase 1 (parse error, unsupported
  construct under default `error`, oversized output, a `ZIC008` traversal name) aborts **before any
  file is written** — even an earlier good zone is not left behind. Reference `zic` writes as it goes
  and can leave a half-built tree; zic-rs deliberately does not. Pinned by
  `tests/cli_operational_parity.rs::{unsupported_later_zone,traversal_zone}_leaves_no_partial_install`.
- **`-D` / `--no-create-dirs`** (`implemented parity`): with it, the `--out` directory must already
  exist (a missing one is a config error → exit `1`, and nothing is created); without it, zic-rs
  creates the explicit `--out` root (never a system default). Tests `no_create_dirs_{missing,existing}_dir_*`.
- **Atomic, no-clobber-by-default + cleanup-on-error** (already shipped): each file is written via a
  temp + atomic publish; re-compiling without `--force` fails (exit `1`) rather than clobbering. The
  hard `ZIC008` boundary is unchanged.

> Honest scope: this is **per-run all-or-nothing on a *fatal*** (phase-1 abort). A full **tree-level
> transaction** across phase-2 write-time I/O errors (write-to-staging-then-atomic-swap) is a future
> hardening item (T17); documented, not silently claimed.

## Localtime / install policy (T9.4 ✅)

Reference `zic`'s `-l <zone>` makes `localtime` a link to `<zone>` (the link's name is `-t`'s value,
default `localtime`); `dolink(lcltime, tzdefault)`. zic-rs reproduces the *capability* as an **opt-in
install policy that stays inside the safety boundary**:

- **`--localtime <zone>` (`-l`)** — `implemented parity` (within the producer model): after a
  successful compile, write a `localtime` entry in `--out` pointing at `<zone>`. The target must also
  be among the selected zones (so the link never dangles); if it is not, the run is a **config error
  (exit 1) before any file is written** — same no-partial-install guarantee as T9.3. The link is
  materialised in phase 2, after its target zone is on disk, with the usual atomic / no-clobber /
  `--link-mode {copy|symlink}` semantics.
- **`--localtime-name <name>` (`-t`)** — `intentional safer divergence` (bucket 3). Chooses the link
  name (default `localtime`), but **constrained to a safe relative name under `--out`**: an
  absolute/traversal/unsafe name (e.g. `/etc/localtime`, `../escape`) is a hard `ZIC008`-class reject
  (exit 1, nothing written). Reference `zic` will write the localtime link to an *arbitrary path*,
  including a system path; zic-rs deliberately will not — output stays under the explicit `--out`.
- **`-p posixrules`** — `intentionally out-of-scope (legacy)`. Reference `zic` 2026b *itself* warns
  `"-p is obsolete and likely ineffective"` for any non-`-` argument; the `posixrules` mechanism is a
  deprecated runtime-policy surface, not canonical-zone conformance. zic-rs does **not** implement it
  and **never** counts its absence as a zone failure. (If a concrete consumer ever needs it, it would
  land as an explicit opt-in compatibility mode — bucket 2 — never a default.)

Pinned by `tests/cli_operational_parity.rs::localtime_{creates_link,custom_name,unselected_target_fails_no_partial_install,unsafe_name_rejected_no_partial_install}`.
Behaviour sweep unchanged (**341/0/0** — this is install policy, never the compile engine).

## File mode / ownership (T9.5 ✅)

Reference `zic` sets created-file metadata: `-m mode` → `fchmod(output_mode)` (chmod-style mode,
octal *or* symbolic); `-u owner[:group]` → `getpwnam`/`getgrnam` + `fchown` (privileged, POSIX-only —
`getpwnam` is a no-op stub off-POSIX). zic-rs treats these as **install metadata, never compiler
surfaces** — they cannot change a single compiled byte (CORE.1 is untouched):

- **`--mode <octal>` (`-m`)** — `implemented parity (octal subset)`. Sets the permission bits of the
  created regular files (compiled TZif zones **and copied** link files). The value is parsed as
  **octal** (`644`, `0644`, `0o600`) and **validated before any write** — a non-octal value or one
  above `0o7777` is a config error (exit 1, nothing written). **Unix-only**: a `--mode` on a non-Unix
  platform is a config error caught in phase 1 (it never silently no-ops). **Not** applied to symlink
  entries — `chmod` follows the link, so it would alter the *target*'s permissions; verified that a
  `--link-mode symlink` entry is left untouched. `intentional simplification`: only the octal subset
  of `zic`'s `-m` is accepted; symbolic chmod expressions (`u+rwx`) are not parsed.
- **`-u owner[:group]`** — `deferred (privileged, Unix-only)`. Changing owner/group requires
  privilege and is platform-specific; it is **not** part of the default safe-compile path. Documented
  as roadmap and **never silently ignored** (there is no `--owner` flag yet, so a request can't be
  mistaken for a no-op). If a concrete consumer needs it, it lands as an explicit Unix-only mode.

Pinned by `tests/cli_operational_parity.rs::{mode_applied_to_compiled_file,mode_applied_to_copied_link,bad_octal_mode_fails_no_partial_install}` (the first two `#[cfg(unix)]`) and `cli::tests::{parses_octal_with_and_without_leading_zero,rejects_non_octal_and_out_of_range}`. Behaviour sweep unchanged (**341/0/0**).

## T9 — complete

`-l`/`-t`/`-D`/`-m` are implemented within the safety boundary; the exit-status taxonomy and the
no-partial-install guarantee are pinned by tests; `-u` (privileged) and `-p` (legacy `posixrules`) are
documented deferrals, never silently ignored, never counted as zone failures. The hard safety boundary
(no implicit system install · `ZIC008` · atomic no-clobber · cleanup-on-error · `--mode` Unix-gated)
was preserved throughout. Remaining `zic` operational flags are owned by later milestones: `-v` → T13,
`-L` → T11 (done), `-r`/`-R` → T10 (done).
