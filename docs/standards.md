# Standards & sources — normative vs orientation

To keep zic-rs free of "blog-driven semantics", we separate what we **implement against**
(normative) from what merely **orients** us (history, articles, prose). When the two ever
disagree, normative wins and the behaviour is pinned by experiment against reference `zic`.

## Normative (implementation authority)

| Source | Governs |
|--------|---------|
| **[RFC 9636](https://www.rfc-editor.org/rfc/rfc9636)** — *The Time Zone Information Format (TZif)* | The TZif binary layout we emit and validate: header + counts, the v1 (32-bit) block, the v2+ (64-bit) block, the footer, and the requirement to **bounds-check counted arrays**. **Obsoletes RFC 8536.** |
| **`zic(8)`** + reference **`zic.c`** (tzcode) | tzdata *source* semantics: `Rule`/`Zone`/`Link` grammar, `ON`/`AT`/`SAVE` forms, `UNTIL`, abbreviation handling, slim/fat output. The manpage is the spec; `zic.c`/observed behaviour is the tiebreaker. |
| **`zdump` behaviour** | Local-time *semantics* of a compiled file (UT instant, local time, UT offset, DST flag, abbreviation). Our behaviour oracle. |
| Pinned **tzdb release** (tzcode/tzdata **2026b** here) | The actual rule data; see `fixtures/MANIFEST.toml`. |

## Historical / orientation only (never implementation authority)

| Source | Why it's here, not above |
|--------|--------------------------|
| **[RFC 8536](https://datatracker.ietf.org/doc/rfc8536/)** | The *prior* TZif RFC — **obsoleted by RFC 9636**. Read for history; implement against 9636. |
| **[RFC 6557](https://www.rfc-editor.org/rfc/rfc6557.html)** / BCP 175 | Governance/process for tzdb (coordinator, mailing list, succession). Shapes our *posture* (see [tzdb-governance.md](tzdb-governance.md)), not the binary format. |
| **draft-lear-iana-timezone-database** | Predecessor context to RFC 6557. Orientation only. |
| Wikipedia (tz database, list of tz zones), OneZero / UCLA profiles, "literary appreciation", the Olson/Perl/pytz ecosystem pages | Motivation and orientation. **Never** cite these for a semantic decision. |

## Rule of thumb

> If a behaviour isn't pinned by RFC 9636, `zic(8)`/`zic.c`, or an experiment against
> reference `zic`/`zdump`, it isn't settled — don't encode it from prose. Record the pinned
> ones in [reference-zic-semantics.md](reference-zic-semantics.md).
