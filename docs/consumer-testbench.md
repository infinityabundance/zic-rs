# Consumer test bench (optional ecosystem interop)

zic-rs is a **producer**: it generates TZif. A natural question is *"can real Rust timezone
consumers actually read and use what it produces?"* The `ecosystem-tests` feature answers that
with an **interoperability** bench — deliberately separate from, and subordinate to, the
correctness oracle.

```sh
cargo test --features ecosystem-tests        # opt-in; default test run never touches it
```

## What it proves — and what it does not

This bench shows generated TZif is **readable and usable** by a selected real Rust consumer,
at the exact instants that expose each semantic trap in the
[behaviour ledger](reference-zic-semantics.md). It is **not** a correctness oracle and does not
claim conformance with any consumer's own model. The correctness hierarchy is strict:

| Layer | Source of truth | Role |
|------:|-----------------|------|
| 1 | reference `zic` / `zdump` behaviour over a declared horizon | **binding contract** |
| 2 | RFC 9636 structural validity (our `tzif::validate` round-trip) | format correctness |
| 3 | zic-rs's own parser/semantic tests (`cargo test`) | internal correctness |
| 4 | **this consumer bench** | interop: generated TZif is readable/usable by a real consumer |

A disagreement at layer 4 is investigated against the layer-1 `zic`/`zdump` oracle **first** —
"`tz-rs` says X" is never authority over reference `zic`. Consumers validate *usefulness*, not
*correctness*.

## Consumers

| Consumer | Role | Default dep? | What it proves here |
|----------|------|:------------:|---------------------|
| [`tz-rs`](https://crates.io/crates/tz-rs) | pure-Rust TZif **file reader** (`TimeZone::from_tz_data`) | **no** (optional, `dep:tz-rs`, behind `ecosystem-tests`) | offset / `is_dst` / abbreviation a consumer observes at trap instants, incl. **footer-projected** future |
| `jiff` | datetime library w/ bundled/system tzdb | no — deferred | its `include!`/bundled-DB API is awkward for one-off zone files; only after tz-rs is clean, and only if it can be done without contortion |

`tz-rs` is the cleanest first target because it consumes a TZif *byte slice / file path*
directly, which is exactly what zic-rs emits. Jiff is intentionally **deferred**, not skipped.

## Assertions (ledger-derived, behaviour-level)

Each case compiles a fixture with zic-rs (`compile_zone_to_bytes`), loads the bytes with
`tz::TimeZone::from_tz_data`, and asserts `(ut_offset, is_dst, designation)`:

* **`Etc/UTC`** → `(0, false, "UTC")`; **`Test/Fixed`** → `(-18000, false, "EST")`.
* **`Test/Simple`** spring/fall to the second: `2020-03-08 06:59:59Z` EST → `07:00:00Z` EDT;
  `2020-11-01 05:59:59Z` EDT → `06:00:00Z` EST.
* **`Test/Eastern`** at `2040` — *beyond* zic-rs's explicit transitions (`RECUR_HI = 2037`), so
  governed purely by the POSIX footer `EST5EDT,M3.2.0,M11.1.0`: winter → EST, summer → EDT.
  This is the strongest claim — the consumer reads our **footer**, not just explicit transitions.
* **`Test/MidDst`** at the mid-DST era boundary `1990-07-01 04:00:00Z`: EDT → AST (same
  `utoff -14400`, different DST flag/abbr — the one-hour trap a naive reader would miss).
* **`Europe/London`** (first real IANA slice): historical BST (`1980-07`), and footer-projected
  `2030` (GMT in winter, BST in summer).

## No overclaiming

The honest claim is: *"optional smoke tests show zic-rs-generated TZif is read correctly by the
`tz-rs` consumer at the tested instants."* We do **not** claim "Jiff-compatible", "the standard
Rust generator", or general consumer conformance. The bench is opt-in, the consumer crate is
never in the default dependency graph, and the binding correctness contract remains reference
`zic`/`zdump`.

## Environment / CI

The dependency was admitted only after empirically confirming `tz-rs` is fetchable and compiles
in this environment (it has zero extra transitive deps). CI runs the bench as a **separate
optional job** (`cargo test --features ecosystem-tests`); the default job is unchanged and does
not fetch `tz-rs`.

> **Intentional CI choice (not an oversight):** the default `check` job runs
> `cargo clippy --all-targets` **without** `--all-features`, so it does *not* enable
> `ecosystem-tests` or pull `tz-rs` into the main graph. The feature is linted and tested only
> in the dedicated optional `ecosystem` job. This is deliberate — the optional consumer
> dependency must never be on the critical path of the default build/test — so "why not
> `--all-features` by default?" has a definite answer: to keep the default graph minimal and
> the correctness contract (reference `zic`/`zdump`) independent of any consumer crate.
