# Design

`zic-rs` is structured as a small compiler. Data flows in one direction, and each stage has
a single responsibility:

```text
source text
   │  source::lexer        bytes → Lines of Fields (comments, quoting, limits)
   ▼
 Lines
   │  source::parser       Lines → typed Rule/Zone/Link records
   ▼
 Database (model)
   │  compile              one zone → in-memory TzifData (offsets, transitions, footer)
   ▼
 TzifData (tzif)
   │  tzif::writer         TzifData → TZif bytes (v1 stub + v2 block + footer)
   ▼
 TZif bytes
   │  fs::output_tree      bytes → safe, atomic file under --out
   ▼
 zoneinfo tree
```

The `compare` module sits to the side as an **oracle**: it runs reference `zic` over the
same source and diffs the result against ours. It is the only code that executes an external
process, and never on the compile path.

## Modules

| Module | Responsibility |
|--------|----------------|
| `source::lexer` | Field splitting; comment/quote rules; 2048-byte and NUL limits |
| `source::names` | Case-insensitive, unambiguous-prefix month/weekday/keyword matching |
| `source::parser` | Build typed `Rule`/`Zone`/`Link` records, incl. zone continuations |
| `model::time` | Parse offsets, `AT`/`SAVE` times, and clock suffixes (`w`/`s`/`u`) |
| `model::calendar` | Proleptic Gregorian math; `lastSun`/`Sun>=N`/`Sun<=N` resolution |
| `compile` | Semantic compilation; **fail-closed** on unsupported constructs |
| `compile::posix_footer` | Synthesise the POSIX `TZ` footer string (exact-or-nothing) |
| `tzif::writer` / `header` / `data_block` | Serialise TZif per RFC 9636 |
| `tzif::validate` | Decode TZif back to semantics (round-trip + oracle decoding) |
| `fs::output_tree` / `atomic_write` | Path safety, atomic writes, link materialisation |
| `compare` | Run reference `zic`; `zdump`/structural diff; comparison report |
| `report` | `support-report`: compile-frontier map over a whole source file, bucketed by reason |
| `structural` | `structural-report`: measured TZif **structural-parity** inventory vs reference `zic` (campaign T9; a *separate axis* from behaviour — see [structural-parity.md](structural-parity.md)) |
| `manifest` | Alias-map + compile-provenance JSON (`zic-rs-*-v1` schemas), dependency-free |
| `hash` | In-house SHA-256 for artifact provenance (NIST-vector tested; not for security) |
| `json` | Shared hand-rolled JSON string escaper (no `serde`) used by `report`/`structural`/`manifest` |
| `cli` | Argument parsing; thin shell over the library |

## Why an in-house calendar

We deliberately avoid `time`/`chrono`/`jiff` for date arithmetic. `zic`'s transition model
has conventions those libraries don't share (day spill across months, `24:00`+ times, far-
past years). Owning the arithmetic — and testing it against reference `zic` — is the only
reliable way to match the oracle exactly. The civil↔days conversion is Howard Hinnant's
algorithm; see `model::calendar`.

## Determinism

Output is deterministic: zone selection order is source order (or the explicit list); the
designation table is built in type order; nothing depends on wall-clock time or hashing of
unordered collections in a way that reaches the output bytes. The only non-deterministic
artefacts are *temporary* file names during atomic writes, which never survive the rename.
