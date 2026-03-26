# rappct Documentation

This directory contains project documentation for `rappct`, a Windows-focused Rust toolkit for AppContainer and LPAC workflows.

## Navigation

- [Documentation Index](./index.md)
- [Tooling and Regeneration](./TOOLING.md)
- [Module Docs](./modules/index.md)

## Quickstart

```powershell
# from repository root
cargo build
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

## Generate Docs

```powershell
# rustdoc API docs
cargo doc --workspace --all-features --no-deps

# mdBook site (requires book.toml in docs/)
mdbook build docs --dest-dir book
```

Generated output locations after running the commands above:

- rustdoc: `target/doc/rappct/index.html`
- mdBook: `docs/book/index.html`

Use [TOOLING.md](./TOOLING.md) for prerequisites, exact regeneration commands, and troubleshooting.
