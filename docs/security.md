# Security model

`zic-rs` treats tzdata source as **untrusted input** and treats the output tree as a
**capability boundary**. The threat model: a malicious or corrupt source file, or a hostile
zone/link name, must not be able to read or write outside the explicit output directory,
crash the process unsafely, or cause it to consume unbounded resources.

## Memory safety

* `#![forbid(unsafe_code)]` at the crate root and the binary — there is no `unsafe` anywhere.
* `overflow-checks = true` even in release builds (correctness over micro-optimisation).
* **Panic discipline** is its own contract — see [`panic-policy.md`](panic-policy.md): no malformed/
  hostile input may panic the library (it becomes a typed `Err`); panics are reserved for violated
  *internal* invariants. T17.1 added the TZif-read bounds-guard (transition `type_index < typecnt`) at
  the `tzif::validate::parse` choke point so no consumer can index out of bounds.
* **The full failure-mode map** — how zic-rs can produce a plausible-but-wrong artifact and what guards
  or explicit non-claims bound each case — is the [`risk-register.md`](risk-register.md) (15 claim-boundary
  risks; the dangerous failures are claim-boundary bugs, not first memory bugs). *Prove the claim with
  typed evidence, or refuse it explicitly.*
* **Filesystem materialization** (path/symlink/hardlink/clobber/temp/rename/fsync policies + the exact
  crash-durability claim and the named TOCTOU residual) is the
  [`install-materialization-contract.md`](install-materialization-contract.md) (T17.4): per-file
  crash-durable publish on Unix; whole-tree crash-atomicity and the parent-component symlink-swap race are
  explicit non-claims.
* **Reports + CLI are public contracts** — [`schema-compatibility-policy.md`](schema-compatibility-policy.md)
  (when a schema bumps, public-literal ownership, typed-unknown-over-silence, old-reader fail-closed) and
  [`cli-compatibility-policy.md`](cli-compatibility-policy.md) (the 0/1/2 exit taxonomy; every command
  classified gate/diagnosis/witness/admission/convenience; *exit 0 from a report command means "it ran,"
  not "all clear"*) (T17.6).

## Input hardening (`source::lexer`)

* **NUL bytes** in a line are rejected.
* **Line length** is capped at 2048 bytes (including the newline), matching `zic`.
* Input must be **valid UTF-8** (a deliberate byte-level parser could relax this; silently
  accepting arbitrary bytes would be a foot-gun, so we don't).
* **Unterminated quotes** are a hard error, not a best-effort guess.

## Output-tree safety (`fs::output_tree`)

Zone and link names are untrusted. [`safe_relative_path`] is the single choke point that
every write passes through. It rejects, with `ZIC008_OUTPUT_PATH_TRAVERSAL`:

* absolute names (leading `/`);
* any `.` or `..` path component (no traversal);
* empty components (`a//b`);
* components beginning with `-` (avoids option-looking names; `zic -v` merely warns — we
  reject, failing closed);
* NUL bytes.

After constructing a path we additionally verify, lexically, that it is **contained** within
the output root (`is_contained`) — defence in depth in case the root itself is unusual.

## No system writes, no clobbering

* Output goes **only** under the explicit `--out` directory. There is no implicit
  `/usr/share/zoneinfo` default; a missing `--out` is a clean error.
* Writes are **atomic** (temp file in the same directory, then `rename`), so readers never
  see a half-written TZif file.
* Existing files are **never overwritten** without `--force`.
* Symlink mode creates **relative** links inside the tree; it does not follow or write
  through pre-existing symlinks (it removes a conflicting link only with `--force`).

## No external execution on the compile path

The compiler never shells out. The *only* code that runs an external process is the
`compare` oracle, which invokes reference `zic` into a caller-controlled temp directory and
is never reached during normal compilation.

## Resource bounds

A per-zone transition limit (`MAX_TRANSITIONS`, `ZIC009`) guards against pathological rule
expansion: a rule set that would generate more explicit transitions than the limit fails
closed with a clear diagnostic rather than allocating unboundedly. This is load-bearing now
that the transition compiler is live (fixed-offset zones still produce zero transitions; DST
rule sets expand across their year span). Making the limit caller-configurable
(`--transition-limit`, wiring `CompileConfig.transition_limit`) is a tracked follow-up.

**T17.1b** added a `limits::ResourceLimits` layer over the *input* dimensions reference `zic` leaves
unbounded — per-file **source bytes**, **zone** / **rule-per-set** / **link** / **leap-table** counts,
**link-chain depth**, and **continuation-eras-per-zone**. Each has a generous hard default (far above
any real tzdb), enforced once at `load_database` (counts + per-file bytes), in `resolve_link_target`
(chain depth — complementing the precise cycle check, and bounding its `visited.contains` cost), and in
`parse_leap_source` (leap count). A breach is a plain `Error::config`, **not** a `ZIC###` diagnostic
(an operational safety limit, not a grammar violation). The line-length cap (`MAX_LINE_LEN`, `ZIC017`)
and the abbreviation-table byte cap (the `u8` designation index) were already in place. See
[`panic-policy.md`](panic-policy.md) for the full reliability posture and its honest non-claims.

## On the in-house SHA-256: provenance/integrity, NOT security

`zic-rs` hashes its compiled TZif files with a small in-house SHA-256 (`src/hash.rs`,
verified against the FIPS 180-4 / NIST vectors and cross-checked against the system
`sha256sum`). That digest is used **only** to fingerprint output artifacts for *deterministic
provenance and change detection* — the `alias-map.json` and the compile manifest. It is
**not** used for, and must not be relied on for, authentication, digital signatures, message
authentication codes (MACs), password hashing, or any adversarial/security protocol. zic-rs
does not implement cryptographic security and assumes a non-malicious build pipeline; the
threat model above is about *untrusted tzdata input and the output-tree capability boundary*,
not about an attacker who controls the hash function's inputs to forge a collision. If a
security-grade digest is ever needed, depend on a vetted cryptographic crate rather than this
module. (See the matching scope note at the top of `src/hash.rs`.)
