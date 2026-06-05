# Generated-data contract

What a consumer (a Rust crate, a test fixture, an embedded bundle) may rely on when it
ingests output produced by zic-rs. This is the bridge between the compiler and its downstream
users; it states guarantees and — honestly — current limits.

## Guarantees (today)

For every zone zic-rs compiles (it **fails closed** rather than emit anything it cannot
produce correctly):

1. **Valid TZif per [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636).** The full layout —
   v1 (32-bit) block + v2+ (64-bit) block + footer, big-endian, with header counts that match
   the data. We round-trip our own output through a reader (`tzif::validate`) as a self-check.
2. **Semantic match against reference `zic`/`zdump`** over a **declared year horizon** (the
   `compare --mode zdump --horizon LO,HI` oracle): same UT instants, UT offsets, DST flags,
   and abbreviations within the window. Behaviour outside the declared window is *not*
   asserted. (Byte-parity is additionally pinned only for fixed-offset fixtures with a
   checked-in reference blob.)
3. **Deterministic output.** The same source + selection yields identical bytes; nothing
   depends on wall-clock time or unordered iteration reaching the output.
4. **Safe layout.** Output lands strictly under the explicit `--out` root; zone/link names
   are validated against traversal (`ZIC008`); writes are atomic and never clobber without
   `--force`. See [security.md](security.md).

## Provenance (shipped — T3.4b/c)

A consumer that needs traceability can request, alongside the tree:

* an **alias/canonical manifest** — `compile --alias-map <path>` writes `alias-map.json`
  (schema `zic-rs-alias-map-v1`): which identifiers are canonical zones vs links, link
  targets, per-file SHA-256, and a summary with `identifiers`/`canonical_zones`/`links` and
  **`duplicated_byte_links`** (links materialised as byte copies — answers jiff#258's
  alias-duplication accounting);
* a **compile-provenance manifest** — `compile --manifest <path>` writes `zic-rs-manifest.json`
  (schema `zic-rs-compile-manifest-v8`): `zic_rs_version`; a `tzdb` block (detected-vs-claimed
  version + `version_status`, source path + SHA-256); a `source_inputs` block (structural `kind`
  + ordered per-file list with hashes + order-sensitive `aggregate_hash`, T12.3); a `build_profile`
  block (`emit_style`/`range`/`redundant_until`/`link_mode`/`output_tree`/`leap_source` — as of T12.5d
  it carries **no** source-variant placeholders); a `link_profile` block (link counts + `alias_map_sha256`
  + selected/omitted link hashes, T12.4b); a `source_profile` block with `backward_evidence` (T12.4d) +
  `backzone_evidence` (T12.5b, hash-anchored to the pinned reference release) +
  `packratlist_evidence` (T12.5c, backzone *scope* from an admitted generation-policy input — never
  from compile `source_inputs`) + `dataform_evidence` (T12.5d, *encoding* form hash-matched against the
  pinned 2026b `.zi` artifacts, + `recipe_hash`/`generated_from`) axes — detected/claimed/status,
  hash-backed-or-claim-only, never inferred; the zones/links touched; and an `oracle` block.

**The manifest describes *this* invocation, not the repo's test status.** A bare `compile`
does not run the oracle, so its `oracle` block is `{"mode":"not-run","result":"not-run"}` —
it never infers a `match` from the test suite. tzdb version detection and an IANA release URL
are still future work (recorded as `unknown` today); per-fixture pins also live in
`fixtures/MANIFEST.toml`.

## Limits (stated plainly, no overclaiming)

* The **declared subset** only: unsupported constructs (inline-save eras with a `%s`/slash
  `FORMAT`, `24:00`/negative compiled times, recurring rules whose `ON` is a *fixed numeric* day
  (no weekday), and leap seconds) **fail closed** — they are never silently
  approximated. See [unsupported-syntax.md](unsupported-syntax.md). (Compatibility/breadth
  features now supported: `FROM = minimum`→1900, inline-save literal/`%z`, **negative inline SAVE
  (law 7)**, **`Sun<=N`/`Sat<=N` recurring `ON` (law 10)**, and genuinely
  mixed-in-era finite+recurring final eras — see [supported-syntax.md](supported-syntax.md).)
* **The footer is not prophetic** — it projects current rules; it is not a prediction of
  future civil-time law. See [tzif-notes.md](tzif-notes.md).
* zic-rs is a **producer/build tool, not a datetime library**. To *read* local time, use a
  consumer (tz-rs/jiff); see [rust-ecosystem.md](rust-ecosystem.md).

## Intended consumers

`tz-rs` (reads TZif), TZif reader crates, embedded bundles, and test fixtures. Optional
ecosystem smoke tests (roadmap T3.4d, behind a feature flag) will demonstrate that a real
Rust reader loads zic-rs-generated TZif.
