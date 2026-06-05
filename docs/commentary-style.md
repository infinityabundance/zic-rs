# Commentary style — literate *inspiration*, not literate tooling

zic-rs draws on the spirit of [literate programming](https://en.wikipedia.org/wiki/Literate_programming)
— code should *explain itself to a human* — but it is **not** a literate-programming project.
There is no tangle/weave step and no documentation-tooling dependency. The code is normal,
readable Rust.

## How we adopt the spirit

* **Code stays ordinary Rust modules.** No web/tangle, no embedded macro DSL. A reader opens
  `src/...` and reads plain Rust.
* **Comment intent and edge cases, not syntax.** Explain the *why* and the tzdata/TZif
  semantics — wall/standard/UT conversion, UNTIL-in-ending-era context, the slim v1 stub,
  count math, path-safety, fail-closed policy, final-recurring-era footer anchoring — never
  narrate obvious Rust. (Standing standard #1.)
* **Docs carry the prose.** Compiler phases live in [design.md](design.md); the binary format
  in [tzif-notes.md](tzif-notes.md); the standards split in [standards.md](standards.md).
* **Hard semantics get *named tests* + a ledger.** The court of record for subtle reference
  behaviour is [reference-zic-semantics.md](reference-zic-semantics.md), paired with
  descriptively-named regression tests (e.g. `until_wall_time_uses_prevailing_save_when_dst_active`,
  `final_continuation_from_2015_projects_from_era_start`). A reviewer should see *which* zic
  subtlety each test guards from its name alone.

The goal is a repo a future maintainer (or our future selves) can re-understand cold —
achieved through explanatory comments, phase docs, and an evidence ledger, not through
literate-programming machinery.
