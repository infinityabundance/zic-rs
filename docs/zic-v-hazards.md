# `zic -v` compatibility hazards tracked by zic-rs

`zic`'s verbose mode (`zic -v`) warns about constructs that **older readers or older `zic` versions
mishandle**. That warning list is institutional memory — a map of the places a timezone compiler can
silently go wrong. This table turns it into an audit surface: each hazard, its zic-rs status, the test
or fixture that guards it, the `support-report` bucket it lands in (if unsupported), and the future
campaign that would close it.

Status legend: **✅ supported** (behaviour-verified) · **▷ fail-closed** (explicit diagnostic, no
approximate output) · **◇ future campaign** (named, not a hidden claim). Each row links a law in
[zic-deep-semantics.md](zic-deep-semantics.md).

| `zic -v` hazard | zic-rs status | Test / fixture | `support-report` bucket | Future campaign |
|-----------------|---------------|----------------|-------------------------|-----------------|
| **`%z` in FORMAT** (pre-2015 `zic` lacked it) | ✅ supported (no-rules era, inline save, ruled) — law 9 | `inline_save_percent_z_uses_effective_offset` (`Test/InlineZ`) | — | — |
| **`STD/DST` slash FORMAT** | ✅ on ruled eras (London `GMT/BST`); ▷ inline-save slash fails closed — law 9 | London slice oracle | `no-rules era: %s or STD/DST slash FORMAT` (if no rules) | inline-save `%s`/slash |
| **`24:00` and `>24:00` AT** | ▷ parsed; compile path fails closed — law 11 | (parser tests) | — | extended-AT + content-driven v3 |
| **negative AT (`-2:30`)** | ▷ parsed; compile path fails closed — law 11 | (parser tests) | — | extended-AT |
| **fractional seconds** (round ties-to-even) | ▷ rounding rule pinned; compile path deferred — law 11 | (parser tests) | — | extended-AT |
| **`ON` rules crossing month boundaries** (`Sun>=31`, `Sat<=30`) | ✅ supported — law 10 (re-anchored onto a clean nth-weekday + extended-time **v3** footer, e.g. Gaza `Sat<=30` → `M3.4.4/50`) | `neighboring_month_on_form_matches_reference_zic`, `recurring_sat_leq_30_uses_v3_extended_time_footer` | — (cleared) | — (a fixed *numeric* `ON` day still fails closed) |
| **negative SAVE** (Ireland-style) | ✅ supported — law 7 (signed effective offset; Prague `1 -1 GMT` → GMT, isdst=1) | `negative_inline_save_is_signed_state_matches_reference_zic` | — (cleared) | — |
| **wall/standard/universal AT vs UNTIL mixing** | ✅ normalized to resolved instants — law 3 | `wall_standard_universal_refs_are_normalized` (residual-five) | — | — |
| **future not representable by a proleptic TZ string** | ✅ exact-or-fail footer (refuse rather than approximate) — law 13 | recurring-footer tests | (refused → ZIC diagnostic) | — |
| **too many transitions** (`ZIC009`) | ✅ enforced `MAX_TRANSITIONS` | transition-limit tests | `ZIC009` | wire `--transition-limit` |
| **link-to-link** (mishandled by `zic` ≤ 2022e and old parsers) | ✅ chain resolution + cycle/missing diagnostics — law 15 | link tests; `tests/support_report.rs` | link cycle/missing | — |
| **abbreviation length / `<3` chars** (`zdump` warns) | ✅ emitted faithfully (matches reference; `zdump` warns on both) | sweep (`Asia/Macau` "CT", `Atlantic/Bermuda` "AT") | — | — |
| **unsafe / non-portable file names** | ✅ path-traversal-proof output tree (`ZIC008`) | `tests/output_safety.rs` | — | — |
| **leap-second table truncation / `Leap` lines** | ◇ leap modes out of scope; `Leap`/`Expires` rejected in zone-source mode — law 16 | (zone-source dispatch) | — | leap-file mode |

## Notes

* **Behaviour vs structure.** Slim/fat transition-count differences (`-b slim`/`fat`, `-R`) are
  **not** hazards — they are structural and `zdump`-equivalent (law 14). Byte parity is claimed only
  where a reference blob is pinned.
* **Honesty.** A hazard marked ✅ means *behaviour-verified against reference `zic`/`zdump` over a
  declared horizon* for the zones that exercise it — not "fully `zic`-compatible." The three
  fail-closed buckets above are the entire canonical-zone behaviour frontier for `tzdata.zi` 2026b
  (see [unsupported-syntax.md](unsupported-syntax.md)).
