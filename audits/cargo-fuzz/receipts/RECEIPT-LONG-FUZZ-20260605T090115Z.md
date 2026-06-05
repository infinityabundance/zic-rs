# RECEIPT — LONG-FUZZ burn-in run — 2026-06-05T09:01:15Z

> **Overall: `clean`.** Vocabulary: clean | crash_found | inconclusive_environment | interrupted | not_run.
> **This run does NOT claim coverage saturation.** A `clean` result means only that no crash was found
> within the stated target / time / seed configuration below. Produced by `fuzz/run-long-burnin.sh`.

## Provenance (recorded by the runner)

```text
command_line:        fuzz/run-long-burnin.sh 
mode:                manual (per-target-minutes=5)
per_target_duration: 300s  (9 targets)
start_time:          2026-06-05T09:01:15Z
end_time:            2026-06-05T09:46:49Z
rustc:               rustc 1.98.0-nightly (31a9463c6 2026-05-25)
cargo_fuzz:          cargo-fuzz 0.13.1
libfuzzer:           bundled with rustc sanitizer runtime (rustc 1.98.0-nightly (31a9463c6 2026-05-25))
host_os:             Linux 7.0.9-1-cachyos x86_64
host_kernel:         7.0.9-1-cachyos
host_cpu:            AMD Ryzen 7 9800X3D 8-Core Processor (16 cpus)
crash_artifact_dir:  fuzz/artifacts/<target>/   (crash-*/oom-*/timeout-* land here)
target_list:         tzif_validate_bytes vendor_oracle_json manifest_json source_lexer zone_rule_link_parser posix_footer aux_table_validator release_diff_tree path_materialization_model
```

## Per-target result

| target | verdict | seeds | seed_corpus_hash | duration | new_crashes | exit |
|---|---|--:|---|--:|--:|--:|
| tzif_validate_bytes | clean | 935 | `86e258ce2515ea42…` | 300s | 0 | 0 |
| vendor_oracle_json | clean | 5561 | `6b81260241530921…` | 300s | 0 | 0 |
| manifest_json | clean | 0 | `e3b0c44298fc1c14…` | 300s | 0 | 0 |
| source_lexer | clean | 3094 | `2faadcac36ebf54b…` | 300s | 0 | 0 |
| zone_rule_link_parser | clean | 5021 | `9b97a36be3888aa8…` | 300s | 0 | 0 |
| posix_footer | clean | 467 | `0a7b85d597ece78b…` | 300s | 0 | 0 |
| aux_table_validator | clean | 3082 | `847e39e829c73f85…` | 300s | 0 | 0 |
| release_diff_tree | clean | 3922 | `e7bc6882399251c1…` | 300s | 0 | 0 |
| path_materialization_model | clean | 797 | `b402836289f94bba…` | 300s | 0 | 0 |

## Non-claims

- **Not a saturation campaign.** Coverage is bounded by the per-target time above; un-run code paths are not exercised.
- A `clean` verdict is evidence of no crash in *this* configuration, **not** a proof of crash-freedom.
- `inconclusive_environment` = a target's libFuzzer process exited non-zero with no new artifact (build/host issue), **not** a zic-rs finding.
- A real 24h-class burn-in (this harness, large `--campaign-hours`) on a dedicated host is the future operator/lab step.

## On a crash

Minimize the artifact (`cargo +nightly fuzz tmin <target> <artifact>`), add it as a regression test in
the **main** crate (`tests/fuzz_regressions.rs`), land the fix with that test, and record both here.
