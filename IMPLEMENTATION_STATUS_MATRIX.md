# Implementation Status Matrix

Status definitions:

- `implemented`: readable implementation exists and aligns with documented shape.
- `partial`: readable implementation exists but coverage, docs, or operational proof is incomplete.
- `stubbed`: exported placeholder or compatibility shim exists without full runtime behavior.
- `missing`: no implementation found.
- `stale`: documentation/claim conflicts with current repository state.
- `broken`: current file state prevents credible implementation.

| Component | Files | Status | Notes |
| --- | --- | --- | --- |
| Crate metadata | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | partial | Manifest was read earlier; release checklist says version target is `0.13.10`, but changelog latest release is `0.13.4`. Needs fresh manifest/version audit before release. |
| Crate root | `src/lib.rs` | partial | Re-exports expected API and implements `supports_lpac()`, but depends on corrupted modules. |
| Error model | `src/error.rs` | implemented | `AcError` variants include `UnsupportedPlatform`, `UnsupportedLpac`, `UnknownCapability`, `AccessDenied`, `LaunchFailed`, `InvalidSid`, `ResourceNotFound`, `Win32`, `Unimplemented`. `Unimplemented` variant is a production smell if unused. |
| SID model | `src/sid.rs` | implemented/needs revalidation | Prior read found SID wrappers and validation. Must re-run tests after restore. |
| Capabilities | `src/capability.rs` | implemented/needs revalidation | Known capability catalog, derivation, builder, use-case presets, and tests were reviewed earlier. |
| ACL helpers | `src/acl.rs` | implemented/needs revalidation | File/directory/registry grant helpers were reviewed earlier; privilege-sensitive integration tests must pass. |
| Profile lifecycle | `src/profile.rs` | broken | File appears all-NUL/corrupted. Docs/tests requiring profile lifecycle are stale until restored. |
| Launch core | `src/launch/mod.rs` | broken | File appears all-NUL/corrupted. Launch APIs and examples are not credible in current checkout. |
| Launch environment helper | `src/launch/env.rs` | implemented | Builds sorted double-NUL UTF-16 env blocks, validates empty/NUL keys, deduplicates case-insensitively. |
| Token introspection | `src/token.rs` | broken | File likely zeroed; token docs/tests unverified. |
| Diagnostics | `src/diag.rs` | implemented/needs revalidation | Readable in prior pass; feature-gated warnings are documented and tested. |
| Network helpers | `src/net.rs` | partial | Readable in prior pass; mutating loopback tests are ignored/opt-in. |
| FFI handle wrappers | `src/ffi/handles.rs` | implemented/needs revalidation | RAII wrappers exist; dead-code allows are present for some APIs. |
| FFI memory wrappers | `src/ffi/mem.rs` | implemented/needs revalidation | `LocalAllocGuard` and `CoTaskMem` documented; must pass clippy unsafe lint. |
| FFI SID wrappers | `src/ffi/sid.rs` | implemented/needs revalidation | `OwnedSid` exists; must pass strict unsafe and allocator ownership checks. |
| FFI security capabilities | `src/ffi/sec_caps.rs` | implemented/needs revalidation | Owns `SECURITY_CAPABILITIES` graph; important for launch lifetime safety. |
| FFI attr list | `src/ffi/attr_list.rs` | implemented/needs revalidation | Owns process thread attribute list; important for launch. |
| FFI wide strings | `src/ffi/wstr.rs` | implemented/needs revalidation | Wide-string helper exists with dead-code annotations. |
| Test support | `src/test_support.rs`, `tests/support/windows_test_utils.rs` | implemented | Integration-only wrappers exist; `tests/support/windows_test_utils.rs` uses `#![allow(dead_code)]`. |
| Windows core tests | `tests/windows_core.rs` | partial | Tests cover capability, SID, token, LPAC override, but token/profile dependencies are corrupted. |
| Windows launch tests | `tests/windows_launch.rs` | partial | Strong intended coverage, but implementation corrupted; large file output was partially truncated during audit. |
| Job guard test | `tests/windows_job_guard.rs` | weak | Ignored and env-gated, so it does not protect default CI. |
| Network tests | `tests/windows_net.rs`, `tests/windows_net_loopback_guard.rs` | partial | Safety-latch test runs; mutating roundtrips are ignored/env-gated. |
| API surface tests | `tests/api_surface.rs` | partial | Prior read confirmed type-level API checks. Needs compile after restore. |
| Examples | `examples/acrun.rs`, `examples/rappct_demo.rs`, `examples/network_demo.rs`, `examples/advanced_features.rs`, `examples/comprehensive_demo.rs` | unverified | Examples depend on corrupted profile/launch/token modules. |
| Local CI | `Justfile`, `scripts/ci.ps1`, `scripts/ci-local.ps1` | partial | Strong local gates exist. `ci-local.ps1` skips missing MSRV/beta/nightly toolchains instead of failing, and nightly/beta failures warn only. |
| Hosted CI | `.github/workflows/ci.yml` | partial | Matrix exists. Duplicate dependency check is non-blocking. Hosted workflow does not run all local deep gates. |
| CodeQL | `.github/workflows/codeql.yml` | broken/corrupted | File output contains unrelated XML-like content, not a valid reviewed CodeQL workflow. |
| Security policy | `SECURITY.md` | implemented | Root policy is concise and local-first; legacy policy has different GitHub advisory flow details. |
| Release scripts | `scripts/release.ps1`, `scripts/release_gate.ps1`, `scripts/release_version_check.ps1` | partial | Scripts are readable and conservative. Version check depends on live network/crates.io and was not run in this audit. |
| Release checklist | `RELEASE-CHECKLIST.md` | partial/stale | Several clean-tree release gates unchecked; registry baseline from 2026-03-04 may be stale. |
| Docs | `docs/*` | partial/stale | Many docs readable and detailed, but some are corrupted and all must be regenerated after source restore. |
| Legacy docs | `legacy/*` | stale/corrupted | Some legacy docs are readable but explicitly deprecated; others are corrupted. |

