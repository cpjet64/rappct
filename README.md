# rappct

Rust toolkit for working with Windows AppContainer and LPAC process boundaries.

Project documentation is in [`docs/`](./docs/).

Start here: [`docs/index.md`](./docs/index.md)

## Local release process (no GitHub Action publish)

This repository now uses a local-only release flow. Publish payload is controlled by a manifest `include` allow-list:

- `LICENSE`
- `README.md`
- `Cargo.toml`
- `src/**`
- `examples/**`
- `tests/**`

The release chain is:

- `just release-version-check` verifies crate version is greater than the published crate on crates.io.
- `just release-gate` runs formatting/lints/tests, packaging list, and dry-run checks on a **clean working tree**.
- `just release-gate-log` runs the full gate with a timestamped transcript in `output/release-gate`.
- `just release` runs the full gate and then prompts for explicit publish confirmation.

Do not run `cargo publish` directly outside this flow.

