# Autopilot Worklog

Last updated: 2026-02-25

## Now
- Repo is classified as **IN-PROGRESS**.
- Standard gates pass locally on current toolchain: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
- Phase 3 is now closed in `MASTER-CHECKLIST.md` and `EXECUTION-PLAN.md` after format-lint matrix closure across examples/tests.
- `scripts/ci-local.ps1` has completed successfully (`[ci-local] OK`) after 1.88 clippy format-lint fixes.

## Next
- Move remaining effort into Milestone 4 completion/signoff (distribution, policy docs, stub alignment, feature completeness).
- Capture final IN-PROGRESS → FINISHED/MOSTLY COMPLETE classification after phase-4 checklist pass.

## Later
- Keep docs/signoff item open until examples/CLI/behavior parity is explicitly documented in master checklist validation prose.

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
- Replaced 1.88-breaking `format!` callsites with inline capture style in:
  - `src/acl.rs`
  - `src/ffi/attr_list.rs`
  - `src/profile.rs`
  - `src/token.rs`
  - `src/capability.rs`
- Re-ran `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
- Executed full `scripts/ci-local.ps1` matrix run to `[ci-local] OK`.
- Re-ran `rustup run 1.88.0 cargo clippy --all-targets --all-features -- -D warnings` to confirm `clippy::uninlined_format_args` is fully resolved.

## Decisions Needed
- Confirm whether `UseCase::FullDesktopApp` should keep a maximal capability set or be narrowed further before next release.

## Evidence
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all-targets --all-features`: passed.
- `cargo run --example acrun -- --help`: passed.
- `scripts/ci-local.ps1`: passed with `[ci-local] OK`.
- `cargo fmt --all -- --check`: passed (on default toolchain + post-fix).
- `rustup run 1.88.0 cargo clippy --all-targets --all-features -- -D warnings`: passed after auto-fixes.
- Source evidence: `src/capability.rs` now includes `UseCase`, `UseCaseCapabilities`, `SecurityCapabilitiesBuilder::from_use_case`, and unit tests in `builder_tests`.

## Assumptions
- Implementing `from_use_case` as preset + `with_profile_sid` API is acceptable and consistent with the checklist example.
- Remaining matrix execution is deferred to a dedicated follow-up pass.
