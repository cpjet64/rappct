# Release TODO

## Critical Fixes & Coverage Gaps
- [x] Wire the `STARTUPINFOEXW.lpAttributeList` pointer after `setup_attributes` so AppContainer/LPAC attributes actually reach `CreateProcessW`; add regression coverage that inspects the child token to verify it is inside the requested container (src/launch/mod.rs:324, src/launch/mod.rs:384, src/launch/mod.rs:407).
- [x] Complete token introspection so LPAC status, package SID, and capability SIDs are populated via the documented token information classes; update `whoami --json` and diagnostics to use the real data (src/token.rs:19, src/token.rs:26, examples/acrun.rs:33).
- [x] Align diagnostics with the current IO implementation by revisiting the stale pipe warning and adding assertions around expected warnings (src/diag.rs:32, tests/windows_launch.rs:76).
- [x] Pass the appropriate `NETISO_FLAG_FORCE_COMPUTE_BINARIES` flag (or document why not) when enumerating AppContainers and validate results against Windows firewall APIs (src/net.rs:11, src/net.rs:17).

## Verification of Completed TODO Items
### v0.1 skeleton
- [x] Re-validate project scaffolding/docs and fix the garbled punctuation artifacts in the top-level docs (README.md:1, RULES.md:1, TODO.md:1, PROMPT.md:1).
- [x] Confirm public re-exports and module visibility match the intended API surface and document any platform-specific fallbacks (src/lib.rs:8).
- [ ] Exercise `acrun` ensure/delete/whoami/launch workflows end-to-end on a Windows host and capture expected output for release notes (examples/acrun.rs:9, examples/acrun.rs:27).
- [ ] Add targeted tests for `AcError` variants to ensure the error context and sources behave as expected (src/error.rs:6, tests/windows_core.rs:22).
- [ ] Audit feature gating to make sure optional modules build only when requested and cover them with cfg-driven smoke tests (Cargo.toml:13, src/lib.rs:12).

### v0.2 core implementations
- [ ] Add integration coverage for profile creation/delete/folder/named-object, including error paths and fallback behavior (src/profile.rs:17, tests/windows_profile.rs:5).
- [ ] Validate capability derivation frees every allocation and returns actionable suggestions on typos; back it with tests (src/capability.rs:66, tests/windows_core.rs:12).
- [ ] After fixing launch attribute wiring, assert that launched processes inherit capabilities/LPAC flags and that job guard cleanup behaves correctly (src/launch/mod.rs:387, tests/windows_launch.rs:10).
- [ ] Finish token query support (TokenIsLessPrivilegedAppContainer, package SID, capability list) and verify serialization consumers (src/token.rs:39, examples/acrun.rs:33).
- [ ] Verify ACL helpers actually modify DACLs by reading descriptors before/after and covering registry + file cases (src/acl.rs:21, tests/windows_acl.rs:4).
- [ ] Extend network helper validation to cover list/add/remove flows and safety latch expectations (src/net.rs:42, tests/windows_net.rs:4).
- [ ] Add tests (or mocks) to ensure `supports_lpac()` gates LPAC usage correctly across Windows versions (src/lib.rs:30).

### v0.3 diagnostics, jobs, polish
- [ ] Update diagnostics test coverage once pipe handling is adjusted and ensure warning enums match documentation (src/diag.rs:7, tests/windows_acl.rs:24).
- [ ] Build job object regression tests for memory, CPU rate, and kill-on-close behavior (src/launch/mod.rs:420, tests/windows_launch.rs:52).
- [ ] Ensure `whoami --json` prints the enriched token information after token fixes and document the output contract (examples/acrun.rs:33).
- [ ] Add unit coverage for capability name suggestions to prove strsim thresholds behave as expected (src/capability.rs:61).
- [ ] Verify `with_lpac_defaults()` yields the documented capability set and is reflected in examples/tests (src/capability.rs:112, tests/windows_launch.rs:38).

## Outstanding Planned Work (v0.4)
- [ ] Integration tests for network isolation ensuring loopback is denied by default and exemptions toggle behavior (tests/windows_net.rs:4).
- [ ] Integration tests that assert ACL grant flows produce expected ACE entries (tests/windows_acl.rs:4).
- [ ] README additions for capability cheat sheet and migration guidance (README.md:60).
- [ ] CI matrix covering Windows Server 2019/2022 and Windows 10/11 (TODO.md:33).

## Documentation & Packaging
- [x] Update the README status blurb to reflect the remaining work and avoid claiming complete Windows coverage (README.md:7).
- [ ] Remove stray numbered source snapshots from the repo (`*.rs.num`) before publishing (src/launch/mod.rs.num:1, src/launch/attr.rs.num:1).
- [ ] Drop unused dependencies/features (`once_cell`, `async`, `nt`) or implement the promised functionality (Cargo.toml:16, Cargo.toml:17, Cargo.toml:22).
- [ ] Document the intentional net/introspection feature gating so consumers know which APIs return `Unimplemented` (src/net.rs:42, src/diag.rs:1).

## Testing & Release Process
- [ ] Run the full test suite with relevant feature combinations (`--features net,introspection,tracing`) on supported Windows versions and capture results for the release checklist (tests/windows_launch.rs:10, Cargo.toml:11).
- [ ] Ensure release artifacts/documentation mention required Windows SDK/CRT dependencies and installation steps (README.md:20).

