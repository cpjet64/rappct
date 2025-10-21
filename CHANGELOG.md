# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this file will be managed automatically by release-please.

## v0.9.1

- Build: Update to Rust edition 2024
- Deps: Update thiserror to 2.0, windows to 0.62
- Docs: Fix README builder pattern example (remove incorrect `?` operators)
- Docs: Add `serde` feature to documentation
- Docs: Update EXAMPLES.md to reference correct examples and clarify admin requirements
- Docs: Align CLAUDE.md with current codebase state

## v0.9.0

- API: `SecurityCapabilitiesBuilder::{with_known,with_named,with_lpac_defaults}` now return `Self` (breaking change).
- Windows CI: Add GitHub Actions to test on `windows-latest` across feature matrix.
- Net: Relax firewall config enum strictness; warn instead of error when mismatch found.
- Util: Add `to_utf16_os` and use `to_utf16`/`to_utf16_os` across FFI boundaries.
- Docs: Add pipes + job limits example and LPAC test override note.
- Examples/Tests updated for new builder ergonomics.
- OwnedHandle: safer `into_file` conversion; use `from_raw` at call sites.
- Repo: set `repository` URL and ignore `Cargo.lock` for a library crate.

