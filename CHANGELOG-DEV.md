# Changelog

## [0.12.3](https://github.com/cpjet64/rappct/compare/rappct-v0.12.2...rappct-v0.12.3) (2025-10-21)


### ⚠ BREAKING CHANGES

* v0.9.0 – API ergonomics, Windows CI, wide-string helpers, net softening, docs, release-please setup, ignore Cargo.lock

### Features

* add automatic crates.io publishing ([827becf](https://github.com/cpjet64/rappct/commit/827becf4b7aba7dacd300e6c3a7b10175509b21b))
* implement dual-branch workflow with automated dev releases ([7024aaa](https://github.com/cpjet64/rappct/commit/7024aaab227c9dc2fb242f7cc2da9751f1dc3755))
* test dev release workflow ([4c3a861](https://github.com/cpjet64/rappct/commit/4c3a861a0391e4e2c4d2e81274537a99c632e8f8))


### Bug Fixes

* add docs.rs metadata for Windows-only crate cross-compilation ([f86c6c6](https://github.com/cpjet64/rappct/commit/f86c6c6db56e04c146b98a65f90892e1bbe10acf))
* checkout correct tag when publishing to crates.io ([badda1c](https://github.com/cpjet64/rappct/commit/badda1c65cd70e95b770095ba2813d579b1c9861))
* **ci:** 2024 edition updates – mark extern blocks unsafe, wrap unsafe ops, clean imports, correct OwnedHandle::into_file; update CI should pass ([c53b3d4](https://github.com/cpjet64/rappct/commit/c53b3d4a48a8b9018e9e06a6c24fadb161b503aa))
* **ci:** 2024 unsafe rules – annotate unsafe fns, fix EqualSid result handling, silence dead_code in feature stubs, rename unused field; pass -D warnings ([eaa344f](https://github.com/cpjet64/rappct/commit/eaa344f13e0eaf91b48836ee6b103e767a749ce2))
* **ci:** edition 2024, remove unused imports, avoid LocalFree import; make set_loopback warnings non-fatal ([a80ab91](https://github.com/cpjet64/rappct/commit/a80ab91436143d516a9983ae47b10cd7193365d1))
* configure dev branch with separate tag prefix to avoid conflicts ([e85ebea](https://github.com/cpjet64/rappct/commit/e85ebea5a298accc17c9c6cdbf0f14127f8cf686))
* configure release-please-dev to target dev branch instead of main ([ff83f19](https://github.com/cpjet64/rappct/commit/ff83f19573dca442bebbd4d5efa2dd4d78714fef))
* make release-please wait for CI to pass ([a05dadc](https://github.com/cpjet64/rappct/commit/a05dadc1aecdd7774f3053f0b1f1dc706d72da1c))
* prevent release-please workflows from running on their own merge commits ([1d96a83](https://github.com/cpjet64/rappct/commit/1d96a830318eeec729e000e1e32e107566e6f484))
* rollback windows crate to 0.60 for crates.io compatibility ([d3eca28](https://github.com/cpjet64/rappct/commit/d3eca28fd8c90ae372b082a3995dd2e1f09ca092))
* update to googleapis/release-please-action@v4 ([af8f17d](https://github.com/cpjet64/rappct/commit/af8f17d379ef7b9413256b46abfaf0f062e9b9e7))


### Miscellaneous Chores

* force version to 0.12.3 ([dcc3e1f](https://github.com/cpjet64/rappct/commit/dcc3e1f5bd72cb40af07a4a2684fcfabb5ba7c27))
* v0.9.0 – API ergonomics, Windows CI, wide-string helpers, net softening, docs, release-please setup, ignore Cargo.lock ([335f20f](https://github.com/cpjet64/rappct/commit/335f20fad7b8ce9558006f0b0154338c4416afd2))

## Dev Branch Changelog

All notable changes to the dev branch will be documented in this file.

This changelog tracks pre-release versions published from the `dev` branch.
For stable releases, see [CHANGELOG.md](CHANGELOG.md).

The format is based on Keep a Changelog and this file will be managed automatically by release-please.
