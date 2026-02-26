# Coverage Maximization Report

Date: 2026-02-26
Branch: coverage-max-20260225-200509

## Baseline

Toolchain combo used (Rust 2026 recommended):
- `cargo nextest run --all-features`
- `cargo llvm-cov --html`

Baseline totals (`cargo llvm-cov report --summary-only`):

| Metric | Value |
|---|---:|
| Regions | 80.34% |
| Functions | 77.49% |
| Lines | 77.41% |

Top low-coverage modules:

| File | Region % | Line % |
|---|---:|---:|
| `src/launch/mod.rs` | 63.24% | 65.08% |
| `src/ffi/handles.rs` | 65.06% | 58.18% |
| `src/ffi/mem.rs` | 68.29% | 60.00% |
| `src/token.rs` | 74.00% | 70.99% |
| `src/acl.rs` | 75.72% | 66.92% |

Notes:
- Uncovered output is dominated by Windows-only defensive/error branches, optional execution modes, and legacy-compat utility wrappers.
- Full uncovered line inventory was captured via `cargo llvm-cov report --text --show-missing-lines` for classification in next phase.

## Iteration 1 Plan

1. Classify uncovered paths into dead/uncoverable/testable.
2. Add targeted tests for coverable branches with highest line impact and lowest risk.
3. Re-run full coverage and compare deltas.
4. Add detailed comments for residual uncoverable paths if still uncovered.

## Iteration 1 Execution

Added targeted tests in:
- `src/acl.rs`
- `src/ffi/handles.rs`
- `src/ffi/mem.rs`
- `src/util.rs`
- `src/launch/mod.rs`
- `src/token.rs`

Coverage-relevant additions:
- ACL capability-SID invalid input path and full-root registry parser path.
- FFI handle validation/duplication branches.
- LocalAlloc/CoTaskMem ownership and null/no-op branches.
- Deprecated utility guard conversion and ownership-release branches.
- Launch helper branches (`make_cmd_args`, `with_env_merge`, handle list/stdin-out inheritance recording).
- Token consistency shape check on real current-process query.

## Coverage Delta

| Metric | Baseline | After Iteration 1 | Delta |
|---|---:|---:|---:|
| Regions | 80.34% | 85.24% | +4.90% |
| Functions | 77.49% | 84.19% | +6.70% |
| Lines | 77.41% | 83.02% | +5.61% |

Largest improvements (line coverage):
- `src/ffi/handles.rs`: 58.18% -> 96.34%
- `src/ffi/mem.rs`: 60.00% -> 94.79%
- `src/util.rs`: 75.63% -> 92.31%
- `src/launch/mod.rs`: 65.08% -> 72.23%

## Dead-Code Investigation Summary

- Proven dead code removed: none.
- Reason: uncovered areas are predominantly defensive/error handling, platform/runtime-condition branches, and privileged/environment-dependent paths rather than unreachable artifacts.

## Residual Uncovered Classification

1. Uncoverable in standard CI/runtime:
- Windows API hard-failure branches requiring allocator/syscall failure injection.
- Privileged or global-state branches (firewall/loopback mutation and similar side effects).
- Non-Windows fallback branches in a Windows-run coverage session.

2. Coverable with higher-cost harnessing:
- Additional launch failure-mode branches requiring controlled OS fault injection or mocked Win32 boundaries.
- More registry/filesystem ACL failure branches requiring synthetic permission environments.

3. Not removed:
- Legacy/deprecated utility wrappers are still part of public compatibility surface and therefore not treated as dead.

## Notes on 100% Goal

- Full 100% line/region coverage is not currently achievable without either:
  - intrusive fault-injection seams across Win32 boundaries, or
  - relaxing safety constraints to force pathological OS failures.
- This iteration maximized low-risk, high-confidence coverage gains while preserving behavior and security posture.

## Verification

- `just ci-fast` passed.
- `just ci-deep` passed.

## What This Means

- Core behavior and most compatibility wrappers now have much stronger executable proof.
- Remaining uncovered logic is concentrated in platform/error contingencies that require specialized fault-injection or privileged harnesses.
- The current state is a practical maximum for low-risk unit/integration testing without altering production safety boundaries.
