# Acknowledgements

zic-rs exists because of the long-running work of the IANA Time Zone Database community — the
original Olson timezone code, the `zic` compiler, the TZif format, and the many maintainers,
reviewers, standards authors, downstream distributors, and civil-time researchers who have
kept this infrastructure working for decades.

> **Not affiliated, not endorsed.** This project is **not affiliated with, endorsed by, or
> maintained by** IANA, ICANN, the TZ Coordinator, the IETF, or the original/current tzdb
> maintainers. Any bugs, incompatibilities, omissions, or misinterpretations in zic-rs are
> the responsibility of **this project alone**. zic-rs studies and mechanically reproduces
> upstream behaviour; it does not speak for the upstream project or its contributors.

A note on attribution: the names below are credited **as recorded in their sources** (the
cited RFCs and the tzdb `NEWS` / release history). We do not assert independent verification
beyond those sources, and the contributor lists are **non-exhaustive** — the authoritative
record is the upstream archive (see the follow-up note at the end).

## Original timezone database and reference code

First and foremost, zic-rs acknowledges **Arthur David Olson**, whose original timezone
database, reference code, mailing-list coordination, and long stewardship created the
foundation this project studies and attempts to reimplement safely in Rust. The name "Olson
database" reflects that founding role. zic-rs treats the Olson/IANA work as the upstream
authority, not as something to supersede.

## Current and long-term stewardship

zic-rs acknowledges **Paul Eggert** for his long-running editorial, maintenance, data,
naming, release, and reference-code work on the IANA Time Zone Database. The modern tz
database and reference `zic`/`zdump` behaviour are this project's central oracles.

zic-rs also acknowledges **Rob Elz** for stewardship recognized in the IETF maintenance
process, and the **ICANN / IANA** team for hosting and supporting the modern IANA Time Zone
Database process (per [RFC 6557](https://www.rfc-editor.org/rfc/rfc6557.html) / BCP 175).

## The TZ mailing-list community

The IANA Time Zone Database is not just a codebase; it is a living civic-technical archive
maintained through reports, citations, corrections, review, disagreement, consensus, and
release work. zic-rs acknowledges the **`tz@iana.org` mailing-list community**: the
engineers, historians, standards participants, national and regional contributors, downstream
distributors, and careful users who supply legislation and historical evidence, correct
civil-time data, identify ambiguous behaviour, find bugs and portability/security issues, and
keep the database usable across operating systems and language runtimes.

## TZif and standards authors

The current TZif specification, **[RFC 9636](https://www.rfc-editor.org/rfc/rfc9636)** (which
obsoletes RFC 8536), is authored by **Arthur David Olson, Paul Eggert, and Kenneth Murchison**
and acknowledges **Michael Douglass, Ned Freed, Guy Harris, Eliot Lear, Alexey Melnikov, and
Tim Parenti**.

The tz-database maintenance process, **RFC 6557 / BCP 175**, acknowledges contributors and
reviewers including **Marshall Eubanks, S. Moonesamy, Peter Saint-Andre, Alexey Melnikov,
Tony Finch, Elwyn Davies, Alfred Hoenes, Ted Hardie, Barry Leiba, Russ Housley, Pete Resnick,
and Elise Gerich**.

These documents matter to zic-rs because the project does **not** attempt to define timezone
truth — it compiles IANA tzdb source according to reference `zic`/`zdump` behaviour and the
TZif format specification. See [docs/standards.md](docs/standards.md).

## Recent and ongoing contributors (non-exhaustive)

The tzdb/tzcode release history (`NEWS`) records many individual contributions: overflow and
adversarial-input findings, portability fixes, legislative corrections, historical-timestamp
research, downstream-compatibility reports, and documentation improvements. zic-rs gratefully
acknowledges that work. Names appearing in recent release notes include, **but are by no means
limited to**:

> Naveed Khan, Arthur Chan, Renchunhui, Christos Zoulas, Dag-Erling Smørgrav,
> G. Branden Robinson, Judah Levine, Heitor David Pinto, Alois Treindl, Yonathan Dossow,
> Roozbeh Pournader, P Chan, Derick Rethans, Justin Grant, Mark Davis, Guy Harris,
> Tim Parenti, Zhanbolat Raimbekov, Heba Hamad, Thomas M. Steenholdt, Zakhary V. Akulov,
> Đoàn Trần Công Danh, Chris Walton, Yoshito Umaoka, Gilmore Davidson, Martin Burnicki,
> Rany Hany, Saadallah Itani, Ahmad ElDardiry, Almaz Mingaleev, Houge Langley,
> Ken Murchison, Rune Torgersen, Evgeniy Gorbanev.

This list is **non-exhaustive** and may contain transcription errors; the authoritative record
is the IANA tzdb **`NEWS`** file, the source history, and the mailing-list archive. A planned
follow-up (below) will derive a complete, mechanically-extracted credit list from `NEWS`.

## Downstream distributors and runtime implementers

zic-rs acknowledges the operating-system and libc maintainers, language-runtime authors,
database vendors, embedded-system maintainers, package maintainers, and application developers
who distribute, test, ship, and update tzdb data. Civil time works only because of this large,
often invisible ecosystem.

## Rust timezone ecosystem

zic-rs is a **producer-side** Rust compiler/build tool, not a datetime library. It is informed
by the needs of Rust timezone **consumers/bundlers** — including `tz-rs`, `tzdb`, `jiff`, and
`jiff-tzdb` — which clarified the value of reproducible, provenance-stamped, oracle-tested
TZif generation (notably the alias-duplication / generation-option questions in jiff#258). See
[docs/rust-ecosystem.md](docs/rust-ecosystem.md).

## Legal and public-domain history

zic-rs acknowledges the public-domain posture of the IANA tzdb code and data, and the legal
history around the database, including the 2011 *Astrolabe, Inc. v. Olson* matter and the
subsequent protection of the database's continued public availability. zic-rs ingests no
proprietary timezone datasets as fixtures and claims no ownership over the IANA tzdb data,
reference `zic` behaviour, or the TZif format. See [docs/legal-history.md](docs/legal-history.md).

## Project stance

zic-rs is an independent Rust implementation effort with a narrow goal: compile a *declared
subset* of IANA tzdb source into TZif using memory-safe Rust, with explicit provenance,
fail-closed handling of unsupported syntax, and reference-`zic`/`zdump` oracle comparison. It
does **not** decide timezone policy, curate timezone history, or "correct" IANA data. It
attempts to learn from, honour, and reproduce the upstream timezone infrastructure as
precisely and transparently as possible.

---

*Follow-up (roadmap):* generate `docs/upstream-credits.md` mechanically from the tzdb `NEWS`
/ release history, so named-contributor credit can be tracked systematically and completely
without bloating this file.
