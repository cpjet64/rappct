# Coverage Maximization Report (Round 2)

Date: 2026-02-26
Branch: `agent/coverage-max-2026-02-26`

## Baseline

Tooling combo (Rust 2026):
- `cargo nextest run --all-features`
- `cargo llvm-cov --html`
- `cargo llvm-cov report --summary-only`
- `cargo llvm-cov report --text --show-missing-lines`

Baseline totals:

| Metric | Value |
|---|---:|
| Regions | 85.22% |
| Functions | 84.19% |
| Lines | 82.99% |

## Iterations

### Iteration A - launch/env + launch/mod helper tests

Added tests for:
- `WideBlock::as_ptr` and `WideBlock::len` behavior.
- empty-entry env block termination.
- `LaunchOptions::default` Windows shape.
- `JobObjectDropGuard::disable_kill_on_drop` and invalid assign error path.
- `InheritList` push/slice bookkeeping.
- `LaunchedIo::wait(timeout)` timeout branch using an unsignaled waitable handle.

### Iteration B - ACL directory custom grant success path

Added integration coverage for:
- `grant_to_package(ResourcePath::DirectoryCustom(...))` success flow and DACL mutation validation.

## Final Totals (Post4)

| Metric | Baseline | Final | Delta |
|---|---:|---:|---:|
| Regions | 85.22% | 88.52% | +3.30 pp |
| Functions | 84.19% | 86.53% | +2.34 pp |
| Lines | 82.99% | 86.52% | +3.53 pp |

## File-Level Highlights

- `launch/env.rs`: reached 100% in this run.
- `launch/mod.rs`: line coverage improved from 72.28% to 79.68%.
- `acl.rs`: line coverage improved from 72.28% to 85.61%.

## Dead/Uncoverable Classification Summary

Parallel dead-code investigation identified three categories:

1. Coverable and targeted this round:
- launch stdio/job helper branches and timeout branch
- ACL directory custom DACL success branch

2. Not dead, but practically uncoverable without fault injection/specialized environment:
- Win32 API hard-failure branches (`SetNamedSecurityInfoW`, `WaitForSingleObject` WAIT_FAILED, similar low-level failures)
- token-profile branches requiring tests to run under a true AppContainer token context

3. Dead code:
- No code was removed as dead in this round; uncovered regions were predominantly defensive/error paths.

## Inline Comment Additions for Uncoverable Paths

Added detailed inline rationale comments to document uncoverable error branches and alternative verification:
- `src/launch/mod.rs` (`WaitForSingleObject` WAIT_FAILED branch)
- `src/acl.rs` (`SetNamedSecurityInfoW` failure branch)

## Artifacts and Archival

- Previous report archived under `legacy/coverage/`.
- New report written to `docs/coverage-report.md`.

## Verification

Planned final verification for this round (before commit):
- `just ci-fast`
- `just ci-deep`

## What This Means

Coverage improved materially again, but remains below literal 100% because remaining gaps are mostly defensive Windows failure paths and environment-sensitive token/profile branches that are not safely reproducible in normal CI without introducing intrusive fault-injection seams.
