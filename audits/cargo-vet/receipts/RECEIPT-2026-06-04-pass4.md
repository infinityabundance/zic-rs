# cargo-vet receipt — 2026-06-04 — T23.cargo-vet.4 (dependency trust reduction)

> **Receipt wording (binding):** *T23.cargo-vet.4 reduces the unaudited dependency surface by reviewing a
> bounded low-risk dependency tier. It does not claim full supply-chain verification, does not claim all
> transitive dependencies are trusted, and leaves unreviewed crates explicitly exempted.*

## Before → after (exact counts)

| | `cargo vet` summary | first-party audits | exempted / UNAUDITED |
|---|---|---|---|
| **before** (T23.cargo-vet.3) | 26 fully + 1 partial audited | 4 | **39** |
| **after** (T23.cargo-vet.4) | **29 fully + 1 partial audited** | **7** | **36** |

`cargo vet` (0.10.2) final: **`Vetting Succeeded (29 fully audited, 1 partially audited, 36 exempted)`**.
Net change: **+3 first-party audited, 39 → 36 exempted.** Counts stay honest:
fully-audited + partial + exempted = the full third-party graph; **exempted is never called "audited."**

## Config fix (required to run, post-publish)

`cargo vet` failed before the review with *"Some non-crates.io-fetched packages match published crates.io
versions: zic-rs:0.1.0"* — because zic-rs was published to crates.io after the T23.cargo-vet.3 pass. Added
`[policy.zic-rs] audit-as-crates-io = false` to `supply-chain/config.toml`: the audit target is **this local
workspace**, not the published crate. Recorded transparently; no audit content was affected.

## The tier (selection criteria)

Picked crates that are small, stable, no `build.rs`, no proc-macro, no network, no crypto-strength claim,
no filesystem mutation, and whose `unsafe` (if any) is minimal and reviewable in one sitting. **Syscall-heavy
crates (libc/rustix/linux-raw-sys/errno/windows-sys/getrandom) and proc-macro/SIMD-`unsafe` crates were NOT
touched.** Three candidates that were *examined and deferred* (honest under-admission): `itoa` (13 `unsafe`
ptr-write sites), `anstyle-parse` (`MaybeUninit::uninit().assume_init()` vte-table build), `anstyle-query`
(mutates the Windows console via `SetConsoleMode` FFI), `semver` (47 `unsafe`), `log` (5673 LoC, global
logger state). These need real review time → **T23.cargo-vet.5**, not rushed here.

## Crates admitted this pass (each read in full → `safe-to-deploy`)

| crate | version | license | repo | unsafe | build.rs | proc-macro | network | fs | reason admitted |
|---|---|---|---|---|---|---|---|---|---|
| `anstyle` | 1.0.14 | MIT OR Apache-2.0 | github.com/rust-cli/anstyle | **1 site** (sound) | no | no | no | no | ANSI style descriptors, `no_std`, zero deps, no I/O; the lone `unsafe` is `from_utf8_unchecked` on a `&str`-only `DisplayBuffer` (bytes valid UTF-8 by construction) |
| `fastrand` | 2.4.1 | MIT OR Apache-2.0 | github.com/smol-rs/fastrand | **none** (`#![forbid(unsafe_code)]`) | no | no | no | no | non-crypto Wyrand PRNG; zero unsafe (compiler-enforced); global seed = benign hash of `Instant::now()` + thread id (std only); reached only via `tempfile` scratch-names, never for secrets |
| `clap_lex` | 1.1.0 | MIT OR Apache-2.0 | github.com/clap-rs/clap | **6 sites, 1 pattern** (sound) | no | no | no | no | CLI arg lexer, zero deps; only I/O is `std::env::args_os()`; all `unsafe` is the std-blessed `OsStr::from_encoded_bytes_unchecked` on `&str`/UTF-8-boundary splits — `split_at` documents `index must be at a valid UTF-8 boundary`, callers pass `&str` prefix lengths / `Utf8Error::valid_up_to()`. Reviewed each site. |

The full per-crate `unsafe`-review notes are in `supply-chain/audits.toml` (`[[audits.anstyle]]`,
`[[audits.fastrand]]`, `[[audits.clap_lex]]`), each with `who = "… T23.cargo-vet.4"`.

## What this is NOT (non-claims)

- **Not full supply-chain verification.** 36 crates remain exempted/UNAUDITED; this pass reduced the surface,
  it did not clear it.
- **Not "all transitive dependencies trusted."** Trusted-import coverage (Mozilla/Google/Bytecode-Alliance)
  is delegated trust, separate from our 7 first-party reviews.
- **vetted ≠ bug-free.** A `safe-to-deploy` audit attests "no obvious malicious/unsafe behaviour on full read,"
  not correctness.
- **fastrand is not a CSPRNG** — admitted as a non-security PRNG; it is never used for secrets.

## Gate

Supply-chain metadata only — **no `src/` change**, so the parent gate (fmt · clippy `-D warnings` · 503 tests ·
CORE.1 341/0/0) is unaffected and unchanged. `cargo vet` Vetting Succeeded. **Next owner:** T23.cargo-vet.5
(the `unsafe`-heavier leaves, with real review time).
