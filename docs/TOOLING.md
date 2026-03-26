# Tooling and Documentation Regeneration

This page defines exact commands to regenerate project documentation for `rappct` using `rustdoc` and `mdBook`.

## Navigation

- [Documentation Index](./index.md)
- [Docs README](./README.md)
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

Run from repository root (`C:\Dev\repos\active\rappct`):

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
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Local Release Commands

- `just release-version-check` - checks local `Cargo.toml` version is greater than latest published crates.io version.
- `just package-list` - runs `cargo package --list --allow-dirty --locked`.
- `just package-list-clean` - runs `cargo package --list --locked` with a clean-tree precheck.
- `just publish-dry-run` - runs `cargo publish --dry-run --allow-dirty --locked` for ad-hoc checks.
- `just publish-dry-run-clean` - runs `cargo publish --dry-run --locked` on a clean working tree.
- `just release-gate` - runs full release gate (version check + `ci-fast` + package listing + dry-run).
- `just release-publish` - runs local preflight and real publish command (single interactive confirmation required in scripts).
- `just release-gate-log` - executes `release_gate` with full transcript to:
  - `output/release-gate/release-gate-YYYY-MM-DD_HH-mm-ss.log`
  - `git branch -a`
  - `git worktree list`
  - `git status --short`
- `just release` - runs the logged gate and then prompts for explicit publish confirmation via local credentials only.

### Release safety rule

- No unattended publish is possible through the local scripts.
- Do not edit `output/release-gate` directly; it is evidence for release review and traceability.

## Notes

- `cargo doc` uses local crate sources and feature flags from `Cargo.toml`.
- `mdbook build docs --dest-dir book` requires `docs/book.toml` and `docs/SUMMARY.md`.
- Keep links in [index.md](./index.md) synchronized if output paths change.

