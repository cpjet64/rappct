# Tooling and Documentation Regeneration

This page defines exact commands to regenerate project documentation for `rappct` using `rustdoc` and `mdBook`.

## Navigation

- [Documentation Index](./index.md)
- [Documentation Overview](./overview.md)
- [Module Index](./modules/index.md)

## Prerequisites

1. Rust toolchain available (`cargo`, `rustc`).
2. Windows host recommended for full crate behavior validation.
3. `mdbook` CLI installed.

```powershell
# verify Rust toolchain
rustc --version
cargo --version

# install mdBook (one-time)
cargo install mdbook --locked

# verify mdBook
mdbook --version
```

## Exact Regeneration Commands

Run from the repository root:

```powershell
# 1) Rust API docs (all features, no dependency docs)
cargo doc --workspace --all-features --no-deps

# 2) mdBook docs from ./docs/book.toml
mdbook build docs --dest-dir book
```

Generated artifacts:

- rustdoc: `target/doc/rappct/index.html`
- mdBook: `docs/book/index.html`

## Clean Rebuild (Optional)

```powershell
# remove previous generated docs (safe local cleanup)
if (Test-Path target\doc) { Remove-Item -Recurse -Force target\doc }
if (Test-Path docs\book) { Remove-Item -Recurse -Force docs\book }

# regenerate all docs
cargo doc --workspace --all-features --no-deps
mdbook build docs --dest-dir book
```

## Verification Commands

```powershell
# quality gates commonly run before docs updates are finalized
./scripts/hygiene.ps1
python scripts/check_code_size.py
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
```

## Local Release Commands

- `just verify-version` - checks `Cargo.toml`, `Cargo.lock`, and tag/version alignment.
- `just prepare-release-dry-run X.Y.Z` - previews the selected legacy/current baseline without changing files or Git state.
- `just prepare-release X.Y.Z` - updates only version surfaces and promotes curated `Unreleased` notes for review.
- `just test-release-flow` - validates tag selection, non-publishing preparation, and exact-byte published-crate evidence in isolated fixtures.
- `just create-release-tag X.Y.Z` - verifies synchronized clean `main` and creates only a local `vX.Y.Z` tag.
- `just api-compat` - checks the exact eight reviewed 0.14.0 API break classes against published 0.13.3 using `cargo-semver-checks` 0.49.0.
- `just release-surface` - verifies production test hooks are absent and compiles a locked downstream all-features consumer.
- `just release-version-check` - checks local `Cargo.toml` version is greater than latest published crates.io version.
- `just package-list` - runs `cargo package --list --allow-dirty --locked`.
- `just package-list-clean` - runs `cargo package --list --locked` with a clean-tree precheck.
- `just package-release-evidence` - writes `target/package/cargo-metadata.json` and SHA-256 files for already packaged `.crate` artifacts.
- `just sbom` - generates and validates `target/sbom/rappct.cdx.json` as deterministic CycloneDX 1.6 JSON from locked Cargo metadata.
- `just publish-dry-run` - runs `cargo publish --dry-run --allow-dirty --locked` for ad-hoc checks.
- `just publish-dry-run-clean` - runs `cargo publish --dry-run --locked` on a clean working tree.
- `just release-gate` - runs version, API compatibility, downstream surface, `ci-deep`, package listing, and publish dry-run gates.
- `just release-gate-log` - executes `release_gate` with full transcript to:
  - `output/release-gate/release-gate-YYYY-MM-DD_HH-mm-ss.log`
  - `git branch -a`
  - `git worktree list`
  - `git status --short`
- `just release` - runs the logged gate and then prompts for explicit publish confirmation via local credentials only.

## GitLab Release Flow

- Branch and merge-request pipelines run blocking `verify_debian`, `verify_macos`, `verify_supply_chain`, and stable/MSRV `verify_windows` jobs on explicit unprotected runner boundaries. Beta and nightly Windows jobs are advisory.
- Hosted helpers use repository-local temporary directories for packaging and
  supply-chain tasks when safe. Windows AppContainer test jobs preserve the
  host process `TEMP`/`TMP` contract because `CreateAppContainerProfile` can
  fail when the entire Cargo test process is redirected; file-backed tests use
  repo-local `.tmp/` scratch internally.
- Assigned runners must have the toolchains already provisioned; CI jobs do
  not install or mutate host toolchains.
- GitLab is the sole CI/CD provider. Repository-native GitLab jobs provide Rust
  security coverage through Clippy, cargo-deny, cargo-audit,
  duplicate-dependency policy, and deterministic SBOM generation. The GitHub
  repository is a source mirror only.
- Protected tag pipelines use the Windows protected runner boundary to run `verify_release_windows` including `just release-version-check`, package `target/package/*.crate`, emit `*.crate.sha256`, `cargo-metadata.json`, and `rappct.cdx.json`, publish to crates.io using the protected `CARGO_REGISTRY_TOKEN`, upload the crate package to GitLab generic packages, and create/update the GitLab release.
- `scripts/prepare-release.ps1` selects the highest reachable `vX.Y.Z` or legacy `rappct-vX.Y.Z` baseline and promotes curated `Unreleased` notes. It never commits, tags, pushes, or publishes.

### Release safety rule

- No unattended publish is possible through the local scripts.
- Do not edit `output/release-gate` directly; it is evidence for release review and traceability.

## Notes

- `cargo doc` uses local crate sources and feature flags from `Cargo.toml`.
- `mdbook build docs --dest-dir book` requires `docs/book.toml` and `docs/SUMMARY.md`.
- Keep links in [index.md](./index.md) synchronized if output paths change.
