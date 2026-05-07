# Project Audit Report

Audit date: 2026-05-06  
Repository: `E:\CursorAI\rappct`  
Scope: read-only production-readiness audit of documentation, source, tests, CI, release scripts, security posture, examples, and legacy planning artifacts.

## Executive Summary

`rappct` is intended to be a Windows-focused Rust library and example CLI for AppContainer and LPAC workflows. The documented product surface is broad and mature: AppContainer profile lifecycle, capability SID derivation and catalogs, AC/LPAC process launch, token introspection, ACL grants, optional firewall loopback helpers, diagnostics, examples, CI matrix, coverage gates, security checks, and local-only crates.io release flow.

The actual repository state is not production-ready. Multiple critical files are binary-corrupted or all-NUL, including core implementation modules and canonical guidance/checklist documents. This makes many current documentation claims stale or untrustworthy. The most severe issue is that current source files required by `src/lib.rs` appear zeroed and cannot plausibly compile: `src/profile.rs`, `src/launch/mod.rs`, and likely `src/token.rs`. Important docs are also corrupted, including `AGENTS.md`, `MASTER-CHECKLIST.md`, `docs/SPEC.md`, `docs/modules/launch.md`, `legacy/rappct/STUBS.md`, and portions of `.github/workflows/codeql.yml` and legacy examples.

Because the implementation modules for profile, launch, and token are unreadable/corrupted in the current checkout, documented promises about `AppContainerProfile::ensure/open/delete`, launch APIs, `LaunchedIo`, job handling, environment merging from launch, and token introspection must be classified as unverified or broken until files are restored from a known-good source and full gates pass.

## Highest Severity Findings

| ID | Severity | Classification | Evidence | Impact | Required resolution |
| --- | --- | --- | --- | --- | --- |
| A1 | Critical | broken/corrupted | `src/profile.rs` is 14,620 bytes and reads as NUL bytes; docs claim it defines `AppContainerProfile::ensure/open/delete`, `folder_path`, `named_object_path`, and `derive_sid_from_name`. | Core profile lifecycle cannot be trusted and likely cannot compile. | Restore from known-good version, verify public API, add corruption guard to hygiene. |
| A2 | Critical | broken/corrupted | `src/launch/mod.rs` reads as NUL/corrupted bytes; docs claim it owns process creation, `LaunchOptions`, `LaunchedIo`, stdio, job objects, handle inheritance, and `merge_parent_env`. | Main runtime launch path is unavailable; core library purpose is broken. | Restore implementation and validate launch tests on Windows. |
| A3 | Critical | broken/corrupted | `src/token.rs` length is 12,757 bytes and sampled offsets read as all zeros. | Token introspection claims and tests cannot be trusted. | Restore token module and validate token tests. |
| A4 | Critical | stale/unreliable docs | `MASTER-CHECKLIST.md`, `AGENTS.md`, and `docs/SPEC.md` are NUL-corrupted while other docs say all milestones are complete. | Planning/source-of-truth documents cannot guide production completion. | Reconstruct canonical plan from readable docs plus restored code. |
| A5 | High | stale/contradictory release docs | `RELEASE-CHECKLIST.md` targets `0.13.10`; `CHANGELOG.md` latest release section is `0.13.4`; published baseline note says crates.io latest was `0.13.3` on 2026-03-04. | Release readiness cannot be asserted without fresh version and registry verification. | Re-check crates.io, manifest, lockfile, package list, release gate, and changelog. |
| A6 | High | CI coverage gap | `.github/workflows/ci.yml` uses `cargo tree -d || true`, hosted CI does not run local deep security/docs/coverage gates, and beta/nightly continue-on-error. | Hosted CI can pass with duplicate dependencies and without deep release/security evidence. | Split hard gates and advisory checks; decide which gates block PR/release. |
| A7 | High | release workflow drift | Legacy `WORKFLOW.md` claims GitHub release workflow publishes; current root `RELEASE-CHECKLIST.md` says release is local-only and GitHub-hosted publish workflows removed. | Operators can follow stale release path. | Remove or mark stale legacy release docs and keep one release source of truth. |

## Current Architecture, As Documented

Readable docs describe this intended module set:

| Area | Intended files | Intended responsibilities | Current audit status |
| --- | --- | --- | --- |
| Crate root | `src/lib.rs` | re-exports API, feature gates, `supports_lpac()` | implemented/readable, but imports corrupted modules |
| Error model | `src/error.rs` | `AcError`, `Result<T>` | implemented/readable |
| SID wrappers | `src/sid.rs` | `AppContainerSid`, `SidAndAttributes` | implemented/readable in prior pass |
| Capabilities | `src/capability.rs` | known capabilities, catalog, builder, use-case presets | implemented/readable in prior pass |
| ACL | `src/acl.rs` | file/directory/registry ACL grants | implemented/readable in prior pass |
| Profile | `src/profile.rs` | profile ensure/open/delete and path helpers | corrupted/currently broken |
| Launch | `src/launch/mod.rs`, `src/launch/env.rs` | AC/LPAC process creation, stdio, job control, env blocks | `env.rs` readable; `mod.rs` corrupted/currently broken |
| Token | `src/token.rs` | current-process AppContainer/LPAC token introspection | likely corrupted/currently broken |
| FFI | `src/ffi/*` | RAII wrappers for Win32 resources | readable and largely implemented |
| Net | `src/net.rs` | firewall loopback helpers behind `net` | implemented/readable in prior pass |
| Diag | `src/diag.rs` | config warnings behind `introspection` | implemented/readable in prior pass |

## Runtime Behavior Assessment

No build, lint, or test command was run during this audit because source corruption is already a hard blocker. Running full gates before restoring corrupted modules would produce noisy failures and does not add useful evidence.

The current runtime behavior should be treated as:

| Surface | Status | Reason |
| --- | --- | --- |
| Library compile | likely failing | `src/lib.rs` declares modules that are NUL/corrupted. |
| Profile lifecycle | broken/unverified | `src/profile.rs` corrupted. |
| Launch APIs | broken/unverified | `src/launch/mod.rs` corrupted. |
| Token introspection | broken/unverified | `src/token.rs` appears zeroed. |
| Capability/SID/ACL/FFI/net/diag | partial | readable code exists, but cannot be production-qualified while crate compile is blocked. |
| Examples/CLI | unverified | examples depend on profile/launch/token surfaces. |
| CI/release gates | stale/unverified | historical docs claim green runs, but current checkout is corrupted. |

## Production Readiness Verdict

Classification: not production-ready.

Primary reason: current checkout integrity is compromised. Production work must start by restoring source and doc files from a known-good commit or release artifact, then validating compile/test/release gates. Feature completion work should not begin until repository integrity is restored.

## Immediate Completion Strategy

1. Restore corrupted tracked files from a known-good source.
2. Add hygiene checks that detect binary/NUL corruption in text files.
3. Run minimal compile gate: `cargo check --all-targets --all-features --locked`.
4. Run full local gates: `just ci-fast`, then `just ci-deep`.
5. Reconcile docs against restored source, deleting or clearly marking stale legacy release docs.
6. Only then continue feature hardening and production polish.

