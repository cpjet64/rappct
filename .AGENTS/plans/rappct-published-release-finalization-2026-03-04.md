# Plan: rappct published-release finalization (2026-03-04)

## Objective
Deliver a final local-only release pass for an already-published crate with deterministic packaging and strict local-only publishing gates.

## Scope
- Version bump to the next patch (`0.13.9`).
- Publish payload hardening (`Cargo.toml` manifest policy and list checks).
- Synchronize lockfile metadata.
- Update release checklist and task tracking for current review state.
- Capture fresh dry-run/package evidence where safe in current dirty-tree context.

## Milestones

- [x] Confirm crates.io published baseline and compare against local manifest.
- [x] Bump manifest version above latest published version.
- [x] Bump `Cargo.lock` local-package version for deterministic `--locked` checks.
- [ ] Harden packaging policy so package list is limited to publishable crate assets only.
- [x] Update RELEASE-CHECKLIST and task notes with current status and new evidence path.
- [ ] Re-run clean-tree strict release gates (`package-list-clean`, `publish-dry-run-clean`, `release-gate`) once workspace is clean.
- [ ] Re-run `just release-gate-log` after strict gates and attach updated evidence.
- [ ] Publish only on explicit user command (`PUBLISH`) through `just release`.

## Current validation state
- `cargo package --list --allow-dirty --locked` should be re-run after manifest hardening and reviewed for residual non-release artifacts.
- `cargo publish --dry-run --allow-dirty --locked` should be re-run after exclude hardening and reviewed for lockfile and tarball compliance.
- Clean-tree commands remain intentionally pending until workspace clean-up.

## Risk notes
- Repository contains historical planning/checklist artifacts that can confuse future audits if not kept with explicit status context.
- `.github` CI workflows include local-hosted-quality controls but no hosted publish workflow.
- Publish is intentionally blocked until manual confirmation by design.
## 2026-03-04 follow-up (post-finalization polish)

- [x] Prevent duplicate gate execution by making `just release` flow skip in-script gate when called via `release` target.
- [x] Normalize git invocation in `scripts/release.ps1` and `scripts/release_gate.ps1` to `git.exe`.
- [x] Bump manifest and lockfile to `0.13.9`.
- [x] Update this plan to reflect include-policy behavior and remaining tarball scope cleanup.
- [ ] Capture fresh `release-gate` transcript after this cleanup on a clean workspace.
- [ ] Capture clean-tree `package-list-clean` and `publish-dry-run-clean` outputs with the updated `include` policy.

## 2026-03-04 - execution snapshot after current pass

- [x] Verified `rappct` package version in `Cargo.toml` is `0.13.10` and lockfile package version remains aligned.
- [x] Confirmed include-policy list in `Cargo.toml` is explicit (`LICENSE`, `README.md`, `Cargo.toml`, `src`, `examples`, `tests`).
- [x] Re-ran `cargo package --list --allow-dirty --locked` and verified output is limited to include-policy plus expected dirty artifacts.
- [x] Re-ran `cargo publish --dry-run --allow-dirty --locked`; precheck passed.
- [x] Re-ran `scripts/release_version_check.ps1` in this environment:
  - local `0.13.10` > published non-yanked stable `0.13.3`.
- [ ] Re-ran `just package-list-clean` on a clean tree.
- [ ] Re-ran `just publish-dry-run-clean` on a clean tree.
- [ ] Re-ran `just release-gate` / `just release-gate-log` on a clean tree and archived updated transcript.
- [ ] Executed `just release` (requires explicit `PUBLISH`) after all blockers are removed.

### Notes
- This run intentionally avoided any network publish. Real publish remains blocked by:
  - clean-tree requirement,
  - main-branch check,
  - and explicit user confirmation in `scripts/release.ps1`.
