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

## Validation Report – 2026-02-25

- Baseline evidence source: `EXECUTION-PLAN.md`, `MASTER-CHECKLIST.md`, and live source files under `src/`, `examples/`, `tests/`, `.github/`.
- Validation commands executed:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features --locked`
  - `cargo run --example rappct_demo -- --help`
  - `cargo run --example advanced_features -- --help`
  - `cargo run --example network_demo --features net -- --help`
  - `cargo run --example network_demo -- --help`
  - `cargo run --example comprehensive_demo -- --help` (with stdin supplied to proceed through prompts)
  - `cargo run --example acrun -- --help`
- Result summary:
  - Gates are green for the commands run in this environment.
  - Milestone evidence indicates a mostly-complete codebase with clear gaps around `UseCase` presets and a small residual amount of legacy FFI/util usage.

### Assessment vs. MASTER-CHECKLIST.md (as of 2026-02-25)

- Milestone 1: 83% complete (core AC/LPAC and launch/token/ACL features are implemented; `AppContainerProfile::open` API is not present as a public method).
- Milestone 2: 72% complete (FFI RAII modules are in place and used widely; legacy `util` remains in limited use; some `unsafe` blocks still have no nearby `SAFETY:` comment by strict search).
- Milestone 3: 63% complete (major feature paths are present and tested, but standard use-case presets are still documented only; cross-feature matrix verification is not explicitly completed here).
- Milestone 4: 70% complete (release/CI/security docs and workflows exist, but checklist outcome evidence still requires explicit check of operational completion criteria and policy milestones).

### Current blocker list

- Implement and expose `UseCase` presets in code (matching checklist section).
- Finish migration away from remaining `crate::util` FFI helper callsites where feasible.
- Complete a strict audit of every `unsafe` block against explicit `SAFETY:` comment coverage.
- Execute full cross-feature matrix validation in the environment and record results in this report.
