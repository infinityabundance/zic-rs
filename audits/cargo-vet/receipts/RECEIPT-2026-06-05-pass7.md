# RECEIPT — T23.cargo-vet.7 (the small-low-unsafe tranche) — 2026-06-05

> Part of **T23.AUDIT-BOARD.1**. Same standard as every pass: each crate read in full, every `unsafe` site
> reasoned, nothing admitted that was not understood. Does not claim audited crates are bug-free.

## Before → after

| | pass6 | pass7 |
|---|--:|--:|
| fully audited | 34 | **39** |
| partially audited | 1 | 1 |
| exempted | 31 | **26** |
| first-party reviews | 12 | **17** |

`cargo vet` → **Vetting Succeeded (39 fully audited, 1 partially audited, 26 exempted)** (`raw-…-pass7.txt`).

## The 5 admitted (the tractable tranche — small, zero/low real-unsafe)

- **thiserror 2.0.18** — a **direct** zic-rs dep; **0 unsafe**; build.rs is the standard dtolnay cfg-probe
  (compiles a tiny `probe.rs`, writes only OUT_DIR, no network).
- **thiserror-impl 2.0.18** — the `#[derive(Error)]` proc-macro; **0 real unsafe** (the lone grep hit is the
  string literal `Keyword("unsafe")` in its expression scanner, not an unsafe block).
- **id-arena 2.3.0** — **`#![forbid(unsafe_code)]`** (the grep hit *is* the forbid attribute).
- **unicode-ident 1.0.24** — 2 trie-table `get_unchecked` lookups, offset bounded by table construction
  (the standard codegen-bounded trie).
- **windows-link 0.2.1** — a Windows-only `raw-dylib` linkage `macro_rules!`; 0 unsafe, no runtime code.

## Still exempted (26) — honestly deferred, not faked

The genuinely large / syscall-heavy / generated tiers stay **explicitly exempted**: `windows-sys` (12530
unsafe sites), `linux-raw-sys` (6932), `rustix` (1702), `libc` (428), `memchr` (335), `r-efi` (324),
`hashbrown` (299), `syn` (153), `getrandom` (111), `anyhow`/`zmij`/`once_cell`/`prettyplease`/`indexmap`/
`serde*`/`clap*`/… — each is a real review cost, none is claimed "verified." cargo-vet.8+ on demand. **No
crate was admitted that was not fully understood.**
