# Oracle testing

The project's central guarantee is **measured compatibility**: our output is compared
against the canonical reference `zic`, not merely asserted to be correct.

## Two comparison modes — and what each actually proves

`compare --mode` selects how the two files are compared. **Be precise about which ran**, so
no result overclaims:

1. **`zdump` behaviour (`--mode zdump`, the default — the real correctness oracle).** Dump
   both files with `zdump -v -c LO,HI` over a **declared year horizon** and diff the
   behaviour line-for-line: UTC instant, local time, UT offset, DST flag, abbreviation.
   Because `zdump` evaluates *both* explicit transitions and the POSIX footer, this is the
   only mode appropriate for **recurring** zones (whose explicit-transition horizons differ
   between implementations while behaviour is identical). Horizon is mandatory and bounded —
   we never compare "the infinite future" vaguely.
2. **decoded-TZif (`--mode structural`).** Decode both files and diff the decoded model
   (footer + transition list + types). Exact and convenient for fixed-offset/finite
   fixtures and debugging, but it **penalises valid representational differences** (type
   ordering, designation sharing, transition cut-off), so it is *not* the correctness
   criterion for recurring zones.

Separately, **byte parity** is a stronger, pinned signal we report when it holds — but only
against reference output **checked in** under `fixtures/expected/` (see
`fixtures/MANIFEST.toml`). We never assert byte parity without a pinned blob.

> Honesty note: earlier drafts described the in-tool comparison as "zdump-match" when it was
> in fact decoded-TZif. The `zdump` behaviour oracle is `--mode zdump` (now the default);
> summaries name exactly which comparison ran ("zdump behaviour match over 2019..2035" vs
> "decoded TZif match").

## Running the oracle

### Via the CLI

```sh
# Behaviour oracle over a declared horizon (default mode):
zic-rs compare --input fixtures/minimal/eastern.zi --zone Test/Eastern --horizon 2019,2035
# => "Test/Eastern: zdump behaviour match over 2019..2035"

# Decoded-TZif comparison (fixed-offset / debugging):
zic-rs compare --input fixtures/minimal --zone Etc/UTC --mode structural
# => "Etc/UTC: decoded TZif match (byte-identical)"
```

`compare` compiles the zone with our compiler, runs reference `zic` over the same source
into a temp directory, and (in `zdump` mode) runs `zdump -v -c` on both absolute paths. It
is the only code path that runs an external `zic`/`zdump`.

### Via `cargo test`

`tests/oracle_compare.rs` runs the comparison for the minimal fixtures. It **auto-skips**
when no `zic` is on `PATH`, so the default `cargo test` never *requires* a system `zic`. CI
runs it in a dedicated job that installs tzcode.

### Via `zdump` (manual cross-check)

```sh
zic   -d /tmp/ref   fixtures/minimal/fixed.zi
zic-rs compile --input fixtures/minimal --out /tmp/ours --zone Test/Fixed
diff <(zdump -v /tmp/ref/Test/Fixed) <(zdump -v /tmp/ours/Test/Fixed)
```

> **Note on `zdump` paths:** pass an **absolute** path to the compiled file. A relative name
> is interpreted as a timezone-database lookup (`TZDIR`), not a file, and you will get
> "unknown timezone" — or, worse, *misleading* output read from some other file. This bit us
> twice: first as plain "unknown timezone" errors, and later during the `Test/Mixed` audit,
> where a relative-path `zdump` produced a false "one explicit transition" reading that briefly
> looked like a reference/zic-rs contradiction. Re-running with absolute paths showed the two
> agreed across the historical window. Hence the explicit `$PWD`/absolute paths in scripts, and
> why the `compare` command always resolves to absolute paths. Decode explicit-transition
> *counts* from the TZif header (`timecnt`), never by counting `zdump` lines.

## Reference toolchain

Pinned for reproducibility: reference `zic` and `zdump` are **tzcode 2026b**; the tzdata
source for pinned slices (milestone T4) is `/usr/share/zoneinfo/tzdata.zi` (2026b).
