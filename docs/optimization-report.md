# Optimization Report

Created: `2026-02-26T02:02:06Z`

| Step | Summary | Metrics |
|---|---|---|

---

## Initialization and Plan

Timestamp: `2026-02-26T02:02:06Z`

| Step | Summary | Metrics |
|---|---|---|
| Scope | Full repo (Rust, Windows-focused crate) | branch `perf-opt-1772071285` |
| Baseline targets | `merge_parent_env`, `make_wide_block`, SID parse candidate | pending benchmark |
| Strategy | One-at-a-time optimization + re-benchmark + full verification | stop when gains <0.5% for two passes |

---

## Optimization Loop Results

Timestamp: `2026-02-26T02:13:39Z`

| Step | Summary | Metrics |
|---|---|---|
| Baseline probe | Release perf probe run on hot paths | `make_wide_block=174659.38 ns/iter`, `merge_parent_env=93100.29 ns/iter` |
| Opt #1 | Reworked `make_wide_block` to sort references + cached fold key + direct UTF-16 writes | `60271.85 ns/iter` (improved from baseline) |
| Opt #2 (rejected) | HashSet/lowercased key index in `merge_parent_env` | Regression: `119503.04 ns/iter` (worse) |
| Opt #2 (accepted) | Lightweight `merge_parent_env` update: reserve + single env lookup | `88419.58 ns/iter` |
| Validation | Full gates with vcvars bootstrap | `just ci-fast` pass, `just ci-deep` pass |

## Per-Finding Summary

### Finding P1 - `make_wide_block` allocation/sort overhead
- File: `src/launch/env.rs`
- Changes:
  - avoid cloning `entries` by sorting references (`Vec<&(OsString, OsString)>`)
  - use `sort_by_cached_key` to avoid repeated lowercase key computation during comparisons
  - remove per-entry temporary UTF-16 line vector and append directly to output buffer
- Baseline: `174659.38 ns/iter`
- Best measured: `60271.85 ns/iter`
- Improvement: **65.49% faster**

### Finding P2 - `merge_parent_env` repeated work
- File: `src/launch/mod.rs`
- Attempt A (rejected): case-folded HashSet cache; improved asymptotics but extra normalization allocations dominated runtime.
- Attempt B (accepted): reserve capacity + remove duplicate `var_os` lookup while preserving existing semantics.
- Baseline: `93100.29 ns/iter`
- Best measured: `88419.58 ns/iter`
- Improvement: **5.03% faster**

## Combination Check
- Combined accepted changes were validated with full suite (`ci-fast`, `ci-deep`) and retained performance wins.

## Final Optimization Summary
- Runtime impact on measured hot paths:
  - `make_wide_block`: **-65.49%** time per iteration.
  - `merge_parent_env`: **-5.03%** time per iteration.
  - Arithmetic mean reduction across the two targeted probes: **-44.47%**.
- Correctness/regression:
  - No test regressions.
  - Lints, format, coverage gates, security checks, and docs generation all passed.

## Notes
- Performance probes were executed during the optimization loop in release mode and removed before final commit to avoid introducing non-production test overhead into CI coverage accounting.
