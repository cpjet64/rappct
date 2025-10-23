# Single-Branch Development Workflow

This document describes rappct's streamlined workflow: short‑lived branches (feat/fix/docs/chore) merged into `main` via PR, with Windows‑only CI and automated releases.

## Branch Strategy

- `main` — protected, production‑ready branch.
- Short‑lived branches off `main`:
  - `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>`.
  - Keep changes focused and small; one logical change per PR when possible.

## Local Loop (before PR)

Run the repo’s mandatory gates locally (Windows host recommended):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets                   # add --features net,introspection as needed
```

Optional: run the Windows matrix locally for extra confidence (PowerShell 7+):

```powershell
scripts/ci-local.ps1
```

Helpful local test toggles (for AppContainer launch quirks): see README “Local Test Toggles”.

## Pull Request

1. Push your branch and open a PR to `main`.
2. CI runs on Windows:
   - Toolchains: `stable`, `1.90.0` (MSRV), `beta`, `nightly`.
   - Feature matrix: `""`, `introspection`, `net`, `introspection,net`.
   - Beta/nightly are continue‑on‑error; stable/MSRV are hard gates.
3. CodeQL Advanced runs on Windows for actions + Rust.
4. Merge once green; avoid force‑pushing after reviews unless necessary.

## Releases

Releases are cut from `main` by the `release` workflow when a version bump is present.

- Bump the version in `Cargo.toml` (e.g., `0.13.3`) and update `CHANGELOG.md`.
- Merge to `main` — GitHub Actions runs `.github/workflows/release.yml` (release‑plz action, `command: release`).
- The workflow tags and publishes to GitHub Releases and crates.io (requires `CARGO_REGISTRY_TOKEN`).

Notes:
- Conventional Commits help drive consistent changelogs and release notes.
- If you prefer a “release PR” flow, we can add a separate workflow that runs release‑plz with `release-pr` to generate the bump PR; current setup expects the bump to be committed.

## Conventional Commits (recommended)

| Prefix                         | Bump  | Example                                   |
|--------------------------------|-------|-------------------------------------------|
| `feat:`                        | minor | feat: add capability suggestion            |
| `fix:`                         | patch | fix: handle token query edge case          |
| `feat!:` or `BREAKING CHANGE:` | major | feat!: redesign launch API                 |
| `chore:` `docs:` `style:`      | none  | docs: update README                        |

Examples:

```bash
git commit -m "feat: add async profile creation"
git commit -m "fix: correct EqualSid error handling"
git commit -m "feat!: simplify launch API

BREAKING CHANGE: removed legacy attr module"
```

## Branch Protection (recommended)

- Require PRs to merge to `main` with at least one review.
- Require status checks to pass (stable/MSRV CI jobs).
- Require conversation resolution before merging.

## Troubleshooting

- CI failures: run `cargo test --all-targets --all-features` and `cargo clippy --all-targets --all-features -- -D warnings` locally.
- CodeQL warning about configuration mismatch on first PR after changes: safe to ignore; it resolves after merge updates the baseline.
- Release not cut: ensure `Cargo.toml` version was bumped on `main`; confirm `CARGO_REGISTRY_TOKEN` is configured.

## References

- Conventional Commits: https://www.conventionalcommits.org/
- Semantic Versioning: https://semver.org/
- release‑plz: https://release-plz.ieni.dev/
- GitHub Actions: https://docs.github.com/en/actions

