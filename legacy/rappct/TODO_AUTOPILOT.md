# Autopilot Worklog

Last updated: 2026-02-24

## Now
- Repo is mostly complete; no intentional feature stubs/TODOs found. Focus has shifted to conversion-validation cleanup and stabilizing local checks after launch of PowerShell tooling.
- Current in-progress tasks: finish standardization pass and resolve the remaining check failures/false-fail items.

## Next
- Reconcile `RUN-THIS-PROMPT.md` checks: ensure `checklist` items that are truly required are implemented cleanly.
- Ensure hook check behavior is unambiguous (`.githooks/pre-commit` / `.githooks/pre-push` call paths).
- Add `.cargo/config.toml` to satisfy explicit tooling check and keep it minimal/safe.
- Verify all `src/launch` changes compile (`src/launch/mod.rs` + new `src/launch/env.rs`) and run at least `just ci-fast`.

## Later
- Re-run `RUN-THIS-PROMPT.md` checklist and record exact PASS/FIX/FAIL table.
- If launch changes are unrelated to requested scope, split into a separate commit or consider stashing for a separate pass.

## Done
- Read core docs (`README.md`, `CONTRIBUTING.md`, `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md`/notes).
- Converted remaining script wrappers to PowerShell entrypoint shims (`ci.sh`, `ci-local.sh`, `hygiene.sh`).
- Converted bash usage in `README.md`, `AGENTS.md`, and `CLAUDE.md` examples and setup snippets.
- Updated hook files to use PowerShell-based execution.
- Confirmed no `TODO|FIXME|XXX|HACK` markers or `NotImplemented` stubs in main source/docs paths.

## Decisions Needed
- None blocking. Remaining decisions are around strictness of migration checklist checks.

## Evidence
- `git status --short` at start of this pass: modified `.gitignore`, `Cargo.lock`, `Justfile`, `rust-toolchain.toml`, `src/capability.rs`, `src/ffi/handles.rs`, `src/launch/mod.rs`, untracked `src/launch/env.rs`.
- `.githooks/pre-commit` and `.githooks/pre-push` run PowerShell scripts that invoke `just` gates.
- `just ci-fast` was previously successful on prior pass after wrapper cleanup.

## Assumptions
- Hook checks should be considered valid when the executable hook script ultimately executes `just ci-fast`/`ci-deep`, even when via `.sh` compatibility wrappers.
- `RUN-THIS-PROMPT.md` file-size and conflict-marker checks should ignore `.git` artifacts and should not be tripped by regex pattern strings used by `hygiene.ps1`.
