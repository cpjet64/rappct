# rappct Execution Plan

Last updated: 2026-02-25 19:08:24 -05:00
Single source of truth for execution order; status checkboxes live in `MASTER-CHECKLIST.md`.

## Current repository reality (preflight baseline)
- Root planning docs were previously archived to `legacy/docs/root` and are now restored at root.
- Root `SPEC.md` is currently missing.
- Root user-facing docs (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `WORKFLOW.md`, `CHANGELOG.md`) are archived under `legacy/docs/root` and not present at root.
- Live codebase is a Rust library crate (`Cargo.toml`) with Windows-first modules under `src/`, examples under `examples/`, and integration tests under `tests/`.

## Governance rules
1. Use this file and `MASTER-CHECKLIST.md` as canonical planning artifacts at repo root.
2. Validate against live code first, then use `legacy/docs/root` for historical context.
3. Keep changes minimal and reversible; avoid unrelated code edits.
4. Run local quality gates before commit/merge.
5. Preserve Windows-first behavior and feature-gated modules.

## Execution phases

### Phase 0 - Preflight and context collection
- Read root `MASTER-CHECKLIST.md`, root `EXECUTION-PLAN.md`, and root `SPEC.md` when present.
- If `SPEC.md` is missing, record the gap and use live code + existing governance docs as temporary source.
- Confirm project type and mandatory commands from `Justfile`, `Cargo.toml`, and local CI scripts.

### Phase 1 - Live codebase validation
- Validate module/API presence from source:
  - Profiles: `src/profile.rs`
  - Capabilities and presets: `src/capability.rs`
  - Launch pipeline: `src/launch/mod.rs`
  - ACL/token/SID modules: `src/acl.rs`, `src/token.rs`, `src/sid.rs`
  - Feature exports: `src/lib.rs`
- Validate tests/examples inventory:
  - Tests: `tests/`
  - Examples: `examples/`

### Phase 2 - Legacy consistency audit
- Compare live code to legacy claims in:
  - `legacy/docs/root/MASTER-CHECKLIST.md`
  - `legacy/docs/root/EXECUTION-PLAN.md`
  - `legacy/docs/root/README.md`
  - `legacy/docs/root/WORKFLOW.md`
  - `legacy/docs/root/SECURITY.md`
- Carry forward valid claims; remove stale or contradictory statements.

### Phase 3 - Canonical doc maintenance
- Update root `MASTER-CHECKLIST.md` with concise, checkable milestone items.
- Update root `EXECUTION-PLAN.md` with clear phase ordering and validation commands.
- Update `docs/standardization-report.md` with timestamps, progress notes, and rationale.

### Phase 4 - Verification and handoff
- Verify only intended files changed.
- Confirm canonical files are consistent with repository state.
- Report created/updated files and outstanding gaps.

## Required local quality gates
Run these before commit/push/merge:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --all-targets --features net,introspection
```

Optional full local matrix:

```powershell
scripts/ci-local.ps1
```

## Agent operating note
When this plan is used by automation, complete all phases autonomously and only interrupt for truly ambiguous or high-impact decisions.
