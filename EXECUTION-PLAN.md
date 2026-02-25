# rappct EXECUTION-PLAN.md
**Single Source of Truth** — Created 2026-02-24  
**All other plans, TODO.md, TODO_AUTOPILOT.md, STUBS.md, WORKLOG.md, and scattered .md files are now DEPRECATED.**  
Move them to `legacy/` after you create this file.

## Governance Rules (never violate)
1. Read README.md, CONTRIBUTING.md, AGENTS.md, and docs/ADR-0001 first.
2. Windows-first: all new code must be `#[cfg(windows)]` where appropriate.
3. Run full quality gates before every commit.
4. Prefer RAII for FFI handles, SIDs, and memory.
5. Add tests alongside features.
6. Keep documentation and CHANGELOG.md in sync.

## Four Target Milestones
See MASTER-CHECKLIST.md for the detailed items under each milestone.

### Milestone 1 – First Functional Library (Core AppContainer Features)
### Milestone 2 – Full FFI Safety & Ownership
### Milestone 3 – Initial MVP (All Features + Examples + Use Case Groupings)
### Milestone 4 – Finished Project

## Step-by-Step Execution Order

**Phase 0 – Stabilize & Bootstrap**
1. Ensure all quality gates pass.
2. Verify basic launch works.
3. Commit.

**Phase 1 – Milestone 1**
1. Complete core APIs.
2. Make basic examples run.
3. Commit.

**Phase 2 – Milestone 2**
1. Complete FFI refactoring (ADR-0001).
2. Commit.

**Phase 3 – Milestone 3**
1. Implement Standard Use Case Groupings (see MASTER-CHECKLIST.md).
2. Complete all examples and CLI.
3. Commit.

**Phase 4 – Milestone 4**
Follow remaining items.

## Agent Instructions
"You are working exclusively from EXECUTION-PLAN.md and MASTER-CHECKLIST.md. Ignore all files in legacy/. Follow the phases exactly. Run the repo’s standard gates before every commit."