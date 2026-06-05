# cargo-fuzz receipt — 2026-06-04 — T23.cargo-fuzz.2 (fix the 3 findings + re-verify)

> **Outcome: the 3 panic-on-hostile-input findings from T23.cargo-fuzz.1 (F1/F2/F3) are FIXED, each
> with its preserved minimized seed as a committed regression witness, and the bounded smoke re-runs
> CLEAN (9/9, 0 crashes).** This **restores the panic-policy claim for the known F1–F3 seeds and the
> stated bounded rerun** — it does **NOT** assert an exhaustive no-panic proof.

## Before → after (the honest delta)

| | T23.cargo-fuzz.1 (find) | T23.cargo-fuzz.2 (fix + re-verify) |
|---|---|---|
| panic sites found | **3** (F1/F2/F3) | — |
| panic sites fixed | 0 (preserved, not fixed) | **3 / 3** |
| seed replay (each crash seed, single input) | crashed (process abort) | **rc=0, no crash** (4/4) |
| bounded smoke (9 targets, 25 s) | 5 clean · **4 crashed** | **9 clean · 0 crashed** |
| parent gate | green (no `src/` touched) | green — fmt · clippy `-D warnings` · **503 tests** · CORE.1 **341/0/0** |

## Toolchain (version-pinned — identical to T23.cargo-fuzz.1)
- `cargo-fuzz 0.13.1` · `rustc 1.98.0-nightly` · libFuzzer via `libfuzzer-sys` (ASan; `-Cdebug-assertions`
  + crate `overflow-checks=true` ⇒ a wrap/overflow is still a finding).
- Host: x86_64, Linux 7.0.9-1-cachyos. zic-rs: 0.1.0. Budget: **`-max_total_time=25` per target**, `-rss_limit_mb=4096`.
- Commands (verbatim): seed replay `cargo +nightly fuzz run <target> <seed.bin>`; smoke
  `cargo +nightly fuzz run <target> -- -max_total_time=25 -rss_limit_mb=4096 -print_final_stats=1`.

## The 3 fixes (narrow, reference-faithful; `src/` only on the panicking lines)

- **F1 — single-`\n` TZif footer → slice-index panic.** `src/tzif/validate.rs::parse_footer` did
  `&tail[1..tail.len()-1]` after a guard a lone `\n` satisfied (the one byte is both first and last),
  slicing `[1..0]`. **Fix:** `tail.strip_prefix(b"\n").and_then(|t| t.strip_suffix(b"\n"))` → a region
  without **both** the `\n…\n` envelope is a malformed footer → typed `Err("malformed TZif footer")`,
  never an index panic. Empty / `\n\n` / `\nTZ\n` shapes unchanged.
- **F2 — huge hour count → multiply-with-overflow panic.** `src/model/time.rs::parse_hms` did
  `h * 3600 + m * 60 + sec`; `zic` is lenient on the hour count, so a value whose `h*3600` overflows
  i64 panicked (the crate sets `overflow-checks=true` in **all** profiles, so it is release-relevant).
  **Fix:** `h.checked_mul(3600).and_then(|hs| hs.checked_add(m*60+sec))` → out-of-range is a typed
  `Err("time … out of range")` (no silent clamp); the rounding `+1` is also `checked_add`. `m`/`sec`
  are each `< 60`, so only `h*3600` was at risk. The final `i32::try_from` still bounds the usable range.
- **F3 — non-char-boundary `str` slice panic.** `src/source/parser.rs::strip_prefix_ci` did
  `s[..prefix.len()]` guarded only by byte length, slicing through a multibyte code point. **Fix:**
  compare on bytes — `s.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())`. A match
  means those bytes are ASCII (they equal ASCII bytes), so `prefix.len()` **is** a char boundary and
  `&s[prefix.len()..]` is panic-free; any non-ASCII byte is a clean `None` (normal non-match).

**Not changed:** no semantics, no TZif output bytes, no diagnostic codes. The default compile path is
byte-identical → CORE.1 **341/0/0** unchanged.

## Regression witnesses (acceptance #1, #2, #5)

- **Sharp per-function unit tests** (next to each fix): `tzif::validate::tests::single_newline_footer_is_typed_err_not_panic`
  · `model::time::tests::huge_hour_count_is_typed_err_not_overflow_panic`
  · `source::parser::tests::strip_prefix_ci_does_not_slice_through_multibyte_char`.
- **Exact-seed replays** (`tests/fuzz_regressions.rs`, `include_bytes!` of the committed seeds, run
  through the same public entry each fuzz target used): `f1_single_newline_footer_seed_does_not_panic`
  · `f2_huge_hour_seed_does_not_overflow_panic` · `f3_multibyte_prefix_seed_does_not_panic`.
- The 4 minimized seeds remain committed under `../findings/` (F1×2, F2, F3) as the fixtures.

## Seed replay (acceptance #5 — old crash seeds no longer crash)

| seed | target | result |
|---|---|---|
| `F1-validate-footer-posix_footer.bin` | posix_footer | **rc=0, no crash** |
| `F1-validate-footer-tzif_validate_bytes.bin` | tzif_validate_bytes | **rc=0, no crash** |
| `F2-time-multiply-overflow-release_diff_tree.bin` | release_diff_tree | **rc=0, no crash** |
| `F3-parser-char-boundary-zone_rule_link_parser.bin` | zone_rule_link_parser | **rc=0, no crash** |

## Bounded smoke re-run (acceptance #4 — same 9 targets, same 25 s budget) — 9/9 CLEAN

| target | rc | execs | result | (was in .1) |
|---|---|---|---|---|
| aux_table_validator | 0 | 406,402 | ✅ clean | clean |
| manifest_json | 0 | 53,530,592 | ✅ clean | clean |
| path_materialization_model | 0 | 6,998,539 | ✅ clean | clean |
| **posix_footer** | 0 | 8,036,252 | ✅ clean | ❌ crashed (9,533 execs → F1) |
| **release_diff_tree** | 0 | 2,371,883 | ✅ clean | ❌ crashed (961,477 → F2) |
| source_lexer | 0 | 3,010,811 | ✅ clean | clean |
| **tzif_validate_bytes** | 0 | 3,640,229 | ✅ clean | ❌ crashed (35,535 → F1) |
| vendor_oracle_json | 0 | 3,546,221 | ✅ clean | clean |
| **zone_rule_link_parser** | 0 | 2,171,159 | ✅ clean | ❌ crashed (118,321 → F3) |

The 4 formerly-crashing targets now run the **full** budget with deep exploration (millions of execs)
instead of aborting in 0–9 s. Logs in `/tmp/fuzz-smoke-2/` (re-creatable by the verbatim commands).

## What this closes / does NOT claim (honest)

- **Closes:** F1/F2/F3 — the 3 panic-on-hostile-input sites, each with a regression seed; the
  claim-boundary-map "No panic on hostile input" row moves **OPEN → CLOSED for F1–F3**.
- **The exact restored claim (verbatim):** *The panic-policy claim is restored only for the known
  T23.cargo-fuzz.1 findings and the stated bounded fuzz rerun. It is not an exhaustive no-panic proof.*
- **Does NOT claim:** exhaustive fuzzing · coverage saturation · that no other panic exists. A clean
  bounded smoke (25 s/target, small corpora) is execution evidence, not a safety certificate. Longer
  (24 h) campaigns remain an operator/lab task; new findings would be admitted by a new receipt.

## Gate
fmt ✅ · clippy `--all-targets -D warnings` ✅ · **503 tests** (497 → +6) ✅ · CORE.1 sweep **341/0/0** ✅.
The 3 fixes touched only the panicking lines in `src/{tzif/validate.rs, model/time.rs, source/parser.rs}`;
default TZif output is byte-unchanged. The `fuzz/` crate stays detached from the parent gate.
