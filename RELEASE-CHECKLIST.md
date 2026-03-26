# Release Checklist (Local-only publish flow)

Goal: maintain a clean, deterministic, local-only crates.io release path for an already-published crate.

## Current target
- Crate: `rappct`
- Planned version: `0.13.10`
- Target status: manifest and lockfile are aligned, package allow-list is explicit, and clean-tree-only strict commands are still blocked by current uncommitted workspace state.

## Crates.io baseline
- `crates.io` latest published non-prerelease version checked via API: `0.13.3` (verified 2026-03-04).
- Local version is greater than the published baseline and satisfies precondition for next publish.

## Packaging evidence

Publish tarball scope is now controlled by manifest `include` allow-list:
- `LICENSE`
- `README.md`
- `Cargo.toml`
- `src/**`
- `examples/**`
- `tests/**`

## Evidence checklist

- [x] Confirm local `Cargo.toml`/`Cargo.lock` versions are synchronized for the release candidate.
- [x] Confirm local manifest uses explicit include policy for publish scope.
- [x] Run `just package-list` and confirm tarball output is limited to include policy paths.
- [x] Run `just publish-dry-run` and confirm lockfile + packaging checks remain green.
- [x] Run `just release-version-check` after version bump (pass: `0.13.10 > 0.13.3`).
- [ ] Run `just package-list-clean` (requires clean working tree).
- [ ] Run `just publish-dry-run-clean` (requires clean working tree).
- [ ] Run `just release-gate` after clean-tree gates are runnable.
- [ ] Run `just release-gate-log` on clean workspace and record transcript path.
- [ ] Run `just release` with explicit user confirmation `PUBLISH`.

## Audit notes
- GitHub-hosted publish workflows have been removed from release execution.
- Real publish still requires:
  - local confirmation in `scripts/release.ps1`
  - clean working tree
  - branch check (`main` only)
  - explicit `PUBLISH` prompt
- Strict evidence (`output/release-gate`) should be attached before final sign-off.

## Current blocker
- Local publish is blocked until all clean-tree checks run and user gives explicit permission.

## Latest evidence captured (this pass)

- `just release-version-check`: passed (`0.13.10 > 0.13.3`).
- `just package-list` (allow-dirty): completed; output constrained to include list plus expected dirty artifacts (`.cargo_vcs_info.json`, `Cargo.toml.orig`, `Cargo.lock`).
- `just publish-dry-run` (allow-dirty): completed successfully.
- `release-gate-log` transcript currently referenced from prior full run: `output/release-gate/release-gate-2026-03-04_18-24-41.log`.
