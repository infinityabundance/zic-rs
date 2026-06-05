# RECEIPT — cargo-valgrind slot → AddressSanitizer substitute — 2026-06-05

**Disposition (typed): `ran_via_substitute_instrumentation` (ASan); native `valgrind` `inconclusive_environment`.**

## Why the substitute

`valgrind` (3.25.1) is installed, but **re-confirmed 2026-06-05** it raises *"Unrecognised instruction …
SIGILL"* on **every** binary on this host — including `/bin/true` (`==…== valgrind: Unrecognised instruction
at address 0x403549d … raise a SIGILL`). This is valgrind's CPU-emulation engine failing on this
CachyOS / AVX-512 host, **not** a zic-rs finding — valgrind never reaches zic-rs's code. The fix (compiler
instrumentation that runs natively, no CPU emulation) is **AddressSanitizer**.

## The real run (AddressSanitizer)

- **Command:** `RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --lib --target x86_64-unknown-linux-gnu`
- **Toolchain:** rustc 1.98.0-nightly (31a9463c6 2026-05-25); ASan from the rustc sanitizer runtime.
- **Host:** x86_64 Linux (CachyOS), AMD Ryzen 7 9800X3D. **Exit:** 0. **Raw:** `raw-asan-2026-06-05.txt`.
- **Scope:** the `tzcompile` library unit tests = **151 tests**, including the highest-value memory surface
  (the `tzif` parse / RFC-9636 validate / data-block decode round-trips + the hostile-input guards:
  out-of-range transition type-index, implausibly-large declared count, single-newline footer, designation
  index, isdst/indicator/utoff validity).

**Result: 151 passed · 0 failed · AddressSanitizer reported NO memory error** (no heap-buffer-overflow,
use-after-free, or leak; an ASan error aborts the process — none did). The run executed natively with **no
SIGILL**, exactly where valgrind could not.

## Non-claims (honest scope)

- ASan instruments **zic-rs's own compiled code + its allocations**; it is not a full `-Zbuild-std` ASan of
  std/deps, and it is **bounded to the code the 151 lib tests exercise** — not all paths, not all inputs.
- zic-rs is `#![forbid(unsafe_code)]`, so the *own-crate* memory surface is structurally minimal; ASan's
  value here is catching runtime heap errors across the actual compiled binary + any dep `unsafe` exercised
  by these tests. It **complements `miri`** (the UB interpreter over the `tzif` core), it does not duplicate it.
- A clean ASan run is evidence of no memory error **in this configuration**, not a proof of memory-safety.
- The authoritative tool list is unchanged: `cargo-valgrind` stays the folder; this records the
  **native-instrumentation substitute** that succeeds where valgrind's emulation cannot run on this host.
