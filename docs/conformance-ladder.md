# The conformance ladder (how zic-rs admits IANA slices)

> Reference standard (Arthur David Olson / Paul Eggert / Kenneth Murchison, strict): IANA slices are
> **not trophies**. They are a deliberately chosen **conformance ladder**. zic-rs admits a slice
> only when it forces or validates a *distinct* reference-`zic` semantic, proves behaviour under
> `zdump`, records provenance, and expands the support frontier **honestly**.

zic-rs is a producer-side compiler for one of the world's strangest living civic datasets. The
tz database is collaborative civil-time infrastructure: it records historical timezone and
daylight-saving rules and is updated as governments change civil-time law (the maintenance
process is governance, per RFC 6557 / BCP 175 — *not* local preference). Our job is to **compile**
it, never to correct, simplify, or editorialize it.

## Authority stack (unchanged, restated)

```
IANA tzdb source  →  reference zic / zdump  →  RFC 9636 (TZif validity)  →  zic-rs + evidence
```

Wikipedia and articles are orientation only. Every subtle behaviour is pinned by experiment
against reference `zic`/`zdump`, never inferred from prose (see
[reference-zic-semantics.md](reference-zic-semantics.md)).

## The five rules (the "make-them-proud" standard)

1. **Every slice has a reason.** A slice is admitted because it forces a new semantic class or
   validates an important implemented one — never because it is famous.
2. **Every slice has a receipt.** Provenance, the transform (if any), reference-`zic` output,
   `zdump` parity over a declared horizon, type/transition counts, footer, alias/link status,
   consumer-bench status (if run), and any caveats. Receipts live in
   [`fixtures/iana-slices/SLICES.toml`](../fixtures/iana-slices/SLICES.toml); the *live*
   enforcement is the per-slice `zdump` oracle tests plus `support-report`.
3. **Behaviour first, structure second.** `zdump` behaviour over a declared horizon is the
   binding contract. TZif structural parity (slim/fat transition counts) is optional unless
   explicitly claimed — the `Test/Mixed` slim/fat result is the pinned doctrine
   ([reference-zic-semantics.md](reference-zic-semantics.md) §6).
4. **Do not editorialize civil time.** We compile tzdb as reference `zic` does. We never "fix"
   or normalize it ([tzdb-governance.md](tzdb-governance.md)).
5. **Links are first-class production identifiers.** Production systems use aliases, not only
   canonical zones, and a current-offset table does not describe a zone's historical data. The
   alias map and `support-report` therefore account canonical zones, links, and total
   identifiers **separately** — never conflated.

## The admission gate

A slice graduates from "pressure probe" to "admitted" only when **all** of these exist:

* **provenance** — tzdb release + source kind, and (for de-abbreviated slices) a byte-identity
  proof against the verbatim extract under reference `zic`;
* **reference-`zic` output** to diff against;
* **`zdump` parity** over a declared horizon (the contract);
* **`explain`** output (the evidence trace);
* **manifest** output (`--alias-map` / `--manifest` provenance);
* a **documented semantic reason** (which class it forces/validates).

Pressure-probe first (extract the slice, run zic-rs, record the *first* fail-closed blocker,
classify it as parser vs semantic vs emission-shape), implement only what the real source forces,
and call it admitted only once the oracle is green. (This is exactly how `Europe/London` and
`America/New_York` were admitted — see `research/SESSION-CONTEXT.md`.)

## The ladder

| Tier | Slice | Semantic class it forces / validates | Status |
|-----:|-------|----------------------------------------|--------|
| 0 | `Europe/London` | multi-era, LMT, GMT/BST/BDST, non-zero fixed-offset era, recurring EU footer, effective-in-era classification, direct `tzdata.zi` ingestion | ✅ admitted (T4.0) |
| 1 | `America/New_York` | U.S. rule history, WWII war time, **genuinely mixed-in-era** finite+recurring final era | ✅ admitted (T4.1) |
| 2 | `Asia/Hong_Kong` | real inline-save / wartime behaviour (proves T3.2b on a real zone) | ▷ next candidate |
| 3 | `Asia/Kolkata` (+ `Asia/Calcutta` alias) | non-integer offset, historical naming / alias↔canonical | ▷ |
| 4 | `Australia/Sydney` | southern-hemisphere seasonal inversion (mental-model discipline) | ▷ |
| 5 | `Pacific/Apia` | date-line / skipped-day civil-time discontinuity ("final boss") | ▷ later |
| 6 | `Etc/UTC`, `Etc/GMT±N`, selected `Link` aliases | fixed/simple zones, reversed POSIX-sign `Etc/GMT` names, link/canonical integrity | ▷ |

**Slice quantity is not progress.** "5 zones covering 8 semantic families" beats "10 zones." Pick
the next slice by *bucket impact* (see `support-report`), not by fame.

## `support-report` — the frontier map

`zic-rs support-report --input <file>` compiles **every** zone in a source file and buckets the
outcome, distinguishing canonical zones from links. It reports **compile** support (a valid TZif
is produced) — *not* behavioural correctness, which only the `zic`/`zdump` oracle establishes.
Every zone lands in exactly one bucket and the accounting is exact (`supported + Σ unsupported ==
zones parsed`), with a catch-all `other` bucket that keeps the raw diagnostic — nothing is hidden.

This is how the next milestone is chosen: by the **largest unsupported bucket**, not by vibes.
Running it on the installed `tzdata.zi` (2026b) at the time of writing reports **162 / 341**
canonical zones compile-supported (345 / 598 identifiers including links), and named the biggest
unlock precisely: **176 zones** blocked by a no-rules era with a **`%z` FORMAT** (e.g.
`0:30 - %z`, which `zic` renders as the numeric offset of the era's standard offset). Implementing
that single small fix took compile-clean coverage to **338 / 341** — a result no amount of guessing
would have produced. That is the point of the ladder: let the real database choose the work.

## Three statuses — `compile-clean` is not `behaviour-verified`

The headline number is honest only if its meaning is precise. Three distinct statuses:

* **compile-clean** — `support-report` says zic-rs emits a valid TZif for the zone. (338 / 341.)
* **behaviour-verified** — the emitted TZif matches reference `zic` under `zdump` over a declared
  horizon. This is the binding contract, and it is **strictly stronger** than compile-clean.
* **fail-closed** — the zone hits an explicit unsupported diagnostic. **None** among canonical zones
  in 2026b (the negative-SAVE and non-POSIX-day-form buckets are both cleared, laws 7 + 10).

A **comprehensive** oracle sweep — **all 341** canonical zones over `1900..2040`, each compiled and
`zdump`-diffed against reference `zic` — reports **341 match / 0 mismatch / 0 fail-closed**
(341 + 0 = 341 compile-clean), *after T5 #1–#5 + law 7 + law 10*: **every canonical zone
behaviour-matches reference `zic` over `1900..2040`** — the canonical-zone behaviour frontier is
closed. The match count rose **264 → 328 → 333 → 338 → 339 → 341** (the old 35-zone *sample* "28 / 6",
which caught only 6 of 74, is retired): **T5 #1–#5** (standard-`LETTER`; final-recurring anchor;
standard-clock absorption −64; prior-active rule-state seeding −5; clock-reference normalization −5),
**law 7** (inline negative SAVE −1: Prague), **law 10** (non-POSIX `ON` + v3 footer −2: Gaza/Hebron).
Catalogue in [reference-zic-semantics.md](reference-zic-semantics.md) §10.

So the honest sentence is: **"zic-rs compiles all 341 canonical zones from `tzdata.zi` 2026b and
behaviour-matches reference `zic` under `zdump` over 1900..2040 for all 341; none fail closed."** —
*over `1900..2040`, a declared conformance sweep, not
infinity; not full-`zic` replacement.* The next milestone is a **`support-report --verify`** mode
that folds this comprehensive sweep into the tool. Only zones that are **behaviour-verified +
receipted** are admitted to
[compatibility.md](compatibility.md).
