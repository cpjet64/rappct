# Autopilot Worklog

Last updated: 2026-02-25

## Now
- Repo is classified as **MOSTLY COMPLETE** after final phase-4 verification.
- Standard gates pass locally on current toolchain: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
- `scripts/ci-local.ps1` has completed successfully across MSRV 1.88, stable, beta, and nightly matrices.
- Milestone 4 signoff is now complete in `MASTER-CHECKLIST.md` and `EXECUTION-PLAN.md`.

## Next
- No immediate mandatory follow-up work remains.
- Preserve optional follow-up hardening as backlog only (e.g., capability preset scope discussion).

## Later
- Keep `UseCase::FullDesktopApp` scope question open in checklist notes if maintainers want tightened capability defaults.

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
- Added docs parity updates:
  - `README.md` now shows `UseCase`-based `SecurityCapabilitiesBuilder` usage and describes non-Windows stub behavior.
  - `docs/capabilities.md` now maps starter capability sets to API `UseCase` presets.
- Marked phase-4 final checklist item complete in `EXECUTION-PLAN.md`/`MASTER-CHECKLIST.md`.
- Confirmed `scripts/ci-local.ps1` completion and updated plan/checklist validation entry (`phase-4 matrix closure`).

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
- `cargo fmt --all -- --check`: passed for docs parity follow-up changes.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed for docs parity follow-up changes.
- `cargo test --all-targets --all-features`: passed for docs parity follow-up changes.
- Source evidence: `src/capability.rs` now includes `UseCase`, `UseCaseCapabilities`, `SecurityCapabilitiesBuilder::from_use_case`, and unit tests in `builder_tests`.

## Assumptions
- Implementing `from_use_case` as preset + `with_profile_sid` API is acceptable and consistent with the checklist example.
- Remaining matrix execution is deferred to a dedicated follow-up pass.
