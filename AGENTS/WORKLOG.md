# Autopilot Worklog

Last updated: 2026-02-25

## Now
- Repo is classified as **IN-PROGRESS**.
- Standard gates pass locally: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
- Phase 2 cleanup is now complete for unchecked checklist items: legacy `crate::util::` production callsites are removed and unsafe-block safety-comments are explicitly documented.
- Phase 1 milestone work is complete; phase 2 checklist now complete in both canonical files.
- Remaining highest-impact work remains on matrix execution and full example/CLI behavioral sweep.

## Next
- Run explicit matrix validation (`scripts/ci-local.ps1`) and persist results.
- Continue remaining `MASTER-CHECKLIST.md` items in Milestones 3 and 4, especially CLI smoke coverage and matrix execution evidence.

## Later
- Complete strict `SAFETY:` unsafe audit for milestone-2 completion criteria.
- Add or adjust docs/examples to reflect any final API shape changes for `UseCase` presets.

## Done
- Implemented `UseCase` enum in `src/capability.rs`.
- Added `SecurityCapabilitiesBuilder::from_use_case(...)` returning a preset builder.
- Added `UseCase::with_profile_sid`-style finalization flow via `UseCaseCapabilities`.
- Re-exported `UseCase` in `src/lib.rs`.
- Added unit tests covering preset composition and profile finalization.
- Added `AppContainerProfile::open` in `src/profile.rs`.
- Added Windows integration test `profile_open_resolves_existing_name` in `tests/windows_profile.rs`.
- Re-ran `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` after code changes.
- Removed remaining production `crate::util::to_utf16*` callsites from `src` modules.
- Cleared local `clippy::undocumented_unsafe_blocks` suppressions from `src/ffi/mem.rs` and `src/ffi/sid.rs`.
- Updated `EXECUTION-PLAN.md` and `MASTER-CHECKLIST.md` to mark phase-2 milestones/checkpoints complete.

## Decisions Needed
- Confirm whether `UseCase::FullDesktopApp` should keep a maximal capability set or be narrowed further before next release.

## Evidence
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all-targets --all-features`: passed.
- `cargo run --example acrun -- --help`: passed.
- Source evidence: `src/capability.rs` now includes `UseCase`, `UseCaseCapabilities`, `SecurityCapabilitiesBuilder::from_use_case`, and unit tests in `builder_tests`.

## Assumptions
- Implementing `from_use_case` as preset + `with_profile_sid` API is acceptable and consistent with the checklist example.
- Remaining matrix execution is deferred to a dedicated follow-up pass.
