# Coverage Maximization Report (Round 6)

Date: 2026-02-26
Branch: coverage-max-2026-02-26-r6

## Baseline (Round 5 End)

Tooling combo:
- cargo llvm-cov nextest
- cargo nextest run --all-features
- cargo llvm-cov report --summary-only

Baseline totals:

| Metric | Value |
|---|---:|
| Regions | 98.92% |
| Functions | 100.00% |
| Lines | 98.21% |

## Round 6 Targeted Changes

Added a Windows integration test to exercise the LPAC env var fallback branch:
- tests/windows_core.rs
  - supports_lpac_unknown_override_uses_runtime_result

Added defensive comments in src/lib.rs for platform-only fail branches that remain hard to force safely in CI:
- supports_lpac() lines handling:
  - RtlGetVersion error status
  - Unsupported major version
  - Unsupported build threshold

No dead code was removed.

## Final Totals (Round 6)

| Metric | Baseline | Final | Delta |
|---|---:|---:|---:|
| Regions | 98.92% | 99.19% | +0.27 pp |
| Functions | 100.00% | 100.00% | +0.00 pp |
| Lines | 98.21% | 98.66% | +0.45 pp |

## Remaining Uncovered Lines (Filtered Coverage Set)

- src/lib.rs: 3 lines remain in defensive/platform-boundary failure branches.

## Verification Outcomes

- just ci-fast: PASS
- just ci-deep: PASS

## What This Means

The gating threshold in just coverage is currently Regions >= 95% and is satisfied.
Remaining misses are intentionally defensive/uncoverable in normal CI environments.
