# rappct dirty tree recovery and main reintegration

## Summary

- Preserve the current mixed worktree first; do not untangle it in place on `main`.
- Rebuild two verified topic branches from a clean `main`: runtime/test fixes first, release/process migration second.
- Keep `.AGENTS/**` and `AGENTS.md` off `main`.

## Branch Strategy

1. Create `salvage/2026-03-05-dirty-tree` from the current checkout and checkpoint the full dirty state locally.
2. Create a clean recovery worktree from `main`.
3. Reconstruct `fix/windows-runtime-cleanup` from the salvage diff using only runtime/test/example changes plus the Windows feature addition in `Cargo.toml`.
4. Merge runtime back to `main` after verification.
5. Reconstruct `chore/local-release-flow` from the updated `main` using the remaining release/process/docs changes.
6. Merge release back to `main` after verification.

## Constraints

- Do not merge the salvage branch.
- Do not bring `.AGENTS/**` or `AGENTS.md` back to `main`.
- Keep `SecurityCapabilitiesBuilder::unwrap` on `main`; do not merge its removal as incidental cleanup.
- Keep the version bump and packaging metadata changes on the release/process branch only.

## Verification

- Runtime branch:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets`
  - `cargo test --all-targets --all-features`
- Release/process branch:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets`
  - `just package-list-clean`
  - `just publish-dry-run-clean`
  - `just release-gate-log`
