# cargo-vet receipt — 2026-06-05 — T23.cargo-vet.5 (unsafe-heavy dependency review tier)

> **Receipt wording (binding):** *T23.cargo-vet.5 reduces the unaudited dependency surface by reviewing a
> bounded unsafe-heavy tier. It does not claim full supply-chain verification, does not claim audited crates
> are bug-free, and leaves any crate not fully understood explicitly exempted.*

## Before → after (exact counts)

| | `cargo vet` summary | first-party audits | exempted / UNAUDITED |
|---|---|---|---|
| **before** (T23.cargo-vet.4) | 29 fully + 1 partial | 7 | **36** |
| **after** (T23.cargo-vet.5) | **32 fully + 1 partial** | **10** | **33** |

`cargo vet` (0.10.2): **`Vetting Succeeded (32 fully audited, 1 partially audited, 33 exempted)`**. Net:
**+3 first-party audited, 36 → 33 exempted.** Counts stay honest (fully + partial + exempted = the graph;
exempted is never called "audited").

## The tier reviewed (deferred unsafe-heavy set from T23.cargo-vet.4)

Candidate set: `itoa`, `anstyle-parse`, `anstyle-query`, `semver`, `log`. **Admitted 3, deferred 2** — per
the rule *any crate not fully understood in one sitting stays exempted.* Each admitted crate's every
`unsafe` site was **read and reasoned about**, not merely counted.

### Admitted (each `unsafe` site reviewed → `safe-to-deploy`)

| crate | ver | features in use | unsafe sites | unsafe purpose | build.rs / proc-macro / FFI | net / fs / env | platform-sensitive? |
|---|---|---|---|---|---|---|---|
| **`itoa`** | 1.0.18 | none (`no-panic` off) | **13** (one pattern) | write ASCII digit-pairs from a 200-byte lookup table into a `MaybeUninit` stack buffer sized for the int type, then read the initialized ASCII tail | none / none / none | none | no (`no_std`) |
| **`anstyle-parse`** | 1.0.0 | `default=["utf8"]` (→ `utf8parse`, `arrayvec`, separately vetted) | **3** | (1) `[MaybeUninit;16]` OSC-slice array init; (2)(3) `transmute(u8→State/Action)` | none / none / none | none | no |
| **`anstyle-query`** | 1.1.5 | default | **1** | `cfg(windows)` Win32 `GetConsoleMode`/`SetConsoleMode` (enable VT) | none / none / **FFI (Windows-only)** | reads colour env vars only (read-only) | **YES — Windows path mutates console mode** |

**Per-crate soundness reasoning (the "understood", not just "counted"):**

- **`itoa` (13 sites, all one pattern).** The encoder writes `buf[i].write(*DECIMAL_PAIRS.0.get_unchecked(pair*2+{0,1}))`
  where `pair = n%100 ∈ 0..99` ⇒ index `0..199`, always in-bounds of the 200-byte ASCII table. Writes land in a
  `[MaybeUninit<u8>; MAX_STR_LEN]` buffer sized to the type. `slice_buffer_to_str` does `get_unchecked(offset..)`
  with `offset = len − digits ≥ 0`, then `from_utf8_unchecked` — sound because the tail is fully initialized with
  ASCII from the table. `format()` casts the i128-sized buffer to the smaller per-type buffer (fits);
  `unreachable_unchecked()` only asserts the encoder's own length invariant. **Call context: input is always a
  primitive integer, never hostile bytes.** Sound.
- **`anstyle-parse` (3 sites).** (1) `[MaybeUninit<&[u8]>; MAX_OSC_PARAMS] = MaybeUninit::uninit().assume_init()`
  is the always-sound array-of-MaybeUninit idiom; only the first `osc_num_params` entries are initialized and only
  `slices[..num_params]` is read back, with `osc_num_params` **capped at `MAX_OSC_PARAMS=16`** (parser returns at
  the cap) ⇒ read region = initialized region. (2)+(3) `transmute::<u8,State>(delta&0x0f)` / `(delta>>4)` —
  **I counted the enums: State and Action each have exactly 16 `#[repr(u8)]` tag-only variants (0..15)**, the
  operands are masked into `0..15`, and the `delta` bytes come from a const transition table that only packs valid
  pairs ⇒ every transmute hits a valid discriminant. Sound (relies on the documented 16-variant invariant, which I
  verified holds in this version).
- **`anstyle-query` (1 site).** Entirely `#[cfg(windows)]`; calls `GetConsoleMode`/`SetConsoleMode` (null-handle
  check + `last_os_error`) to enable ANSI VT. **Platform-sensitive note recorded:** `SetConsoleMode` *mutates* the
  Windows console mode — a standard, documented VT-enable, but a real side effect — and it is **never compiled on
  non-Windows targets** (zic-rs builds on Unix). The portable path only **reads** `CLICOLOR`/`CLICOLOR_FORCE`/
  `NO_COLOR`/`TERM` (read-only). Admitted with that explicit note.

### Deferred (left exempted — not fully understood in one sitting; honest)

- **`semver` 1.0.28** — **~47 unsafe sites** (NonNull / manual tagged-pointer `Identifier` interning + ptr
  arithmetic). Genuinely subtle pointer-provenance reasoning across ~2117 LoC; **not** a one-sitting review.
  Remains exempted → a future dedicated pass.
- **`log` 0.4.30** — ~5673 LoC with **global mutable logger state** (atomics + a `&'static dyn Log` lifetime
  transmute, 6 unsafe). The soundness argument is whole-crate global-state lifetime reasoning; **too large for one
  coherent sitting.** Remains exempted.

## What this is NOT (non-claims)

- **Not full supply-chain verification** — 33 crates remain exempted/UNAUDITED.
- **Not "audited crates are bug-free"** — `safe-to-deploy` attests "no malicious/unsound behaviour on full read +
  every `unsafe` site reasoned sound," not functional correctness.
- **Not "all transitive deps trusted"** — trusted-import coverage is separate delegated trust.
- `anstyle-query`'s Windows console mutation is real (recorded), just `cfg`-excluded from our builds.

## Gate

Supply-chain metadata only — **no `src/` change**, so the parent gate (fmt · clippy `-D warnings` · 503 tests ·
CORE.1 341/0/0) is unaffected and unchanged. `cargo vet` Vetting Succeeded. `doc-staleness-check` green.
**Next owner:** T23.cargo-vet.6 — `semver` (the 47 NonNull-interning sites) and `log` (global-logger lifetime),
each with real dedicated review time.
