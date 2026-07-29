# Dependency Upgrade Report

Created: `2026-02-26T01:52:38Z`

## 2026-07-29 Modernization Sweep

| Step | Status | Notes |
|---|---|---|
| Stack Detection | Complete | Rust 2024 library crate with Cargo lockfile, Justfile gates, GitLab primary CI, GitHub mirror CI/CodeQL, mdBook docs, PowerShell/Node release tooling. |
| Compatible Direct Updates | Applied | `clap 4.6.1 -> 4.6.4`, `serde 1.0.228 -> 1.0.229`, `serde_json 1.0.149 -> 1.0.151`, `thiserror 2.0.18 -> 2.0.19`; lockfile refreshed with `cargo update`. |
| Major Upgrade Review | Complete | No direct major upgrades were available for the current dependency set. `windows 0.62.2`, `tempfile 3.27.0`, `strsim 0.11.1`, and `tracing 0.1.44` remained current. |
| Unused Dependency Check | Complete | `cargo machete` reported no unused direct dependencies. |
| Duplicate Dependency Check | Reviewed | `syn 2.0.117` remains required by current `windows` and `tracing` proc macros while `syn 3.0.3` is required by current `thiserror`, `clap`, and `serde` proc macros. Collapsing the split requires upstream migration or direct-dependency downgrades, so it remains visible under `cargo deny`'s warning policy. |
| Advisory Review | Resolved | Updated transitive `anyhow 1.0.102 -> 1.0.103`, the first patched release for `RUSTSEC-2026-0190`. `cargo audit` now reports no vulnerabilities or warnings, and no advisory exception was added. |
| SBOM | Added | `python scripts/generate_sbom.py` generates and validates deterministic CycloneDX 1.6 JSON from locked Cargo metadata. GitLab supply-chain and protected package jobs retain the SBOM as an artifact. |
| Release Evidence | Hardened | GitLab package jobs emit crate SHA-256 files, `cargo-metadata.json`, and `rappct.cdx.json` alongside the `.crate` artifact. |

## 2026-05-08 Modernization Pass

| Step | Status | Notes |
|---|---|---|
| Stack Detection | Complete | Rust crate with Cargo manifest/lockfile, Justfile gates, GitLab CI, GitHub Actions/CodeQL, mdBook docs, and PowerShell/Node release scripts. |
| Baseline Validation | Complete | `just ci-fast` passed before dependency changes. |
| Compatible Updates | Complete | Planned direct dev-dependency updates: `clap 4.5.60 -> 4.6.1`, `tempfile 3.26.0 -> 3.27.0`; lockfile refreshed with `cargo update`. |
| Toolchain Update | Complete | Local pinned toolchain moved from `1.93.1` to `1.95.0`; MSRV remains `1.88`. |
| Matrix Alignment | Complete | Local/GitHub Rust matrix documentation extended from `1.88.0`..`1.93.0` to `1.88.0`..`1.95.0`. |

| Step | Status | Notes |
|---|---|---|

---

## Baseline and Plan

Timestamp: `2026-02-26T01:52:38Z`

| Step | Status | Notes |
|---|---|---|
| Initialization | Complete | Scope is current repo on `feat/100pct-coverage`. |
| Stack Detection | Complete | Rust crate with `Cargo.toml` + `Cargo.lock`. |
| Toolchain Baseline | Complete | `rust-toolchain.toml` channel `1.93.1`; vcvars bootstrap passed. |
| Dependency Baseline | Complete | `clap 4.5.56`, `tempfile 3.24.0`, `serde 1.0.228`, `windows 0.62.2`, `thiserror 2.0.18`, `tracing 0.1.44`, `strsim 0.11.1`. |

## Planned Waves

| Wave | Scope | Risk | Rollback |
|---|---|---|---|
| 1 | `cargo update` lockfile refresh (patch/minor) | Low | `git reset --hard HEAD~1` equivalent via reverting commit (if needed) |
| 2 | Targeted lockfile bumps for any lagging crates | Low-Medium | Revert wave commit |
| 3 | Major upgrades/toolchain changes | High (deferred) | Separate branch/commit and revert if required |

---

## Wave 1 Execution and Validation

Timestamp: `2026-02-26T01:58:27Z`

| Step | Status | Notes |
|---|---|---|
| Wave 1 Apply | Complete | `cargo update -p clap -p tempfile` |
| Wave 1 Validation | Complete | `just ci-fast` and `just ci-deep` both passed under vcvars bootstrap. |
| Security Scan | Complete | `cargo deny check`, `cargo audit`, and `python scripts/enforce_advisory_policy.py` passed. |
| Outdated Recheck | Complete | `cargo outdated -R` reports all dependencies up to date. |

## Upgrade Results

- `clap` `4.5.56 -> 4.5.60`
- `tempfile` `3.24.0 -> 3.26.0`
- Transitive updates: `clap_builder 4.5.56 -> 4.5.60`, `clap_lex 0.7.7 -> 1.0.0`, `libc 0.2.180 -> 0.2.182`, `linux-raw-sys 0.11.0 -> 0.12.1`, `rustix 1.1.3 -> 1.1.4`

## Residual Risk

- Low risk: updates are lockfile-only and backward-compatible for this crate's public API surface.
- No unresolved advisories detected after scan.
