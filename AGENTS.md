# Repository Guidelines

## Project Structure & Module Organization
The crate is organized as a standard Rust library. Core code lives under `src/`, with modules for launch tooling (`src/launch/`), capability handling (`src/capability.rs`), profiles (`src/profile.rs`), networking helpers (`src/net.rs`, feature gated), and diagnostics (`src/diag.rs`, `introspection` feature). Examples that demonstrate end-to-end usage are under `examples/`, while integration-style checks belong in `tests/`. Workspace metadata is managed by `Cargo.toml` and `Cargo.lock` at the repo root.

## Build, Test, and Development Commands
Use `cargo build` for a debug build and `cargo build --release` when you need optimized artifacts. Run `cargo test` to execute unit and integration tests; add `--features net,introspection` when validating optional modules. Examples double as smoke tests: `cargo run --example network_demo --features net` or `cargo run --example comprehensive_demo`. Clippy and formatting are part of CI, so run `cargo fmt` and `cargo clippy --all-targets --all-features` before proposing changes.

## Local Quality Gates (mandatory)
Before every commit, push, or merge, you must run the same checks CI enforces:

- Formatting: `cargo fmt --all -- --check`
- Lints: `cargo clippy --all-targets --all-features -- -D warnings`
- Tests: `cargo test --all-targets` (repeat with feature sets as needed, e.g. `--features net,introspection`)

This repository includes Git hooks and helper scripts to make this easy:

- Enable hooks locally: `git config core.hooksPath .githooks`
- Pre-commit runs fmt, clippy, and tests for the current toolchain.
- Pre-push runs the full local CI script (stable + MSRV 1.88.0–1.93.0 across feature matrix):
  - Bash: `scripts/ci-local.sh`
  - PowerShell: `scripts/ci-local.ps1`

Bypassing hooks (`--no-verify`) is discouraged and should only be used for emergencies.

## Coding Style & Naming Conventions
Follow idiomatic Rust style with `rustfmt` (default configuration). Use `snake_case` for functions and modules, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep public APIs documented with Rustdoc comments. Prefer explicit module paths over glob imports, except where the library intentionally re-exports helper types (e.g., `rappct::*` in examples).

## Testing Guidelines
Unit tests typically sit alongside the code they cover (e.g., `src/capability.rs`). Cross-module scenarios belong in `tests/` or in dedicated examples. When adding features guarded by `net` or `introspection`, include feature-flagged tests to avoid breaking default builds. Favor descriptive test names such as `lpac_defaults_enable_flag` and ensure new tests run cleanly with `cargo test --all-features` on Windows hosts.

## Commit & Pull Request Guidelines
Follow the existing history: short, lowercase, imperative subject lines with optional scopes (`ci:`, `test(windows):`). Reference related issues in the body when applicable. Pull requests should summarize the change, list any feature flags or examples to run, mention testing performed, and include screenshots or logs for user-facing demos. Keep PRs focused; split unrelated changes into separate submissions.

## Security & Configuration Tips
Many modules are Windows-only. Clearly mark new APIs with `#[cfg(windows)]` or feature gates, and guard LPAC or firewall operations behind explicit checks (`supports_lpac()`, `LoopbackAdd::confirm_debug_only()`). Avoid introducing network calls in tests unless guarded behind the `net` feature to keep CI deterministic.
