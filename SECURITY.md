# Security Policy

## Supported Versions

This repository follows trunk-style maintenance for active development. Security fixes are applied to the current active branch and then merged into long-lived integration branches as needed.

## Reporting a Vulnerability

Report potential vulnerabilities privately to the maintainers before opening a public issue.

Include:
- affected file/module and feature flags
- reproduction steps and expected impact
- environment details (Windows version/build, Rust toolchain, privileges)

Do not disclose exploit details publicly until a fix is available.

## Security Posture

This project intentionally preserves a strict local-first and self-hosted workflow:
- no required cloud/CDN runtime dependencies
- mandatory local CI gates (`just ci-fast`, `just ci-deep`) before integration
- dependency and advisory validation in the local/release gate (`cargo deny`, `cargo audit`, advisory policy script)
- GitLab is the primary hosted CI provider: branch and merge-request pipelines run blocking Debian, macOS, and Windows matrix checks on explicit unprotected runner boundaries
- GitLab is the sole hosted CI/CD execution provider; protected GitLab tag jobs publish matching GitLab and GitHub releases
- GitLab jobs provide Rust security coverage through Clippy, cargo-deny, cargo-audit, duplicate-dependency policy, and deterministic SBOM generation
- protected GitLab tag pipelines own crates.io and dual-provider release publishing on the Windows protected runner boundary and require protected CI credentials
- Windows boundary-sensitive functionality guarded by explicit checks and feature flags

## Hardening Notes for Contributors

- Avoid weakening sandbox, auth, ACL, or capability boundaries.
- Keep defensive error paths and capability checks fully tested.
- Do not introduce telemetry, secrets, or credential material into source, tests, or scripts.
- Maintain existing security checks in CI and local workflows.
