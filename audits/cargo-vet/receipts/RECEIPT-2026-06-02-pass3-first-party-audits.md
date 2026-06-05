# T23.cargo-vet.3 — first-party audit pass — 2026-06-02

- **Tool:** `cargo-vet`. **Host:** x86_64, Linux. **zic-rs:** 0.1.0. **Exit:** `cargo vet` = 0. **Raw:** `raw-2026-06-02-pass3.txt`.
- **Reviewer:** `zic-rs maintainer (full-source review)` — see the **non-claims** (this is a full read of *small* crates, not a professional security audit).

## Before → After (dependency coverage; 66 third-party total)

| | total deps | trusted-import covered | exempted (unaudited) | first-party audited |
|---|---:|---:|---:|---:|
| **Before** (T23.cargo-vet.2) | 66 | 23 (22 full + 1 partial) | 43 | 0 |
| **After** (T23.cargo-vet.3) | 66 | 23 (22 full + 1 partial) | **39** | **4** |

`cargo vet` → *"Vetting Succeeded (26 fully audited, 1 partially audited, 39 exempted)"* (26 = 22 import + 4
first-party). Certifying each crate **auto-removed its exemption** (43 → 39).

## Audited crates (each read in full before certifying)

| Crate | Version | Criteria | Reviewed (files / LoC) | Why low-risk | Result |
|---|---|---|---|---|---|
| `cfg-if` | 1.0.4 | `safe-to-deploy` | `src/lib.rs`, 212 LoC | pure declarative `macro_rules!` cfg-cascade; `#![no_std]`; **emits no runtime code**; 0 `unsafe`; no I/O; no `build.rs` | ✅ certified |
| `colorchoice` | 1.0.5 | `safe-to-deploy` | `src/lib.rs`, 115 LoC | `#![no_std]`; a `ColorChoice` enum behind an `AtomicUsize` (get/set); 0 `unsafe`; no I/O; no `build.rs`; the lone `expect()` is safe-by-construction (only valid discriminants are stored) | ✅ certified |
| `is_terminal_polyfill` | 1.70.2 | `safe-to-deploy` | `src/lib.rs`, 50 LoC | sealed `IsTerminal` trait forwarding to `std::io::IsTerminal` for std handles; 0 `unsafe`; no I/O of its own; no `build.rs`; MSRV polyfill | ✅ certified |
| `once_cell_polyfill` | 1.70.2 | `safe-to-deploy` | `src/lib.rs` + `src/sync/mod.rs`, 51 LoC | thin newtype over `std::sync::OnceLock` forwarding every method; 0 `unsafe`; no I/O; no `build.rs`; MSRV polyfill | ✅ certified |

**`reason for `safe-to-deploy`:** each is `no_std`-or-thin-std-forwarding, allocation-light, with **zero
`unsafe`, no filesystem/network/process/env access, and no `build.rs`** — there is no surface through which
the crate could introduce a serious vulnerability to production software exposed to untrusted input. That is
exactly what `safe-to-deploy` attests, and it is justified by a full read of the (tiny) source.

## Commands

```sh
cargo vet                              # before: 22 fully audited, 1 partial, 43 exempted
# (read each crate's full source first)
cargo vet certify cfg-if 1.0.4               --criteria safe-to-deploy --who "…" --notes "…" --accept-all
cargo vet certify colorchoice 1.0.5          --criteria safe-to-deploy --who "…" --notes "…" --accept-all
cargo vet certify is_terminal_polyfill 1.70.2 --criteria safe-to-deploy --who "…" --notes "…" --accept-all
cargo vet certify once_cell_polyfill 1.70.2   --criteria safe-to-deploy --who "…" --notes "…" --accept-all
cargo vet                              # after: 26 fully audited, 1 partial, 39 exempted
```

## Non-claims

- **This does not mean the whole dependency graph is audited.** 39 of 66 deps remain **exempted = UNAUDITED**.
- **This does not mean `unsafe` dependencies are verified.** The four audited crates were chosen *because*
  they have zero `unsafe`; the `unsafe`-bearing crates (`libc`, `rustix`, `linux-raw-sys`, `getrandom`,
  `itoa`, `fastrand`, `unicode-ident`, …) are **deliberately left** to trusted-import/exemption pending a
  proper review.
- **This does not replace `cargo-audit`** (advisory scan — separate, `../cargo-audit/`, clean).
- **This does not imply supply-chain-compromise resistance** (no SBOM/SLSA/signing; those are planned, `docs/security-rewrite-evaluation.md` §D).
- **Exempted dependencies remain unaudited** and are not described as reviewed anywhere.
- **Reviewer caveat:** the audit is an **full read of small crates**, recorded transparently in
  the `who` field — *not* an independent professional security audit. A human re-review before any
  high-assurance claim is appropriate.

## Status · next

- **Status:** ◐ **RUN (T23.cargo-vet.3) — 4 first-party audits added; 66→39 exempted; `cargo vet` exit 0.** No exempted dep promoted silently.
- **T23.cargo-vet.4 (next, on demand):** review the next tier of small zero-/low-`unsafe` crates (`anstyle`,
  `r-efi`, `is_terminal_polyfill`'s siblings) and eventually the `unsafe`-bearing leaves with real review time.
