# Coverage Maximization Report (Round 4)

Date: 2026-02-26
Branch: `agent/coverage-max-2026-02-26-r4`

## Baseline (Round 4)

Tooling combo:
- `cargo nextest run --all-features`
- `cargo llvm-cov --html`
- `cargo llvm-cov report --summary-only`
- `cargo llvm-cov report --text --show-missing-lines`

Baseline totals:

| Metric | Value |
|---|---:|
| Regions | 90.90% |
| Functions | 87.25% |
| Lines | 88.80% |

## Round 4 Targeted Changes

Added deterministic tests for remaining launch/profile branches:
- `src/launch/mod.rs`
  - extended `with_handle_list_and_stdio_inherit_record_raw_handles` to cover explicit stderr override in `with_stdio_inherit`
- `tests/windows_profile.rs`
  - replaced invalid-name assumption with deterministic SID equivalence test:
    - `profile_open_matches_derived_sid_for_name`
- `tests/windows_launch.rs`
  - `launch_with_stdio_inherit_overrides_succeeds`

No newly identified dead code was removed in this round.
No additional uncoverable comments were required for the specific branches closed in this round.

## Final Totals (Round 4)

| Metric | Baseline | Final | Delta |
|---|---:|---:|---:|
| Regions | 90.90% | 91.87% | +0.97 pp |
| Functions | 87.25% | 87.58% | +0.33 pp |
| Lines | 88.80% | 89.37% | +0.57 pp |

## Combined Progress Since Round 2 Baseline

- Prior major baseline (before round 2):
  - Regions: 85.22%
  - Functions: 84.19%
  - Lines: 82.99%
- Current (after round 4):
  - Regions: 91.87%
  - Functions: 87.58%
  - Lines: 89.37%

Net gain across rounds 2+3+4:
- Regions: +6.65 pp
- Functions: +3.39 pp
- Lines: +6.38 pp

## Remaining Gap Classification

Remaining uncovered code is still concentrated in:
1. Win32 hard-failure defensive branches requiring privileged/fault-injection scenarios.
2. Token/profile paths that depend on specific runtime token/AppContainer states not consistently reproducible in standard CI.
3. Error-path guards around API failures that are intentionally difficult to force without intrusive seam injection.

## Verification Plan

Before commit/integration for this round:
- `just ci-fast`
- `just ci-deep`

Verification outcomes:
- `just ci-fast`: PASS
- `just ci-deep`: PASS

## What This Means

Coverage improved again without weakening behavior guarantees, but 100% remains impractical under normal test environments without invasive fault-injection hooks or privileged environment orchestration.
