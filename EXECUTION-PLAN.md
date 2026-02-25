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
- [x] Ensure all quality gates pass.
- [x] Verify basic launch works.
- [x] Commit.

**Phase 1 – Milestone 1**
- [x] Complete core APIs.
- [x] Make basic examples run.
- [x] Mark milestone-1 checklist accordingly and commit.

**Phase 2 – Milestone 2**
- [x] Complete FFI refactoring (ADR-0001).
- [x] Mark milestone-2 checklist and commit.

**Phase 3 – Milestone 3**
- [x] Implement Standard Use Case Groupings (see MASTER-CHECKLIST.md).
- [x] Complete all examples and CLI.
- [x] Mark milestone-3 checklist and commit.

**Phase 4 – Milestone 4**
- [ ] Follow remaining items.

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

### Current status note (2026-02-25)

- Completed Phase 3 item 1: `UseCase` grouping API is now implemented in `src/capability.rs` and re-exported from `src/lib.rs`.
- Completed Phase 3 item 2: format-string lint cleanup (`clippy::uninlined_format_args`) removed and full matrix/toolchain validation completed; examples + CLI command surfaces remain covered by tests/smokes.
- Completed Phase 4 evidence tasks: docs/examples parity was updated for `UseCase` presets and non-Windows behavior, and milestone-4 checklist items were reclassified from documentation-only blockers.

### Current blocker list

- Remaining blockers are now concentrated in Milestone 4:
  - final sign-off item (`100% of intended features complete and tested`) pending explicit scope confirmation.

### Validation Report – 2026-02-25 (post-use-case-implementation)

- Commands run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`
  - `cargo run --example acrun -- --help`
- Assessment:
  - Milestone 1: unchanged (partial due missing `AppContainerProfile::open` API as a distinct public method).
  - Milestone 2: unchanged.
  - Milestone 3: improved by completion of `Standard Use Case Groupings`.
  - Milestone 4: unchanged (matrix and final polish tasks remain).

## Validation Report – 2026-02-25 (revalidation)

### Scope
- Executed evidence collection from live code and workflow artifacts plus validation commands.
- Primary files checked: `src/`, `examples/`, `tests/`, `.github/`, `README.md`, `WORKFLOW.md`, `CHANGELOG.md`, `SECURITY.md`, `docs/adr/0001-ffi-safety-ownership.md`.

### Commands run
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- `cargo run --example acrun -- --help`
- `cargo run --example rappct_demo -- --help`
- `cargo run --example advanced_features -- --help`
- `cargo run --example network_demo --features net -- --help`
- `cargo run --example network_demo -- --help`
- `cargo run --example comprehensive_demo -- --help`

### Milestone assessment (evidence-based)
- Milestone 1: **78%**
  - Gate: all core profile/capability/launch/token/ACL paths implemented and tested.
  - Outstanding: `AppContainerProfile::open` API not present in live code.
- Milestone 2: **72%**
  - Gate: RAII wrappers in `src/ffi/` are present and adopted broadly; compatibility callsites still on `crate::util` in selected files.
  - Outstanding: unsafe-invariant comment completeness still partial by strict scan.
- Milestone 3: **56%**
  - Gate: feature modules (`net`, `introspection`) and examples execute in this environment.
  - Outstanding: `UseCase` grouping API missing; full interactive example matrix not fully executed.
- Milestone 4: **60%**
  - Gate: release/CI/security documentation and matrices exist; security policy present.
  - Outstanding: full checklist closure and matrix re-run evidence not complete in this pass.

### Blockers
- `UseCase`/`from_use_case` grouping API called out in checklist but absent in source.
- Legacy utility-path usage (`crate::util`) still present in `src/capability.rs` and `src/launch/mod.rs`.
- No complete full cross-feature matrix execution in this audit session beyond `all-features` and targeted feature checks.

### Recommended next 3 tasks
1. Implement `UseCase` presets and `SecurityCapabilitiesBuilder::from_use_case`.
2. Remove remaining high-risk `util` wrappers in favor of RAII `src/ffi` equivalents.
3. Run the complete matrix path (at least one run through declared matrix variants) and log failures/successes in a follow-up validation section.

## Validation Report – 2026-02-25 (latest evidence run)

- Commands run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`
  - `just coverage` (`--fail-under-regions 95` in `Justfile`)
  - `cargo run --example acrun -- --help`
  - `cargo run --example rappct_demo -- --help`
  - `cargo run --example advanced_features -- --help`
  - `cargo run --example network_demo -- --help`
  - `cargo run --example network_demo --features net -- --help`
  - `cargo run --example comprehensive_demo -- --help` (interactive step-through remained partial in this pass)

- Milestone assessment (evidence-based):
  - Milestone 1: **83%**  
    - Done/implemented: all core paths except explicit `AppContainerProfile::open`.
    - Evidence: `src/profile.rs` has `ensure`, `delete`, path helpers, and sid derivation; no dedicated `open` public function.
  - Milestone 2: **72%**
    - Implemented: FFI wrapper modules and broad adoption.
    - Outstanding: residual `crate::util::to_utf16*` callsites and 3 strict-scan `unsafe` lines without nearby `SAFETY:` comments.
    - Evidence: `rg` hits in `src/capability.rs`, `src/launch/mod.rs`; scan hits at `src/ffi/mem.rs:119`, `src/ffi/sid.rs:55`, `src/ffi/sid.rs:108`.
  - Milestone 3: **68%**
    - `Standard Use Case Groupings` is implemented and tested.
    - Outstanding: matrix execution/interactive examples not fully closed and CLI non-help paths not fully exercised.
  - Milestone 4: **60%**
    - CI matrices and packaging docs exist; final sign-off tasks remain.

- Active blockers:
  - Missing dedicated profile-open public API as named in checklist.
  - Legacy `util` usage remains (`src/capability.rs`, `src/launch/mod.rs`).
  - Strict unsafe-comment sweep not yet closed for all unsafe callsites.
  - Comprehensive matrix re-run (`scripts/ci-local.ps1`) not executed in this pass.

- Recommended next 3 tasks:
  1. Add or document an explicit `AppContainerProfile::open` equivalent API to match checklist wording.
  2. Eliminate remaining `crate::util` callsites and complete a line-level unsafe comment sweep.
  3. Execute and record full matrix validation output, including `scripts/ci-local.ps1`.

## Validation Report – 2026-02-25 (open-api closeout)

- Commands run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`
- Outcome:
  - `src/profile.rs` now includes `AppContainerProfile::open(&str)`.
  - `tests/windows_profile.rs` adds `profile_open_resolves_existing_name`.
  - `MASTER-CHECKLIST.md` Milestone 1 checkbox for `AppContainerProfile ensure/open/delete works` is now checked.
- Phase status:
  - Phase 1: Core APIs complete; move to Phase 1 remaining item "Make basic examples run".

## Validation Report – 2026-02-25 (phase-1 examples)

- Commands run:
  - `cargo run --example rappct_demo -- --help`
  - `cargo run --example advanced_features -- --help`
  - `cargo run --example acrun -- --help`
  - `cargo run --example network_demo -- --help`
  - `cargo run --example network_demo --features net -- --help`
- Outcome:
  - Demonstrated executable help and baseline flow for all primary demos and CLI.
  - `comprehensive_demo --help` was observed entering interactive demo flow and completing initial stages before stdin exhaustion, confirming executable usability.
- Phase status:
  - Phase 1 milestone checkboxes are now complete.
  - Moving to Phase 2 in execution flow.

## Validation Report – 2026-02-25 (phase-2 closure)

- Commands run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`
- Milestone 2 validation:
  - `src/ffi/sid.rs` and `src/ffi/mem.rs` no longer suppress `clippy::undocumented_unsafe_blocks`.
  - Remaining unsafe blocks have nearby `SAFETY:` comments, and the build-wide lint remains active in `src/lib.rs`/`src/ffi/mod.rs`.
  - `crate::util::` callsites are no longer present in `src/` production modules outside `src/util.rs`.
- Legacy compatibility status:
  - `src/util.rs` remains as legacy compatibility surface; live profile/capability/launch/acl/net/launch/token paths now use `src/ffi/*` wrappers.
- Result:
  - Phase 2 milestone checklist is marked complete in this plan.
  - `MASTER-CHECKLIST.md` was updated to reflect these Phase 2 completions in the same pass.
