# tzdb governance — and why zic-rs is humble about it

The IANA Time Zone Database (tzdb) is **not** a clean mathematical dataset. It is a record of
civil-time *law*: governments set time zones and daylight-saving rules and change them, often
with little notice and for political reasons. tzdb records and distributes those decisions; it
does not invent them. (See IANA's [tz-link](https://data.iana.org/time-zones/tz-link.html):
the data "are by no means authoritative" in the sense of dictating law — they track what
governments do.)

## What zic-rs is — and is not

**zic-rs is a compiler for IANA tzdata. It is not a timezone-policy authority.** It does not
decide civil-time truth, curate timezone history, or "correct"/"normalise"/"improve" the
data. The only legitimate claim it makes is:

> for a *pinned* tzdb release and a *declared* syntax subset, zic-rs's output matches
> reference `zic`/`zdump` behaviour (with evidence).

Any wording that sounds like zic-rs owns or improves the dataset is a bug in the docs.

## The authority chain (highest to lowest)

1. **IANA tzdb release** — the source of record for civil-time rules, maintained through the
   IANA process described in [RFC 6557](https://www.rfc-editor.org/rfc/rfc6557.html) / BCP 175
   (volunteer coordinator + the `tz@iana.org` mailing list + a documented update and
   succession procedure). zic-rs pins a release; it never edits the data.
2. **Reference `zic` / `zdump` behaviour** — the canonical compiler and dumper. When the
   manpage is ambiguous, *their observed behaviour* is the tiebreaker, pinned by experiment.
3. **[RFC 9636](https://www.rfc-editor.org/rfc/rfc9636)** — the normative TZif binary format
   (obsoletes RFC 8536). Governs the on-disk layout we emit and validate.
4. **zic-rs implementation + oracle evidence** — the bottom of the chain. We change *our*
   behaviour to match the layers above, never the reverse.

## Consequences for the code

* **No editorialising.** We do not drop, merge, reorder, or "tidy" zones/rules beyond what
  `zic` does. Representational choices (slim output, type ordering, transition horizon) follow
  `zic`'s observable behaviour, validated by the `zdump` oracle.
* **Pin the release.** Fixtures and byte-parity blobs record the exact tzdb/tzcode version
  (`fixtures/MANIFEST.toml`). A future release-provenance manifest (roadmap T3.4) will make
  every compiled output traceable to its source release.
* **The future is best-effort.** The POSIX footer projects *current* rules forward; it is not
  a prediction of future law (see [tzif-notes.md](tzif-notes.md)).

See also [standards.md](standards.md) (normative vs historical sources),
[legal-history.md](legal-history.md) (public-domain posture), and
[reference-zic-semantics.md](reference-zic-semantics.md) (pinned behaviours).
