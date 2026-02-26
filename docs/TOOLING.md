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

## Notes

- `cargo doc` uses local crate sources and feature flags from `Cargo.toml`.
- `mdbook build docs --dest-dir book` requires `docs/book.toml` and `docs/SUMMARY.md`.
- Keep links in [index.md](./index.md) synchronized if output paths change.
