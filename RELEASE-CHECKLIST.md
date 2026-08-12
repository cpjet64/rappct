# Release Checklist (GitLab tag-driven publish flow)

Goal: maintain a clean, deterministic, GitLab tag-driven crates.io and dual-provider release path for an already-published crate, with a guarded local fallback.

## Current target
- Crate: `rappct`
- Planned version: `0.14.0`
- Target status: the manifest, lockfile, and changelog are aligned at the
  release-ready `0.14.0` candidate; release remains blocked on protected
  credential confirmation and exact-SHA hosted evidence below.

## Crates.io baseline
- Refresh immediately before tagging with `just release-version-check`.
- The tag pipeline also runs `just release-version-check` before packaging or publishing.

## Packaging evidence

Publish tarball scope is now controlled by manifest `include` allow-list:
- `LICENSE`
- `README.md`
- `Cargo.toml`
- `CHANGELOG.md`
- `SECURITY.md`
- `docs/releases/0.14.0-migration.md`
- `src/**`
- `examples/**`
- `tests/**`

## Evidence checklist

- [x] Confirm local `Cargo.toml`/`Cargo.lock` versions are synchronized for the release candidate.
- [x] Confirm `scripts/verify-version-surfaces.cjs` validates all version surfaces and tag alignment.
- [x] Run `just prepare-release-dry-run 0.14.0` and confirm `rappct-v0.13.3` is selected.
- [x] Run `just api-compat` and confirm only the reviewed 0.14.0 migration set is reported.
- [x] Run `just release-surface` and confirm no production test hooks are packaged.
- [x] Run `just prepare-release 0.14.0`; because the manifest and lockfile were
  already at 0.14.0, review and commit the changelog-only candidate finalization.
- [ ] From synchronized clean `main`, run `just create-release-tag 0.14.0`, review the local tag, then push it explicitly.
- [x] Confirm local manifest uses explicit include policy for publish scope.
- [x] Run `just package-list` and confirm tarball output is limited to include policy paths.
- [x] Run `just publish-dry-run` and confirm lockfile + packaging checks remain green.
- [x] Run `just release-version-check` after version bump; observed crates.io baseline: `0.13.3`.
- [x] Run `just package-list-clean` (requires clean working tree).
- [x] Run `cargo package --locked` and `just package-release-evidence`;
  generated `rappct-0.14.0.crate` with SHA-256
  `bbe244b32836547bb86cc19f78e17e2d6c077562578cdfb20dc5b8da69571141`,
  Cargo metadata, and CycloneDX SBOM.
- [x] Run `just publish-dry-run-clean` (requires clean working tree).
- [x] Run `just release-gate` after clean-tree gates are runnable.
- [x] Run `just release-gate-log`; transcript:
  `output/release-gate/release-gate-2026-08-12_14-25-53.log`.
- [x] Run the mandatory `scripts/ci-local.ps1` stable and MSRV 1.88.0-1.95.0
  feature matrix on commit `6630f6fdcffa122bc7ec158c2f2240fc8ca7ca76`.
- [ ] Push the protected tag and let GitLab's `publish_crates_io` job publish.
  `just release` is an emergency local fallback and is intentionally not run
  during the normal protected tag flow.

## Audit notes
- GitHub-hosted automation has been retired; GitLab owns CI/CD execution and
  orchestrates matching GitLab and GitHub releases.
- Real publish normally occurs in GitLab tag pipeline job `publish_crates_io`.
- GitLab publish requires:
  - protected tag `vX.Y.Z` matching `Cargo.toml`
  - an assigned, online runner matching the exact `windows-protected` boundary
  - protected `CARGO_REGISTRY_TOKEN` in CI variables
  - protected `GITHUB_RELEASE_TOKEN` in CI variables
  - successful protected-tag `verify_release_windows` and `package_crate` jobs; `verify_release_windows` includes the deep supply-chain gate
  - package evidence artifacts: crate tarball, SHA-256 checksum, and Cargo metadata
  - successful byte-for-byte verification of the published crates.io archive against the pre-publish package
  - successful GitLab and GitHub release creation/update jobs
- Local fallback still requires local confirmation in `scripts/release.ps1`, a clean `main` tree, and explicit `PUBLISH` prompt.
- Strict evidence (`output/release-gate`) should be attached before final sign-off.

## Current blockers

- The replacement crates.io token and the GitHub release token must be added as
  protected, masked-and-hidden GitLab CI variables.
- Do not create or push the release tag until the remediation lands on
  synchronized `main`, the exact-SHA GitLab pipeline is green, and both
  protected release credentials are present. The protected GitHub release job
  fast-forwards the source mirror without force and fails closed on divergence.

## Historical evidence captured

- Historical `just release-version-check`: passed for the prior `0.13.10` candidate; refresh immediately before releasing 0.14.0.
- `just package-list` (allow-dirty): completed; output constrained to include list plus expected dirty artifacts (`.cargo_vcs_info.json`, `Cargo.toml.orig`, `Cargo.lock`).
- `just publish-dry-run` (allow-dirty): completed successfully.
- Historical `output/release-gate` evidence from before the integrity restore is not sufficient for release sign-off.
- `just release-gate`: passed on 2026-05-07 after integrity restore and gate hardening.
- A new `just release-gate-log` transcript should still be captured immediately before an actual publish.
