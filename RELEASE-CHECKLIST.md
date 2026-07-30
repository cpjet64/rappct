# Release Checklist (GitLab tag-driven publish flow)

Goal: maintain a clean, deterministic, GitLab tag-driven crates.io release path for an already-published crate, with a guarded local fallback.

## Current target
- Crate: `rappct`
- Planned version: `0.14.0`
- Target status: pre-release hardening is in progress; the manifest and lockfile intentionally remain at `0.13.10` until this checklist is satisfied.

## Crates.io baseline
- Refresh immediately before tagging with `just release-version-check`.
- The tag pipeline also runs `just release-version-check` before packaging or publishing.

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
- [x] Confirm `scripts/verify-version-surfaces.cjs` validates all version surfaces and tag alignment.
- [ ] Run `just prepare-release-dry-run 0.14.0` and confirm `rappct-v0.13.3` is selected.
- [ ] Run `just api-compat` and confirm only the reviewed 0.14.0 migration set is reported.
- [ ] Run `just release-surface` and confirm no production test hooks are packaged.
- [ ] Run `just prepare-release 0.14.0` on a topic branch; review, validate, and merge the three-file change.
- [ ] From synchronized clean `main`, run `just create-release-tag 0.14.0`, review the local tag, then push it explicitly.
- [x] Confirm local manifest uses explicit include policy for publish scope.
- [x] Run `just package-list` and confirm tarball output is limited to include policy paths.
- [x] Run `just publish-dry-run` and confirm lockfile + packaging checks remain green.
- [ ] Run `just release-version-check` after version bump and record the observed crates.io baseline.
- [ ] Run `just package-list-clean` (requires clean working tree).
- [ ] Run `cargo package --locked` and `just package-release-evidence`; attach `target/package/*.crate.sha256` and `target/package/cargo-metadata.json`.
- [ ] Run `just publish-dry-run-clean` (requires clean working tree).
- [ ] Run `just release-gate` after clean-tree gates are runnable.
- [ ] Run `just release-gate-log` on clean workspace and record transcript path.
- [ ] Run `just release` with explicit user confirmation `PUBLISH`.

## Audit notes
- GitHub-hosted publish workflows have been removed from release execution.
- Real publish normally occurs in GitLab tag pipeline job `publish_crates_io`.
- GitLab publish requires:
  - protected tag `vX.Y.Z` matching `Cargo.toml`
  - an assigned, online runner matching the exact `windows-protected` boundary
  - protected `CARGO_REGISTRY_TOKEN` in CI variables
  - successful protected-tag `verify_release_windows` and `package_crate` jobs; `verify_release_windows` includes the deep supply-chain gate
  - package evidence artifacts: crate tarball, SHA-256 checksum, and Cargo metadata
  - successful GitLab release creation/update job
- Local fallback still requires local confirmation in `scripts/release.ps1`, a clean `main` tree, and explicit `PUBLISH` prompt.
- Strict evidence (`output/release-gate`) should be attached before final sign-off.

## Current blocker
- Do not prepare the version or create the tag until the pre-0.14 hardening MR and exact-SHA pipeline are green.

## Historical evidence captured

- Historical `just release-version-check`: passed (`0.13.10 > 0.13.3`); refresh immediately before preparing 0.14.0.
- `just package-list` (allow-dirty): completed; output constrained to include list plus expected dirty artifacts (`.cargo_vcs_info.json`, `Cargo.toml.orig`, `Cargo.lock`).
- `just publish-dry-run` (allow-dirty): completed successfully.
- Historical `output/release-gate` evidence from before the integrity restore is not sufficient for release sign-off.
- `just release-gate`: passed on 2026-05-07 after integrity restore and gate hardening.
- A new `just release-gate-log` transcript should still be captured immediately before an actual publish.
