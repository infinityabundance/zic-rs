# RECEIPT — T23.cargo-vet.8 (large-tranche review; small fully-understood admits) — 2026-06-05

> A bounded review of the remaining dependency trust tail (large syscall, proc-macro, serde/clap, and
> platform crates). **It does not claim full supply-chain verification.** Same standard as every pass: each
> admitted crate read in full, every `unsafe` site reasoned; any crate not fully understood **stays
> explicitly exempted**. Does not claim audited crates are bug-free.

## Before → after

| | pass7 | pass8 |
|---|--:|--:|
| fully audited | 39 | **42** |
| partially audited | 1 | 1 |
| exempted | 26 | **23** |
| first-party reviews | 17 | **20** |

`cargo vet` → **Vetting Succeeded (42 fully audited, 1 partially audited, 23 exempted)** (`raw-…-pass8.txt`).

## Reachability map (the campaign's main deliverable — host-used vs all-target-only)

`cargo tree -i <crate>` on the host build, per remaining/affected exemption. This separates what actually
compiles in zic-rs's real builds from what only exists on other targets / in the all-target vet graph:

| crate | host-reachable? | tier |
|---|---|---|
| `syn`, `clap`, `clap_builder`, `clap_derive`, `rustix`, `libc`, `linux-raw-sys`, `getrandom`, `once_cell`, `tempfile` | **yes (host)** | large proc-macro / syscall / CLI — exempted (see below) |
| `errno` | no (reached via rustix/tempfile on some targets) | **ADMITTED this pass** |
| `anstream` | yes (host; clap colour stack) | **ADMITTED this pass** |
| `anstyle-wincon` | no (Windows-only) | **ADMITTED this pass** |
| `serde`, `serde_core`, `serde_derive`, `serde_json` | **no — not in host graph** (all-target/dev-graph only) | exempted |
| `windows-sys`, `r-efi`, `errno`, `hashbrown`, `indexmap`, `memchr`, `anyhow`, `prettyplease`, `zmij`, `anstyle-wincon` | no (platform / all-target-only) | exempted |

**Honest note on the serde/clap buckets the campaign named:** `serde*`/`serde_json` are **not in the host
dependency graph at all** (they enter only via the all-target / dev-graph), and `clap_builder`/`syn` are
host-reachable but are **large** (clap_builder thousands of LoC; syn 153 unsafe sites) — neither is
"fully understood" in one tranche, so both **stay exempted**. The rule held: admit only what is fully
understood.

## The 3 admitted (small, fully-reviewed, every unsafe site reasoned)

- **errno 0.3.14** (~472 LoC, 12 unsafe) — cross-platform `errno` accessor. Every unsafe site is a standard
  bounded pattern: `from_utf8_unchecked(&input[..valid_up_to()])` (prefix proven valid UTF-8); `strerror_r`
  into a fixed 1024-byte **stack** buffer + `strlen`-bounded slice (ERANGE / glibc<2.13 negative-rc handled);
  thread-local errno-pointer deref (`__errno_location`/`GetLastError`/etc). No build.rs, no proc-macro, no
  net/fs — the only effect is reading/writing the thread-local C errno + formatting its message.
- **anstream 1.0.0** (~2562 LoC, **only 3 unsafe**) — the clap auto-stripping stdout/stderr stream. All 3
  unsafe sites are one pattern: a private `from_utf8_unchecked` that does the **checked** `from_utf8().expect()`
  under `cfg!(debug_assertions)` and only goes unchecked in release; its sole caller passes a `printable`
  slice split from already-UTF-8-validated input at a non-(printable|UTF-8-continuation) boundary → a
  whole-codepoint prefix. No build.rs/proc-macro/net.
- **anstyle-wincon 3.0.11** (~473 LoC, 2 unsafe) — Windows-console colour. Both unsafe sites are
  `#[cfg(windows)]` Win32 FFI (`GetConsoleScreenBufferInfo`/`SetConsoleTextAttribute`): null-check handle,
  `zeroed()` on a plain-integer C struct, return-code → `last_os_error()`. `SetConsoleTextAttribute` mutates
  the console (a documented, standard colour side effect); **never compiled on zic-rs's non-Windows targets**.

## Still exempted (23) — honestly deferred, not faked

The genuinely large / syscall-heavy / generated / not-host-reachable tiers stay **explicitly exempted**:
`windows-sys` · `linux-raw-sys` · `rustix` · `libc` · `memchr` · `r-efi` · `hashbrown` · `syn` ·
`getrandom` · `anyhow` · `zmij` · `once_cell` · `prettyplease` · `indexmap` · `serde`/`serde_core`/
`serde_derive`/`serde_json` · `clap`/`clap_builder`/`clap_derive` · `tz-rs`. Each is a real review cost;
none is claimed "verified." cargo-vet.9+ on demand.

**No crate was admitted that was not fully understood.** "Vetting Succeeded" ≠ "every dependency reviewed."
