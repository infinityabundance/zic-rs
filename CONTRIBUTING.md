# Contributing to zic-rs

zic-rs is **infrastructure code with a court around it**: every compatibility claim is tied to an admitted
reference, a typed contract, an oracle mode, a machine-readable report, and an explicit non-claim. Read
[`docs/reviewer-orientation.md`](docs/reviewer-orientation.md) first — it is the frame. This file is the
working contract for changes.

## The gate (every change must keep it green)

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                      # the default suite
cargo doc --no-deps
bash /tmp/t9sweep.sh            # CORE.1 → 341 match / 0 mismatch / 0 fail-closed (1900..2040)
```

**CORE.1** (all 341 canonical zones in `tzdata.zi` 2026b behaviour-match reference `zic`/`zdump` over
`1900..2040`) is the standing regression gate. A change that moves it must be deliberate and explained.

## Doctrine (non-negotiable)

- **Reference first · evidence second · behaviour third · claims last.** Pin reference `zic`/`zic.c` →
  diagnose → implement narrowly → full-sweep gate → seal. Don't guess at `zic` behaviour; read the source.
- **No overclaiming.** `compile-clean ≠ behaviour-match`; behaviour claims are phrased *"over 1900..2040"*;
  structural / byte / diagnostic / operational parity are **separate axes**, never collapsed. A safer
  default is **not** "parity" — it is a labelled bucket-3 divergence. If a human can read it as a claim, a
  machine must locate the field that proves or bounds it.
- **Typed claim surfaces.** A finite-vocabulary, claim-bearing value is a Rust enum rendered to its literal
  at the JSON boundary (CONTRACT.TYPING; `reports/contract-typing-audit.md`) — never a free string.
- **Fail closed.** Unsupported/malformed input gets an explicit diagnostic; never emit an approximate file.
  No panic on untrusted input ([`docs/panic-policy.md`](docs/panic-policy.md)).
- **`#![forbid(unsafe_code)]`** (crate + binary) — there is no `unsafe`. **No `build.rs`.** **Minimal
  dependencies** — adding a runtime dependency to the core crate needs a strong, documented reason
  (the `fuzz/` crate and the optional `ecosystem-tests` feature are separate and never in the default
  graph). `overflow-checks = true` in all profiles.
- **Same-batch seal (the operating rule).** A change is not done until its **docs + the plan + the
  SESSION-CONTEXT + memory** ship in the **same batch** as the code, with the gate green. Stale
  doctrine-bearing prose is a defect (comments and docs carry claims here).
- **Documentation is dense on purpose.** Optimise docs for evidentiary completeness, auditability, and
  adversarial verification — not for skimming. Reorganisation is **additive only** (index/anchors/tables/
  cross-links/append-only ledgers); never shrink, simplify, or move the dense record out. (The lab README
  is a field notebook, not a pamphlet.)

## Commentary

Comments explain **intent and claim boundaries**, not just "what this does": why a boundary exists, what a
type owns, what it must not be confused with, the historical trap that shaped it, which report consumes it,
which future milestone owns the deferred part. A reviewer should be able to reconstruct the *decision tree*,
not only re-run the commands. Keep comments current in the same change (stale doctrine comments are a defect).

## Schema / CLI changes

Reports and the CLI are public contracts. Before changing emitted JSON or CLI behaviour, read
[`docs/schema-compatibility-policy.md`](docs/schema-compatibility-policy.md) and
[`docs/cli-compatibility-policy.md`](docs/cli-compatibility-policy.md): bump a schema's `vN` only on a
**meaning** change (additive optional fields with a documented default do not bump), keep public literals
enum-owned, render `unknown` as a typed value (never silence), and keep `tests/schemas.rs` (the registry/
drift guard) + the goldens green.

## Tests

Tests are the gate, not an afterthought. New behaviour lands with: an oracle-pinned or fixture-backed test;
totality tests for new enums; a golden update (reviewed + bounded) if it touches a pinned report;
hostile-input coverage where the surface is reachable from untrusted bytes; and CORE.1 unchanged unless the
change is *about* CORE.1. The optional consumer bench is `cargo test --features ecosystem-tests` (never the
correctness oracle). Fuzz targets live in `fuzz/` (run with nightly + `cargo-fuzz`; receipt-bearing — see
`fuzz/README.md`).

## Milestone discipline

Work follows the single contiguous **T0 → T23** ladder in
the project milestone plan (maintained out-of-tree)
(execution order == numeric order; later proposed arcs T22+ are tracked in the plan's roadmap block).
Each milestone ends green and sealed with a receipt where it closes (`reports/t*-close-receipt.md`).
