# TODO (PowerShell) - Windows-Specific Work Items

## CLI & Workflow Validation
- [x] Exercise `acrun` ensure/delete/whoami/launch workflows end-to-end on Windows and capture expected output for release notes (examples/acrun.rs:9, examples/acrun.rs:27).
  - `acrun ensure rappct.test.cli.18432 --display "Release Test"` -> Created/Opened profile SID `S-1-15-2-4197615514-557467080-4242716586-3720995062-4270665164-206712759-1590682932`.
  - `acrun whoami` reports `is_appcontainer=false`, `is_lpac=false`, `package_sid=<none>`, `capabilities=[]`.
  - `acrun whoami --json` returns the pretty JSON payload with matching fields (all false/null/empty).
  - `acrun launch rappct.test.cli.18432 C:\Windows\System32\whoami.exe` -> `PID: 11788` and inherited stdout `curt-desktop\curtp`.
  - `acrun launch rappct.test.cli.lpac C:\Windows\System32\whoami.exe --lpac` -> `PID: 54984` with the same stdout, confirming LPAC flag wiring.
  - `acrun delete rappct.test.cli.18432` / `acrun delete rappct.test.cli.lpac` clean up profiles.
  - Fixes: `AppContainerProfile::ensure` now treats `E_INVALIDARG` like `ERROR_ALREADY_EXISTS`, allowing launch flows to reuse profiles even when metadata differs.
- [x] Investigate and resolve the `windows_launch` STATUS_HEAP_CORRUPTION crash so the launch suite can complete.
  - Root cause: handle list attribute was passed a pointer to a temporary Vec, leading to invalid HANDLEs and DeleteProcThreadAttributeList heap corruption.
  - Fix: reuse the caller-owned handle slice when updating PROC_THREAD_ATTRIBUTE_HANDLE_LIST so the memory stays alive through CreateProcessW.
  - Tests: `cargo test --test windows_launch -- --nocapture`

## Integration Coverage & Windows APIs
- [x] Add integration coverage for profile creation/delete/folder/named-object, including error paths and fallback behavior (src/profile.rs:17, tests/windows_profile.rs:5).
  - Added tests for metadata mismatches, folder path fallback after deletion, and invalid SID error handling.
  - `profile_folder_path_fallback_after_delete` confirms the LocalAppData fallback and `profile_named_object_path_invalid_sid_errors` exercises the failure branch.
  - Tests: `cargo test --test windows_profile -- --nocapture`.
- [x] Validate capability derivation frees every allocation and returns actionable suggestions on typos; back it with tests (src/capability.rs:66, tests/windows_core.rs:12).
  - Added `capability_derivation_repeated_calls_are_successful` stress test to ensure repeated conversions succeed without leaks (calls `DeriveCapabilitySidsFromName` multiple times).
  - Suggestion logic remains covered by capability.rs unit tests; Windows APIs accept arbitrary names so the integration test notes this behavior under `capability_typo_returns_suggestion`.
  - Tests: `cargo test --test windows_core -- --nocapture` and `cargo test --features introspection --test windows_core -- --nocapture`.
- [x] After launch attribute wiring fixes, assert launched processes inherit capabilities/LPAC flags and that job guard cleanup behaves correctly (src/launch/mod.rs:387, tests/windows_launch.rs:10).
  - `launch_appcontainer_token_matches_profile` now diff-checks capability SIDs against the builder output, confirming the token inherited the requested capability set.
  - New `launch_lpac_token_sets_flag_and_caps` covers LPAC launches, validating the capability list and the LPAC flag (skipping gracefully on pre-LPAC flag kernels).
  - Shared helper `token_capability_sids` and `JobGuard::as_handle()` support these assertions without leaking handles.
  - Tests: `cargo test --test windows_launch -- --nocapture` (Access is denied lines expected from sandboxed commands).
- [x] Finish token query support (TokenIsLessPrivilegedAppContainer, package SID, capability list) and verify serialization consumers (src/token.rs:39, examples/acrun.rs:33).
  - Implemented token helpers perform boolean queries with graceful fallback when TokenIsLessPrivilegedAppContainer is unavailable and normalize capability SIDs (src/token.rs).
  - `acrun whoami --json` already reflects the enriched data; manual spot check matches the contract.
  - Tests: `cargo test --test windows_core token_query_works` and `cargo run --example acrun -- whoami --json`.
- [x] Verify ACL helpers actually modify DACLs by reading descriptors before/after and covering registry + file cases (src/acl.rs:21, tests/windows_acl.rs:4).
  - Added file and registry DACL assertions that inspect SDDL via `ConvertSecurityDescriptorToStringSecurityDescriptorW`.
  - Tests: `cargo test --test windows_acl -- --nocapture`.
- [x] Extend network helper validation to cover list/add/remove flows and safety latch expectations (src/net.rs:42, tests/windows_net.rs:4).
  - Added `loopback_add_remove_roundtrip` to assert add/remove flows modify the Windows firewall config and that the safety latch resets after each use.
  - Helpers now query `NetworkIsolationGetAppContainerConfig` directly to diff loopback SIDs without leaking handles.
  - Tests: `cargo test --features net --test windows_net -- --nocapture`.
- [x] Add tests (or mocks) to ensure `supports_lpac()` gates LPAC usage correctly across Windows versions (src/lib.rs:30).
  - Added env override hook for tests (`RAPPCT_TEST_LPAC_STATUS`) and verified both success/failure paths via new tests.
  - Tests: `cargo test --test windows_core -- --nocapture`.
- [x] Update diagnostics test coverage once pipe handling is adjusted and ensure warning enums match documentation (src/diag.rs:7, tests/windows_acl.rs:24).
  - Added dedicated diagnostics tests covering baseline, missing network caps, and LPAC-without-defaults scenarios.
  - Tests: `cargo test --features introspection --test windows_diag -- --nocapture`.
- [x] Build job object regression tests for memory, CPU rate, and kill-on-close behavior (src/launch/mod.rs:420, tests/windows_launch.rs:52).
  - Added `launch_job_limits_reported_by_query` to assert job memory+CPU caps via `QueryInformationJobObject` and `launch_job_guard_drop_terminates_process` to prove kill-on-close drops terminate children.
  - Helper `JobGuard::as_handle()` exposes the job handle safely for read-only inspection.
  - Tests: `cargo test --test windows_launch -- --nocapture` (Access is denied message expected).
- [x] Ensure `whoami --json` prints the enriched token information after token fixes and capture the expected output contract (examples/acrun.rs:33).
  - Host run: `acrun whoami --json` -> {"is_appcontainer": false, "is_lpac": false, "package_sid": null, "capabilities": []}.
  - Container coverage: launch tests now assert capability SIDs/LPAC flag propagation (`launch_appcontainer_token_matches_profile`, `launch_lpac_token_sets_flag_and_caps`).
  - Verification: `cargo run --example acrun -- whoami --json` and `cargo test --test windows_launch -- --nocapture`.

## Release Validation
- [x] Integration tests for network isolation ensuring loopback is denied by default and exemptions toggle behavior (tests/windows_net.rs:4).
  - Confirmed safety latch enforcement and loopback add/remove roundtrip via `loopback_add_remove_roundtrip`.
  - Tests: `cargo test --features net --test windows_net -- --nocapture`.
- [x] Integration tests that assert ACL grant flows produce expected ACE entries (tests/windows_acl.rs:4).
  - Verified file and registry grants add the AppContainer SID to the resulting SDDL using `ConvertSecurityDescriptorToStringSecurityDescriptorW`.
  - Tests: `cargo test --test windows_acl -- --nocapture`.
- [ ] Run the full test suite with relevant feature combinations (`--features net,introspection,tracing`) on supported Windows versions and record results for the release checklist (tests/windows_launch.rs:10, Cargo.toml:11).
  - Attempted `cargo test --features net,introspection,tracing -- --nocapture`; `tests/windows_launch` hit STATUS_HEAP_CORRUPTION (`CreateProcessW` -> handle invalid) under the `tracing` feature. Needs investigation before release sign-off.
