# Coverage Maximization Report (Round 5)

Date: 2026-02-26
Branch: `agent/coverage-max-2026-02-26-r5`

## Baseline (Round 5)

Tooling combo:
- `cargo nextest run --all-features`
- `cargo llvm-cov --html`
- `cargo llvm-cov report --summary-only`
- `cargo llvm-cov report --text --show-missing-lines`

Baseline totals:

| Metric | Value |
|---|---:|
| Regions | 91.87% |
| Functions | 87.58% |
| Lines | 89.37% |

## Round 5 Targeted Changes

Added deterministic unit tests for previously uncovered launch helper branches:
- `src/launch/mod.rs`
  - `inflate_security_caps_prefers_override_when_provided`
  - `duplicate_additional_handles_ignores_null_entries`

No dead-code removals were identified in this round.
No additional uncoverable-path comments were required for the branches closed in this round.

## Final Totals (Round 5)

| Metric | Baseline | Final | Delta |
|---|---:|---:|---:|
| Regions | 91.87% | 91.95% | +0.08 pp |
| Functions | 87.58% | 87.67% | +0.09 pp |
| Lines | 89.37% | 89.46% | +0.09 pp |

## Combined Progress Since Round 2 Baseline

- Prior major baseline (before round 2):
  - Regions: 85.22%
  - Functions: 84.19%
  - Lines: 82.99%
- Current (after round 5):
  - Regions: 91.95%
  - Functions: 87.67%
  - Lines: 89.46%

Net gain across rounds 2+3+4+5:
- Regions: +6.73 pp
- Functions: +3.48 pp
- Lines: +6.47 pp

## Remaining Gap Classification

Remaining uncovered code is still concentrated in:
1. Win32 hard-failure defensive branches requiring privileged or synthetic fault-injection scenarios.
2. Token/profile paths that require runtime states not consistently reproducible in standard CI.
3. Error-path branches around OS API failure modes that are intentionally difficult to force safely.

## Verification Plan

Before commit/integration for this round:
- `just ci-fast`
- `just ci-deep`

Verification outcomes:
- `just ci-fast`: PASS
- `just ci-deep`: PASS

## What This Means

Coverage continues to increase incrementally with deterministic tests and no behavior regressions. 100% remains impractical without intrusive fault-injection seams and specialized runtime orchestration.
