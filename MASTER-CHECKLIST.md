# rappct Master Completion Checklist
**Generated:** 2026-02-24  
**Single source of truth:** EXECUTION-PLAN.md  
**Agent instruction:** Verify every item against the live codebase. Do not pre-mark anything.

## Milestone 1 – First Functional Library (Core AppContainer Features)
- [ ] AppContainerProfile ensure/open/delete works
- [ ] SecurityCapabilitiesBuilder + KnownCapability mapping works
- [ ] launch_in_container (basic + with_io) works
- [ ] Token introspection works
- [ ] ACL grant helpers work for file/registry
- [ ] All quality gates pass (`cargo fmt`, clippy, test)
- [ ] Basic example (`rappct_demo`) runs successfully

## Milestone 2 – Full FFI Safety & Ownership
- [ ] All FFI RAII wrappers in `src/ffi/` implemented and adopted
- [ ] Legacy `util.rs` guards migrated to new ffi wrappers
- [ ] All `unsafe` blocks have explicit safety comments
- [ ] ADR-0001 checklist fully complete
- [ ] Unit tests for guard drop semantics and conversions
- [ ] Windows smoke tests for launch/profile/acl paths

## Milestone 3 – Initial MVP (All Features + Examples + Use Case Groupings)
- [ ] Standard Use Case Groupings implemented (see below)
- [ ] Network isolation helpers (`net` feature) work
- [ ] Diagnostics and configuration validation (`introspection`)
- [ ] All examples run cleanly
- [ ] Full CLI tool (`acrun`) functional
- [ ] Cross-feature matrix tests pass
- [ ] Documentation and examples match behavior

## Milestone 4 – Finished Project
- [ ] Full distribution (crates.io publishing, GitHub releases)
- [ ] CI matrix (MSRV 1.88, stable, beta, nightly + feature matrix)
- [ ] Security policy and responsible disclosure documented
- [ ] All stubs resolved or intentionally documented
- [ ] 100% of intended features complete and tested

## Standard Use Case Groupings (New High-Level API)
Developers can now use these presets instead of manually building capabilities:

- `UseCase::SecureWebScraper` → InternetClient + file ACL helpers
- `UseCase::IsolatedBuildEnvironment` → LPAC defaults + limited registry
- `UseCase::NetworkConstrainedTool` → privateNetworkClientServer + loopback
- `UseCase::MinimalLpac` → registryRead + lpacCom only
- `UseCase::FullDesktopApp` → broad capabilities for desktop tools
- `UseCase::Custom` → fallback for manual builder

Usage example:
```rust
let caps = SecurityCapabilitiesBuilder::from_use_case(UseCase::SecureWebScraper)
    .with_profile_sid(&profile.sid)
    .build()?;
```

## Component Checklists (for reference only)
- Core Profile & Launch
- Capability & SID Handling
- ACL Utilities
- Token Introspection
- Network Helpers
- Diagnostics
- FFI Safety & Ownership
- Use Case Groupings (new)
- Examples & CLI Tool
- CI / Release Pipeline

## Validation Report – 2026-02-25

### Milestone 1 – First Functional Library (Core AppContainer Features)
- AppContainerProfile ensure/open/delete works — **Partial**
  - Evidence: `src/profile.rs` has `ensure`, `delete`, `folder_path`, `named_object_path`, and `derive_sid_from_name`; no public `open` function was found.
- SecurityCapabilitiesBuilder + KnownCapability mapping works — **Done**
  - Evidence: `src/capability.rs` defines `SecurityCapabilitiesBuilder`, `KnownCapability`, `with_lpac_defaults`, and `build`.
  - Tests: `capability::builder_tests::lpac_defaults_enable_flag_and_append_registry_and_lpaccom` and `known_capabilities_are_mapped_to_expected_names` pass in `cargo test --all-targets --all-features`.
- launch_in_container (basic + with_io) works — **Done**
  - Evidence: `src/launch/mod.rs` exports `launch_in_container`, `launch_in_container_with_io`, and tests include `launch_with_pipes_and_echo`.
  - Evidence from `cargo run --example advanced_features -- --help` and `cargo run --example comprehensive_demo -- --help`: I/O launch path is exercised.
- Token introspection works — **Done**
  - Evidence: `src/token.rs` function `query_current_process_token`; tests in `tests/windows_core.rs` and `src/token.rs` pass.
- ACL grant helpers work for file/registry — **Done**
  - Evidence: `src/acl.rs` exposes `grant_to_package` and `grant_to_capability`; tests in `tests/windows_acl.rs` validate file and registry ACL updates.
- All quality gates pass (`cargo fmt`, clippy, test) — **Done**
  - Evidence: validation run in this pass succeeded for all three commands.
- Basic example (`rappct_demo`) runs successfully — **Done**
  - Evidence: `cargo run --example rappct_demo -- --help` executes profile + launch workflow and exits cleanly.

### Milestone 2 – Full FFI Safety & Ownership
- All FFI RAII wrappers in `src/ffi/` implemented and adopted — **Partial**
  - Evidence: `src/ffi/{handles,mem,sid,wstr,sec_caps,attr_list}.rs` exist and are used by `profile`, `acl`, `capability`, `launch`, `token`, `net`.
  - Legacy `util.rs` remains referenced in selected callsites (not fully removed).
- Legacy `util.rs` guards migrated to new ffi wrappers — **Partial**
  - Evidence: `src/util.rs` still used by `src/capability.rs` and `src/launch/mod.rs`; wrappers there are marked deprecated.
- All `unsafe` blocks have explicit safety comments — **Partial**
  - Evidence: many `SAFETY:` comments exist (search shows broad coverage), but strict neighborhood scan still reports a number of `unsafe` blocks without inline `SAFETY` wording on the preceding lines.
- ADR-0001 checklist fully complete — **Partial**
  - Evidence: `docs/adr/0001-ffi-safety-ownership.md` exists with checkboxes mostly checked and status log entries, but no automatic traceability evidence confirms full checklist closure in source and test files within this pass.
- Unit tests for guard drop semantics and conversions — **Done**
  - Evidence: tests in `src/ffi::{handles,mem,sid}` and `test_support` validate conversions and drop behavior.
- Windows smoke tests for launch/profile/acl paths — **Done**
  - Evidence: `tests/windows_launch.rs`, `tests/windows_profile.rs`, `tests/windows_acl.rs` exist and pass in `cargo test --all-targets --all-features`.

### Milestone 3 – Initial MVP (All Features + Examples + Use Case Groupings)
- Standard Use Case Groupings implemented — **Missing**
  - Evidence: checklist references `UseCase::SecureWebScraper`, `IsolatedBuildEnvironment`, etc., but no `UseCase` type or `from_use_case` API appears in `src`.
- Network isolation helpers (`net` feature) work — **Done**
  - Evidence: `src/net.rs`, `tests/windows_net.rs`, and `tests/windows_net_loopback_guard.rs` validate loopback helpers.
- Diagnostics and configuration validation (`introspection`) — **Done**
  - Evidence: `src/diag.rs` exists and is feature-gated (`#[cfg(feature = "introspection")]` in `src/lib.rs`); `tests/windows_diag.rs` passes under all-features run.
- All examples run cleanly — **Partial**
  - Evidence: `rappct_demo`, `advanced_features`, `network_demo` (with `--features net`), `comprehensive_demo`, and `acrun` help flows execute.
  - Limitation: `network_demo -- --help` fails early without `net` feature (documented gating message).
- Full CLI tool (`acrun`) functional — **Partial**
  - Evidence: `cargo run --example acrun -- --help` shows command layout (`ensure/delete/whoami/launch`) and example run path is available; functional non-help behavior was not separately exercised in this pass.
- Cross-feature matrix tests pass — **Partial**
  - Evidence: not explicitly executed here; `scripts/ci-local.ps1` exists and defines matrix tasks but matrix execution was not run in this pass.
- Documentation and examples match behavior — **Partial**
  - Evidence: `README.md`, `WORKFLOW.md`, `CHANGELOG.md`, and `SECURITY.md` are present; however, use-case preset API is currently documented in checklist but absent in code.

### Milestone 4 – Finished Project
- Full distribution (crates.io publishing, GitHub releases) — **Partial**
  - Evidence: `.github/workflows/release.yml`, README badges, `WORKFLOW.md`, and `CHANGELOG.md` describe release flow.
- CI matrix (MSRV 1.88, stable, beta, nightly + feature matrix) — **Done (configured, not fully re-run)**
  - Evidence: `.github/workflows/ci.yml`, `scripts/ci-local.ps1` define required toolchain+feature matrix.
- Security policy and responsible disclosure documented — **Done**
  - Evidence: `SECURITY.md` exists.
- All stubs resolved or intentionally documented — **Partial**
  - Evidence: non-Windows stubs are intentional in `src/ffi/mod.rs`; no unresolved TODO stubs found in src, but compatibility stubs remain intentional.
- 100% of intended features complete and tested — **Missing**
  - Evidence: `UseCase` API and some migration/completeness tasks above are still outstanding.
- Production-ready with full polish and examples — **Partial**
  - Evidence: strong baseline exists, but no final sign-off items in this audit were fully completed.
