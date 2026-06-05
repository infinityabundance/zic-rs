# T14 — Hostile-input & parser-edge parity: closure receipt

> **T14 CLOSED.** Reference-first, classify-first, fail-closed. T14 took zic-rs from "works on normal
> input" to "survives hostile input responsibly" — every substep pinned reference `zic`/`zic.c` 2026b,
> made the result an **executable witness** (the T13 lesson), and recorded residuals rather than hiding
> them. **Gate at close: 360 tests · `fmt`/`clippy -D warnings`/`doc` clean · CORE.1 341/0/0** (no valid
> `tzdata.zi` behaviour changed across all of T14). Diagnostic contract grew **ZIC001–ZIC020 → ZIC001–ZIC025**,
> every new code added through the machine-checked totality path (`layer()`/`span_precision()`/`default_severity()`
> exhaustive + the contract-metadata test).

## The arc

| Substep | What | Headline | Code(s) | Doc / witness |
|---------|------|----------|---------|---------------|
| **T14.1** | admissibility **inventory** (reference-first; no behaviour change) | line-cap **2048 matches** pinned `_POSIX2_LINE_MAX` (the "511" manpage was stale — verified against the pinned source); made executable from day one | — | `zic-hostile-input-parity.md` · `tests/input_admissibility.rs` |
| **T14.2** | lexical **fatal tightening** (first behaviour change) | missing-final-newline + odd-quote now fatal, matching `inputline`/`getfields`; T14.1 witness flipped lenient→fatal | `ZIC021` `ZIC022` | same doc · `tests/diagnostic_parity.rs` |
| **T14.3** | metamorphic **source-ordering law** (contract-level) | permuted records / comments / inter-field whitespace → **byte-identical** per-zone output; continuation exception enforced; source identity differs | — | `tests/metamorphic_ordering.rs` |
| **T14.4** | **pathology ledger** (classify-first) — and **removed a panic** | "two rules for same instant" (reference fatal) was a `debug_assert!` → panic in debug / invalid TZif in release; now fails closed | `ZIC023` | `zic-pathology-ledger.md` · `tests/pathology_ledger.rs` |
| **T14.5** | **`ZoneNamePathPolicy`** (name-as-path) | fatal `ZIC008` policy already matched reference `namecheck`; added the `-v` portability warnings the ledger surfaced | `ZIC024` `ZIC025` | `zic-zone-name-path-policy.md` · `tests/zone_name_path_policy.rs` |
| **T14.6** | **hostile-output-tree (TOCTOU)** boundary | pre-planted file/symlink/dir at the leaf fail closed, never written through; concurrent parent-swap race honestly `NotClaimed` | — | `zic-hostile-output-tree.md` · `tests/hostile_output_tree.rs` |

## New coded diagnostics (ZIC021–ZIC025)

| Code | Class | Layer | Severity / verbosity | Reference pin |
|------|-------|-------|----------------------|---------------|
| `ZIC021_UNTERMINATED_INPUT_LINE` | missing final newline | Lexical | Error / AlwaysOn | `inputline` "unterminated line" (fatal) |
| `ZIC022_UNTERMINATED_QUOTE` | odd quotation marks | Lexical | Error / AlwaysOn | `getfields` "Odd number of quotation marks" (fatal) |
| `ZIC023_SIMULTANEOUS_TRANSITION` | two rules at one instant | Semantic | Error / AlwaysOn | `zic.c` "two rules for same instant" (fatal) |
| `ZIC024_ZONE_NAME_NONPORTABLE_BYTE` | non-benign name byte | Warning | Warning / VerboseOnly | `namecheck` "contains byte" (`-v`) |
| `ZIC025_ZONE_NAME_OVERLENGTH_COMPONENT` | >14-byte component | Warning | Warning / VerboseOnly | `componentcheck` "overlength component" (`-v`) |

## Headline finding — a panic on untrusted input, removed

The pathology ledger (T14.4) was not bureaucratic overhead: it immediately found that **"two rules for
the same instant"** — which reference `zic` treats as a fatal error — was guarded in zic-rs only by a
`debug_assert!(strictly increasing)`, i.e. a **panic in debug builds and a non-monotonic (invalid) TZif
in release builds**. The fix is a real `ensure_strictly_increasing()` guard on every compiled stream →
fail-closed `ZIC023`. A panic on untrusted input is now a controlled, coded rejection — replacement-grade
hardening, and a direct win for the project's panic policy.

## Intentional divergences (recorded, classified)

* **Stricter than reference (bucket 3 — safer divergence):** leading-`-` zone-name component rejected
  (reference only `-v`-warns); NUL rejected; UTF-8 required (reference is byte-oriented).
* **Verbosity:** zic-rs has no quiet mode, so it always *collects* `-v`-gated warnings; the CLI prints
  them only under `--verbose` (matching `zic` vs `zic -v`).
* **One warning per name, not per byte:** `ZIC024`/`ZIC025` are class/location parity with `zic -v`, not
  per-byte count parity.

## Residuals handed off (named, not hidden)

* **Multi-era same-instant** where wall→UT `save_prev` separates the activations still *accepts* (valid
  output; reference errors) — bucket 4 → **T15** (semantic-witness can catch it).
* **`-v` warning "values over 24 hours"** (offset/SAVE magnitude portability) — not yet emitted → **T15**
  warning-parity tail.
* **Concurrent hostile-output-tree TOCTOU** (parent-component symlink-swap mid-run) — `NotClaimed`,
  needs fd-relative `openat`/`O_NOFOLLOW` materialization → **T17/T20** (gated behind the `unsafe`/dep
  decision).
* **Windows reserved names / case-insensitive collisions** — `PlatformDependent`, ledgered → **T16/T17**.
* **QEMU-backed vendor diagnostic oracle lab** (run the T13 + T14 fixture corpus against real vendor
  `zic` binaries) → **T16.5**.

## Non-claims (T14)

* Not an exhaustive hostile-input/pathology enumeration — the inventories are seeded with the major
  classes, not every conceivable degenerate input.
* No byte-exact reference **stderr** parity — comparison is class · location · severity, wording last.
* No full concurrent-TOCTOU resistance, no non-Unix output-safety parity, no `-v` warning-text parity.
* Diagnostics remain `diagnostic_artifact`s — they prove the tool *rejects/notices* ill-formed input at
  the right layer, never anything about compiled output.

## Next

**T15** — `support-report` as the public conformance engine: `zdump` **semantic witnesses**,
**oracle-availability** (`oracle_mode`), **`negative_capabilities`** JSON, the one-line machine status,
RFC 9636 TZif **structural validator**, and the evidence-category field — each born typed per the
**CONTRACT.TYPING** apply-in-place rule (T17 sweep ratifies).
