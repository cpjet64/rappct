# Autopilot Worklog

Last updated: 2026-02-25

## Now
- Repo is classified as **IN-PROGRESS**.
- Standard gates pass locally: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
- Phase 1 implementation resumed and completed: `AppContainerProfile::open` is now available and covered by test.
- Verified executable help flow remains healthy via `cargo run --example acrun -- --help`.

## Next
- Close Phase 1 milestone by validating examples and marking remaining phase checkboxes.
- Run explicit matrix validation (`scripts/ci-local.ps1`) and persist results.
- Tighten and close remaining `master` checklist milestones (CLI full-functional and documentation/examples coverage).
- Finish residual `crate::util` migration in live FFI callsites where practical.

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
