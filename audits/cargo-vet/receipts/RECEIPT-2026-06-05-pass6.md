# RECEIPT — T23.cargo-vet.6 (semver + log first-party review) — 2026-06-05

> **Claim wording (binding):** *T23.cargo-vet.6 reviews the two deferred harder dependencies, `semver` and
> `log`, under a first-party audit standard. It does not claim audited crates are bug-free and does not claim
> full supply-chain verification. Any crate not fully understood remains explicitly exempted.*

## Tooling

`cargo-vet 0.10.2`. Source read from `~/.cargo/registry/src/index.crates.io-…/{semver-1.0.28, log-0.4.30}`.

## Before → after (acceptance #2)

| | before (pass5) | after (pass6) |
|---|---|---|
| fully audited | 32 | **34** |
| partially audited | 1 | 1 |
| exempted | 33 | **31** |
| first-party reviews | 10 | **12** |

`cargo vet` → **Vetting Succeeded (34 fully audited, 1 partially audited, 31 exempted)**
(`raw-2026-06-05-pass6.txt`). Both crates moved **exempted → first-party audited** (`safe-to-deploy`).

## The shared call-context finding (recorded honestly)

Both crates enter the graph **only on WASI targets**, via `getrandom`'s WASI-preview backend:

```text
zic-rs → tempfile → getrandom → wasip3 → wit-bindgen → wit-bindgen-rust-macro
       → wit-bindgen-core → wit-parser → {semver, log}
```

`cargo tree -i semver` and `cargo tree -i log` on the host are **empty** — neither is compiled in zic-rs's
actual (non-WASI) builds. cargo-vet audits the **all-targets** graph, so they are still required; this audit
satisfies that, and the WASI-only reachability is an additional (not load-bearing) safety margin.

## semver 1.0.28 — ADMITTED (acceptance: every unsafe category understood, invariant stated)

- **Features in use:** none beyond default `std` (no `serde`). No build.rs, no proc-macro, no FFI, no
  network/fs/env. Crate-wide **`#![deny(unsafe_op_in_unsafe_fn)]`** — every unsafe op is in an explicit
  block with a SAFETY comment.
- **49 unsafe sites.** `lib.rs` = **0** (the lone grep hit is the `deny(unsafe_op_in_unsafe_fn)` attribute).
- **`identifier.rs` (43)** — the bit-tagged short-string `Identifier` (dtolnay's documented pointer-tag SSO).
  **Invariant:** the 8-byte repr distinguishes **inline** (positive `i64`: 1–8 nonzero ASCII bytes + `\0`
  padding), **heap** (negative `i64`: heap ptrs are align-2 so the freed LSB carries the rotated-out bit, MSB
  set), and **empty** (all-ones `-1`; also the niche making `size_of::<Version>() == size_of::<Option<…>>()`).
  Categories reviewed: pointer↔repr tagging (`ptr_to_repr`/`repr_to_ptr` use the provenance-correct
  `wrapping_add(diff)` idiom; `NonNull` nonzero because the MSB is set); owned heap alloc/dealloc
  (`new_unchecked`/`Clone`/`Drop` use a **matching** `Layout(size, align=2)`; 16/32-bit `isize::MAX` overflow
  guard present); base-128 varint length header (`decode_len`/`bytes_for_varint`, bounded reads); inline reads
  (`inline_len`/`inline_as_str`, aligned `NonZeroU64` read, `from_utf8_unchecked` sound because contents are
  ASCII by precondition); `transmute([u8;8] → Identifier)` (head nonzero since ≥1 ASCII byte).
  **`unsafe impl Send/Sync`:** sound — an **owned** buffer (inline bytes or a uniquely-owned heap allocation),
  no interior mutability, no shared ownership.
- **`parse.rs` (5)** — `Identifier::new_unchecked(string)` ×2 (precondition *ASCII, no `\0`* enforced by the
  `identifier()` lexer, which only accepts `[-.0-9A-Za-z]`); `out.as_mut_ptr().add(depth).write(…)` ×2 +
  `comparators.set_len(len)` (the standard **reserve-then-write-each-index-exactly-once** idiom —
  `reserve_exact` precedes the writes; the recursion writes one comparator per depth and returns the exact
  count passed to `set_len`).
- **The one caller-upheld invariant** (`new_unchecked` requires ASCII/no-`\0`) is verified: the only callers
  are the two ASCII-validated parse paths. **Verdict: admitted, every site sound.**

## log 0.4.30 — ADMITTED (acceptance: global-state model + lifetime + ordering stated)

- **Features in use:** none beyond default `std` (no `kv`/`serde`/`value-bag`). No build.rs, no proc-macro,
  no FFI, no network/fs/env.
- **8 unsafe sites.** **(1) Global logger state machine** — `static LOGGER: &dyn Log` guarded by
  `STATE: AtomicUsize` (`UNINITIALIZED → INITIALIZING → INITIALIZED`). `set_logger` CASes
  `UNINITIALIZED→INITIALIZING` (AcqRel); **only the CAS winner writes `LOGGER`**, then stores `INITIALIZED`
  (Release). `logger()` reads `LOGGER` **only when `STATE == INITIALIZED`** (Acquire) — the Release/Acquire
  pair establishes happens-before, so the read sees the fully-written value; the **`'static`** bound on
  `set_logger` guarantees the logger outlives every read. **Lifetime + ordering invariant: sound.**
  **(2)** `transmute(MAX_LOG_LEVEL_FILTER.load(Relaxed))` `usize → LevelFilter`: the static is private and
  only ever written by casting a `LevelFilter` to `usize`, so every stored value is a valid discriminant.
  **(3)** the `_racy` variants are `pub unsafe fn`s exposing a **documented thread-unsafe contract** for
  no-atomic platforms; they do nothing memory-unsafe and zic-rs never calls them. **(4)** `unsafe impl Sync
  for AtomicUsize` is `#[cfg(not(target_has_atomic = "ptr"))]`-gated to single-core no-atomic targets — not
  compiled on any real zic-rs target. **Concurrency model:** a one-shot set + lock-free reads via the `STATE`
  atomic. **Verdict: admitted, every site sound.**

## Still deferred / exempted (31)

The remaining 31 exemptions (the `clap`/`serde`/`syn`/`rustix`/`libc`/`windows-*` tiers, etc.) stay
**explicitly exempted** — not silently treated as verified. cargo-vet.7+ can take the next tranche. **No crate
was admitted that was not fully understood** (the standing rule).

## Gate

`cargo vet` passes (34/1/31); fmt/clippy/tests/CORE.1 unaffected (config/docs only — no `src/` change);
doc-staleness green. Reproduce: `cargo vet`.
