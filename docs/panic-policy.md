# Panic policy (T17.1)

> **The rule in one line:** *no input — however malformed, hostile, or pathological — may panic the
> library; invalid input becomes a typed `Err`, never an abort.* A panic is reserved for a violated
> **internal** invariant (a programmer error), never for a value that crossed a trust boundary.

This is a reliability statement, not a security claim: it says what the code *guarantees by
construction*, and it pairs with the broader [`security.md`](security.md) (memory safety · input
hardening · output-tree safety · resource bounds) — that doc covers the *what*, this one the
*panic-discipline contract* and how it is enforced.

## The two categories

Every fallible site in `src/` belongs to exactly one of these. The distinction is the policy:

1. **Untrusted-reachable → must `Result`.** Anything reachable from a value that crossed a trust
   boundary — `.zi` source text, a leap-seconds file, a TZif byte stream handed to `tzif::validate::parse`
   / `compare`, a zone/link *name* used as an output path, a CLI argument — **must** fail closed with a
   `crate::error::Error`, never `panic!`/`unwrap`/`expect`/`unreachable!`/array-index/overflowing
   arithmetic. Malformed input is a *diagnosable condition*, not a bug.

2. **Internal-invariant → may assert.** A condition that *cannot* hold unless the program itself is
   wrong (a value the code just constructed, an exhaustive-match arm that the type system can't prove
   unreachable, a post-condition of a prior validated step) **may** use `expect("<why it holds>")` /
   `unreachable!("<why>")`. The message must state the invariant, so a failure reads as the programmer
   error it is. These are assertions, not error handling, and must never sit on an untrusted path.

## What is enforced today

- **`#![forbid(unsafe_code)]`** (crate + binary) and **`overflow-checks = true`** in *all* profiles
  (release included): integer overflow on an untrusted count traps as a panic rather than wrapping to a
  wrong-but-silent value. Trapping is acceptable as a last-resort backstop (a controlled abort, not
  memory corruption); the *intent* is still that untrusted arithmetic is checked/bounded before it can
  reach that backstop (see [`security.md`](security.md) "Resource bounds" and T17.1b caps).
- **No `panic!` / `todo!` / `unimplemented!`** in non-test `src/`.
- The parser/lexer/model/compile/tzif-writer/output paths return `Result` on every malformed-input
  condition (the T13/T14 diagnostic contract `ZIC001`–`ZIC026` is the typed-rejection surface).
- **TZif *reading* is bounds-guarded at the choke point.** `tzif::validate::parse` is the single entry
  for decoding a TZif byte stream (own output *and* reference `zic` output *and*, via `tzif-validate`,
  arbitrary `--input`). It validates structurally as it reads: counted-array bounds via a
  bounds-checked `Cursor`, designation indices via `read_cstr`, and **(T17.1) every transition's
  `type_index` against `typecnt`** — so no downstream consumer (`compare::semantic::diff`,
  `compile::leap`) can index `types[type_index]` out of bounds. The deeper RFC 9636 structural
  validator (`tzif/rfc9636.rs`, T15.4) layers further invariant checks and is likewise bounds-safe
  (`Err`, never panic).

## Allowed internal-assertion sites (the audited exceptions)

These are category-2 sites, each guarded by a prior check or by construction — kept as assertions
*on purpose*, with the invariant named:

- `tzif/data_block.rs` — the abbreviation packer's two-pass lookups (`expect("…present after the build
  pass")`, `expect("…NUL-terminated")`): pass-1 guarantees what pass-2 reads.
- `tzif/data_block.rs` — `i32::try_from(at).expect("32-bit transition time out of range")` and the
  `unreachable!("unsupported TZif time size {other}")` in the time-size match: the **writer** controls
  `time_size ∈ {4, 8}`, and per the T1 policy the v1 block carries **no** transitions, so the 32-bit
  path is never fed an out-of-range value. *If that policy ever changes, this becomes a real check.*
- `compile/transitions.rs` — `.expect("non-empty")` / `.expect("finite set has a concrete TO")`: each
  guarded by an `is_empty()` / `has_recurring` test immediately above.
- `source/parser.rs` — `.expect("continuation without an open zone is impossible by construction")`:
  the lexer's record-ordering invariant; a *detached* continuation in real input is caught earlier as
  the `ContinuationWithoutZone` diagnostic, not here.

## How the policy stays true

- **Tests are the gate.** Hostile-input coverage (`tests/input_admissibility.rs`,
  `tests/pathology_ledger.rs`, `tests/hostile_output_tree.rs`, `tests/fuzz_regressions.rs`) plus the
  T17.1 parse bounds-guard regression
  (`tzif::validate::tests::out_of_range_transition_type_index_is_rejected_not_panic`) assert
  *typed rejection, not panic*, on the shapes that matter.
- **Fuzz-exercised (bounded).** `T23.cargo-fuzz.1` (libFuzzer, 25 s/target) found **3 implicit panics
  the static census could not see** — a footer slice-index (`tzif/validate.rs`), an `h*3600`
  multiply-overflow (`model/time.rs`), and a non-char-boundary `str` slice (`source/parser.rs`).
  `T23.cargo-fuzz.2` fixed all three (`strip_prefix`/`strip_suffix`, `checked_mul`/`checked_add`,
  byte-comparison), each with its minimized seed as a regression test; the bounded smoke then re-ran
  **9/9 clean** and the 4 seeds replay `rc=0`. This **restores the no-panic claim for the known F1–F3
  seeds and that bounded rerun only — it is not an exhaustive no-panic proof** (a longer campaign could
  find more; it would be admitted by a new receipt). See
  `audits/cargo-fuzz/receipts/RECEIPT-2026-06-04-fuzz2.md`.
- **A new `unwrap`/`expect`/`unreachable!` on an untrusted path is a policy violation**, reviewable as
  such. (A CI grep gate over non-test `src/` is the intended mechanical backstop — see `roadmap.md`
  T17 "verify-here"; it is not yet wired in this no-network sandbox, recorded honestly.)

## Honest non-claims

- This is **not** a claim of total DoS resistance. T17.1b added generous `limits::ResourceLimits` caps
  over the input-driven dimensions (source bytes · zone/rule/link/leap counts · link-chain &
  continuation depth), so the unbounded-growth tail is now bounded; but the caps sit far above any real
  tzdb (they stop the pathological tail, they are not a tight quota), and `overflow-checks` traps rather
  than corrupts — a trap is still a controlled abort. The guarantee is **no memory unsafety and no panic
  on malformed input on the audited paths**, widening as T17 proceeds.
- This is **not** a security sandbox for executing hostile binaries (the `tzif-validate` reader is
  bounds-safe but is a validator, not a sandbox — see T15.4).
