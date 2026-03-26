# Plan: rappct local-only publish full pass (2026-03-04)

## Goal
Finalize a deterministic, auditable local release path for an already-published crate and validate it with non-publishing checks.

## Scope
- Release manifest hygiene (`Cargo.toml` include policy and version bump)
- Version preflight against crates.io stable published versions
- Evidence artifacts (`package-list`, `publish-dry-run`, `release-gate-log`)
- Documentation and checklist state
- No network publish without explicit user `PUBLISH` command

## Execution checklist
- [x] Normalize release payload policy in `Cargo.toml` to explicit `include` allow-list.
- [x] Bump crate version in `Cargo.toml` above published baseline.
- [x] Sync lockfile package version entry.
- [x] Harden version check script to compare against non-yanked stable crates.io versions.
- [ ] Run `just package-list` and confirm output is restricted to `LICENSE`, `README.md`, `Cargo.toml`, `src/**`, `examples/**`, `tests/**`.
- [ ] Run `just publish-dry-run` and capture output.
- [ ] Re-run `just release-version-check` after manifest version bump.
- [ ] Update `RELEASE-CHECKLIST.md` with final evidence and next action.
- [ ] Collect clean-tree gate evidence (`package-list-clean`, `publish-dry-run-clean`, `release-gate-log`) after workspace is clean.

## 2026-03-04 - execution snapshot (post-sweep)

- [x] `scripts/release_version_check.ps1` verified local version `0.13.10` is above latest published non-yanked stable `0.13.3`.
- [x] Verified `Cargo.toml`/`Cargo.lock` version alignment at `0.13.10`.
- [x] Verified include-policy list is set to:
  - `LICENSE`, `README.md`, `Cargo.toml`, `src/**`, `examples/**`, `tests/**`.
- [x] Re-ran `just package-list` in allow-dirty mode and confirmed scope is expected.
- [x] Re-ran `just publish-dry-run` in allow-dirty mode and confirmed success.
- [ ] Re-run `just release-version-check` as part of strict release gate after any further manifest edits.
- [x] Re-ran `just package-list` and `just publish-dry-run` and updated `RELEASE-CHECKLIST.md` evidence accordingly.
- [ ] Collect clean-tree gate evidence (`package-list-clean`, `publish-dry-run-clean`, `release-gate`, `release-gate-log`) once working tree is clean.
- [ ] Finalize release via `just release` only with explicit `PUBLISH`.

## Risk notes
- Current workspace has existing uncommitted files from prior release-readiness work, so clean-tree-only commands are intentionally deferred.
- Dry-run checks remain the maximum safe action for now.
