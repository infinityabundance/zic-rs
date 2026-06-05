# Prior art

Due diligence (Phase 0): does a Rust `zic` — a compiler from tzdata *source text* to binary
*TZif files* — already exist? Surveyed crates.io and GitHub in May 2026.

**Conclusion: no.** Every Rust crate in this space either *reads/consumes* TZif or
*generates Rust data structures*; none writes TZif binaries from tzdata source. `zic-rs`
does not duplicate existing work.

| Crate / project | What it does | Parses tzdata source? | Writes TZif binary? |
|-----------------|--------------|:--------------------:|:-------------------:|
| [`parse-zoneinfo`](https://crates.io/crates/parse-zoneinfo) | Parses tzdata text into in-memory tables (chrono-tz's build dep) | ✅ | ❌ (Rust structs only) |
| `zoneinfo_parse` | Older Olson-DB text parser (superseded by above) | ✅ | ❌ |
| [`zoneinfo_compiled`](https://github.com/rust-datetime/zoneinfo-compiled) | Reads compiled TZif | ❌ | ❌ (reader) |
| [`chrono-tz`](https://github.com/chronotope/chrono-tz) | Generates chrono `TimeZone` impls at build time | via `parse-zoneinfo` | ❌ (emits Rust source) |
| [`tzif`](https://docs.rs/tzif) | Parses TZif binary + POSIX TZ strings | ❌ | ❌ (reader) |
| [`tzfile`](https://github.com/kennytm/tzfile) | chrono impl reading system tz files | ❌ | ❌ (reader) |
| [`tz-rs`](https://github.com/x-hgg-x/tz-rs) | Pure-Rust localtime/gmtime; reads TZif | ❌ | ❌ (reader) |
| [`libtzfile`](https://github.com/nicolasbauw/rs-tzfile) | Parses TZif; can emit JSON | ❌ | ❌ (reader) |
| [`jiff` / `jiff-cli`](https://github.com/BurntSushi/jiff) | Reads TZif; `jiff-cli` **shells out to C `zic`** to build TZif | ❌ | ❌ (delegates to C zic) |

## Closest building block

`parse-zoneinfo` genuinely parses the tzdata source grammar but stops at in-memory Rust
structures — it has no TZif serialiser, and its own docs point binary users to
`zoneinfo_compiled` (a reader). So the *parse half* exists in Rust; the **TZif-writing half
does not exist anywhere**. We implement our own parser anyway (to control diagnostics,
limits, and exact `zic` semantics), but `parse-zoneinfo` is a useful cross-check for the
parse stage.

## Many non-Rust tz compilers/consumers exist — we make no "first" claim

IANA's [tz-link](https://data.iana.org/time-zones/tz-link.html) lists a large ecosystem of
timezone compilers and consumers across languages — e.g. C **Vzic**, Perl
**DateTime::TimeZone**, Howard Hinnant's C++ **`date`** parser/runtime, **ICU**, Java
**java.time**/**Joda-Time**/Noda Time/Time4J, numerous JavaScript packages, Julia, Object
Pascal, Python **pytz**, Ruby **TZInfo**, and others. zic-rs is **not** "the first tz
compiler" or "the first non-C tz compiler" — such claims would be false.

The **honest, narrow claim** is scoped to Rust + our method:

> No obvious **Rust** crate compiles IANA tz *source* into binary **TZif** with
> **reference-`zic`/`zdump` oracle comparison** and **release provenance**. `parse-zoneinfo`
> parses the source but writes no TZif; the rest of the Rust ecosystem *reads* TZif or
> delegates to the C `zic`.

What is distinctive here is the *combination*: memory-safe Rust, a declared/​fail-closed
syntax subset, the hostile `zic`/`zdump` oracle, and (roadmap) traceable release provenance —
not novelty as "a timezone compiler". See [standards.md](standards.md) and
[tzdb-governance.md](tzdb-governance.md).

## Naming

The crate name `zic` on crates.io is a squatted `v0.0.0` placeholder ("Reserved name"), so
publishing under `zic` is not possible; **`zic-rs` is free** and is the name we use.
