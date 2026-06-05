# zic-rs in the Rust timezone ecosystem

Rust already has good timezone **consumers** and **bundlers**. It lacks a Rust-native,
provenance-aware, oracle-tested **producer** — a compiler/build tool that *generates and
audits* TZif from IANA tz source. That producer niche is what zic-rs fills. It does **not**
compete with the libraries below; it sits upstream of them.

```
IANA tz source (Zone/Rule/Link)
        │
        ▼
     zic-rs            ← producer: compile + verify (zic/zdump oracle) + (roadmap) provenance
        │
        ▼
  TZif output tree  +  manifest / oracle report / alias map (roadmap)
        │
        ▼
  consumers / bundles / test fixtures   ← tz-rs, tzdb, jiff, jiff-tzdb, TZif readers
```

## The consumers (and how zic-rs relates)

| Crate | Role | Relationship to zic-rs |
|-------|------|------------------------|
| [`tz-rs`](https://github.com/x-hgg-x/tz-rs) | Pure-Rust `localtime`/`gmtime`/`mktime`; **reads** POSIX TZ strings and TZif files | zic-rs **produces** the TZif files tz-rs reads. tz-rs's own README tells non-Unix users to compile IANA data to a local zoneinfo dir — that compile step is exactly zic-rs's job. |
| [`tzdb`](https://crates.io/crates/tzdb) | **Statically bundles** existing timezone definitions for consumers like tz-rs | zic-rs can generate provenance-stamped compiled data a tzdb-like crate could embed (roadmap). |
| [`jiff`](https://docs.rs/jiff) | High-level datetime library; integrates the **system** IANA tzdb and **embeds** a copy when absent | zic-rs is **not** a datetime library — it is a compiler/build tool that can generate auditable TZif bundles. Different layer entirely. |
| [`jiff-tzdb`](https://docs.rs/jiff-tzdb) | The timezone data jiff bundles | zic-rs can help **document and reproduce** how such a bundle is generated (release, alias policy, generation options). |

## "Why not just use jiff / tz-rs / tzdb?"

Because they answer a different question. They *consume* timezone data at runtime; zic-rs
*produces and verifies* that data at build time. If you need to **read** local time in an
app, use jiff/tz-rs. If you need to **compile IANA source into TZif and prove it matches
reference `zic`/`zdump` for a pinned release**, that's zic-rs.

## The concrete gap: build provenance (jiff#258)

jiff's issue [#258](https://github.com/BurntSushi/jiff/issues/258) asks how `jiff-tzdb` data
is generated: it observes that the concatenated data appears to **duplicate** data for
aliases (2025a had 597 identifiers but only 339 non-alias zones) and asks to document
generation options like `backzone` and `rearguard`. That is precisely the build-side
provenance zic-rs aims to make first-class:

* an **alias/canonical manifest** (zones vs links, with counts and hashes) — roadmap T3.4b,
  directly answering the alias-duplication accounting;
* a **generation-provenance manifest** recording tzdb release + source hash + generation
  options (`backzone`/`backward`/`rearguard`/`slim`/`leapseconds`) + reference `zic`/`zdump`
  versions + oracle result — roadmap T3.4c.

See [generated-data-contract.md](generated-data-contract.md) for what a zic-rs output bundle
will guarantee a consumer, and [roadmap.md](roadmap.md) (T3.4a–d) for the integration lane.

## Honest scope

This positioning is **not** a claim of novelty as "a timezone compiler" — many exist across
languages (see [prior-art.md](prior-art.md)). The narrow, honest claim is: a memory-safe
**Rust** producer with reference-`zic`/`zdump` oracle comparison and (roadmap) release
provenance, complementing the existing Rust consumers.
