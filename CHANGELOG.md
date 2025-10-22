# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this file will be managed automatically by release-please.

## [0.13.1] - 2025-10-22

### Miscellaneous Tasks

- MSRV 1.90, toolchain pin; add Windows MSRV job; version/docs sync\n\nfeat(net,launch): RAII loopback guard; env-merge helper; job drop-guard\n\ndocs: add capabilities catalog; README expectations; examples use new helpers\n\ntest: unit tests for helpers; opt-in integration tests for loopback and job guard

### Fmt

- Apply rustfmt changes across examples, src, and tests



## [0.12.2](https://github.com/cpjet64/rappct/compare/rappct-v0.12.1...rappct-v0.12.2) (2025-10-21)


### Bug Fixes

* checkout correct tag when publishing to crates.io ([deea246](https://github.com/cpjet64/rappct/commit/deea24647491b2443c82e731d1cdc8eeb46fcd18))

## [0.12.1](https://github.com/cpjet64/rappct/compare/rappct-v0.12.0...rappct-v0.12.1) (2025-10-21)


### Bug Fixes

* add docs.rs metadata for Windows-only crate cross-compilation ([f86c6c6](https://github.com/cpjet64/rappct/commit/f86c6c6db56e04c146b98a65f90892e1bbe10acf))
* configure release-please-dev to target dev branch instead of main ([ff83f19](https://github.com/cpjet64/rappct/commit/ff83f19573dca442bebbd4d5efa2dd4d78714fef))
* prevent release-please workflows from running on their own merge commits ([1d96a83](https://github.com/cpjet64/rappct/commit/1d96a830318eeec729e000e1e32e107566e6f484))

## [0.12.0](https://github.com/cpjet64/rappct/compare/rappct-v0.11.1...rappct-v0.12.0) (2025-10-21)


### Features

* implement dual-branch workflow with automated dev releases ([7024aaa](https://github.com/cpjet64/rappct/commit/7024aaab227c9dc2fb242f7cc2da9751f1dc3755))

## [0.11.1](https://github.com/cpjet64/rappct/compare/rappct-v0.11.0...rappct-v0.11.1) (2025-10-21)


### Bug Fixes

* make release-please wait for CI to pass ([a05dadc](https://github.com/cpjet64/rappct/commit/a05dadc1aecdd7774f3053f0b1f1dc706d72da1c))

## [0.11.0](https://github.com/cpjet64/rappct/compare/rappct-v0.10.0...rappct-v0.11.0) (2025-10-21)


### Features

* add automatic crates.io publishing ([827becf](https://github.com/cpjet64/rappct/commit/827becf4b7aba7dacd300e6c3a7b10175509b21b))

## [0.10.0](https://github.com/cpjet64/rappct/compare/rappct-v0.9.0...rappct-v0.10.0) (2025-10-21)


### ⚠ BREAKING CHANGES

* v0.9.0 – API ergonomics, Windows CI, wide-string helpers, net softening, docs, release-please setup, ignore Cargo.lock

### Bug Fixes

* **ci:** 2024 edition updates – mark extern blocks unsafe, wrap unsafe ops, clean imports, correct OwnedHandle::into_file; update CI should pass ([c53b3d4](https://github.com/cpjet64/rappct/commit/c53b3d4a48a8b9018e9e06a6c24fadb161b503aa))
* **ci:** 2024 unsafe rules – annotate unsafe fns, fix EqualSid result handling, silence dead_code in feature stubs, rename unused field; pass -D warnings ([eaa344f](https://github.com/cpjet64/rappct/commit/eaa344f13e0eaf91b48836ee6b103e767a749ce2))
* **ci:** edition 2024, remove unused imports, avoid LocalFree import; make set_loopback warnings non-fatal ([a80ab91](https://github.com/cpjet64/rappct/commit/a80ab91436143d516a9983ae47b10cd7193365d1))
* update to googleapis/release-please-action@v4 ([af8f17d](https://github.com/cpjet64/rappct/commit/af8f17d379ef7b9413256b46abfaf0f062e9b9e7))


### Miscellaneous Chores

* v0.9.0 – API ergonomics, Windows CI, wide-string helpers, net softening, docs, release-please setup, ignore Cargo.lock ([335f20f](https://github.com/cpjet64/rappct/commit/335f20fad7b8ce9558006f0b0154338c4416afd2))

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
