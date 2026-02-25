# rappct EXECUTION-PLAN.md
**Single Source of Truth** — Created 2026-02-24  
**All other plans, TODO.md, TODO_AUTOPILOT.md, STUBS.md, WORKLOG.md, and scattered .md files are now DEPRECATED.**  
**Move them to `legacy/` after you create this file.

## Governance Rules (never violate)
1. Read README.md, CONTRIBUTING.md, AGENTS.md, and docs/ADR-0001 first.
2. Windows-first: all new code must be `#[cfg(windows)]` where appropriate and return `UnsupportedPlatform` elsewhere.
3. Run full quality gates before every commit (`cargo fmt`, clippy, test, `just ci-fast` if present).
4. Prefer RAII for FFI handles, SIDs, and memory.
5. Add tests alongside features; keep examples runnable.
6. Keep documentation and CHANGELOG.md in sync.

## Four Target Milestones
See MASTER-CHECKLIST.md for the detailed items under each milestone.

### Milestone 1 – First Functional Library (Core AppContainer Features) (target: 2-3 days)
### Milestone 2 – Full FFI Safety & Ownership (target: 4-5 days)
### Milestone 3 – Initial MVP (All Features + Examples) (target: 1-2 weeks)
### Milestone 4 – Finished Project (target: 3-4 weeks)

## Step-by-Step Execution Order (agent must follow exactly)

**Phase 0 – Stabilize & Bootstrap (do this first)**
1. Ensure all quality gates pass cleanly.
2. Verify basic launch and profile creation works.
3. Add the first end-to-end example test.
4. Update this file with current status notes.
5. Commit.

**Phase 1 – Milestone 1 (First Functional Library)**
1. Complete core profile, capability, launch, and ACL APIs.
2. Make basic examples run.
3. Complete Milestone 1 checklist items.
4. Update this file + commit.

**Phase 2 – Milestone 2 (FFI Safety & Ownership)**
1. Complete ADR-0001 FFI refactoring.
2. Migrate all legacy guards to new ffi wrappers.
3. Complete Milestone 2 checklist items.
4. Update this file + commit.

**Phase 3 – Milestone 3 (Initial MVP)**
1. Wire network helpers and diagnostics features.
2. Make all examples and CLI fully functional.
3. Complete Milestone 3 checklist items.
4. Update this file + commit.

**Phase 4 – Milestone 4 (Finished)**
Follow the remaining items in MASTER-CHECKLIST.md and README.md.

## Agent Instructions
"You are working exclusively from EXECUTION-PLAN.md and MASTER-CHECKLIST.md. Ignore all files in legacy/. Follow the phases exactly. Run the repo’s standard gates before every commit. After completing any milestone, update the status notes in this file and commit the change."