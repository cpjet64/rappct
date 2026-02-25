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