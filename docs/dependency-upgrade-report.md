# Dependency Upgrade Report

Created: `2026-02-26T01:52:38Z`

## 2026-07-29 Modernization Sweep

| Step | Status | Notes |
|---|---|---|
| Stack Detection | Complete | Rust 2024 library crate with Cargo lockfile, Justfile gates, GitLab primary CI, GitHub mirror CI/CodeQL, mdBook docs, PowerShell/Node release tooling. |
| Compatible Direct Updates | Applied | `clap 4.6.1 -> 4.6.4`, `serde 1.0.228 -> 1.0.229`, `serde_json 1.0.149 -> 1.0.151`, `thiserror 2.0.18 -> 2.0.19`; lockfile refreshed with `cargo update`. |
| Major Upgrade Review | Complete | No direct major upgrades were available for the current dependency set. `windows 0.62.2`, `tempfile 3.27.0`, `strsim 0.11.1`, and `tracing 0.1.44` remained current. |
| Unused Dependency Check | Complete | `cargo machete` reported no unused direct dependencies. |
| Duplicate Dependency Check | Complete | `cargo tree -d --locked` reported no duplicate crate versions in the default target graph. |
| Advisory Review | Action Required | `cargo audit` exited successfully but reported warning `RUSTSEC-2026-0190` for `anyhow 1.0.102` through a target-specific WASI dev-dependency path. No baseline exception was added; reassess when upstream `tempfile/getrandom/wasip3` graph changes. |
| Release Evidence | Hardened | GitLab package jobs now emit crate SHA-256 files and `cargo-metadata.json` alongside the `.crate` artifact. |

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
