# rappct

Rust toolkit for working with Windows AppContainer and LPAC process boundaries.

Project documentation is in [`docs/`](./docs/).

Start here: [`docs/index.md`](./docs/index.md)

## GitLab release process

This repository uses a GitLab tag-driven release flow. Publish payload is controlled by a manifest `include` allow-list:

- `LICENSE`
- `README.md`
- `Cargo.toml`
- `src/**`
- `examples/**`
- `tests/**`

The release chain is:

- `just bump-version-dry-run X.Y.Z` previews the semver bump and commit-derived changelog entry.
- `just bump-version X.Y.Z` updates `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md`, commits, tags `vX.Y.Z`, and pushes the branch plus tag.
- Branch and merge-request GitLab pipelines run blocking Debian, macOS, and Windows checks on explicit unprotected runner boundaries. The Windows matrix covers stable plus supported MSRV toolchains across every feature combination; beta and nightly are advisory.
- GitHub Actions CI and CodeQL remain active as mirror/fallback coverage until an exact-SHA GitLab parity pipeline is green and verified.
- A protected GitLab tag pipeline runs `just ci-deep` on the Windows protected runner boundary, verifies release version freshness, packages the crate, emits crate checksums and Cargo metadata as release evidence, publishes to crates.io, and creates/updates a GitLab release.
- `just release-version-check` verifies crate version is greater than the published crate on crates.io.
- `just release-gate` runs the full local quality/security/docs gate, packaging list, and dry-run checks on a **clean working tree**.
- `just release-gate-log` remains available for a local transcript before manual release intervention.
- `just release` remains as a guarded local fallback and prompts for explicit publish confirmation.

Do not run `cargo publish` directly outside the GitLab tag pipeline or guarded local fallback.
