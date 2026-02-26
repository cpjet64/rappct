# Coverage Maximization Report (Round 3)

Date: 2026-02-26
Branch: `agent/coverage-max-2026-02-26-r3b`

## Baseline (Round 3)

Tooling combo:
- `cargo nextest run --all-features`
- `cargo llvm-cov --html`
- `cargo llvm-cov report --summary-only`
- `cargo llvm-cov report --text --show-missing-lines`

Baseline totals:

| Metric | Value |
|---|---:|
| Regions | 88.52% |
| Functions | 86.53% |
| Lines | 86.51% |

## Round 3 Targeted Changes

Added tests focused on remaining coverable launch and ACL paths:
- `src/launch/mod.rs`
  - `with_security_capabilities_sets_internal_override`
  - extended `JobObjectDropGuard` coverage via `as_handle` assertion
- `tests/windows_launch.rs`
  - `launch_with_null_stdio_has_no_parent_streams`
  - `launch_with_explicit_handle_list_succeeds`
- `tests/windows_acl.rs`
  - `grant_to_package_updates_directory_default_inheritance_dacl`

## Final Totals (Round 3 Post2)

| Metric | Baseline | Final | Delta |
|---|---:|---:|---:|
| Regions | 88.52% | 90.90% | +2.38 pp |
| Functions | 86.53% | 87.25% | +0.72 pp |
| Lines | 86.51% | 88.80% | +2.29 pp |

## Combined Progress Since Round 2 Baseline

- Prior major baseline (before round 2):
  - Regions: 85.22%
  - Functions: 84.19%
  - Lines: 82.99%
- Current (after round 3):
  - Regions: 90.90%
  - Functions: 87.25%
  - Lines: 88.80%

Net gain across rounds 2+3:
- Regions: +5.68 pp
- Functions: +3.06 pp
- Lines: +5.81 pp

## Remaining Gap Classification

Remaining uncovered code is still concentrated in:
1. Win32 hard-failure defensive branches requiring privileged/fault-injection scenarios.
2. Token/profile paths that depend on running tests under specific AppContainer/token states.
3. Non-critical internal error branches guarded by API failures not safely reproducible in standard CI.

No proven dead code was removed in this round.

## Verification Plan

Before commit/integration for this round:
- `just ci-fast`
- `just ci-deep`

## What This Means

Coverage continues to increase with behavior-preserving tests, but 100% is still not practically reachable without introducing intrusive fault-injection seams or running in specialized privileged/runtime environments.
