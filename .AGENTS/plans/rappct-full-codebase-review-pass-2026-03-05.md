# 2026-03-05 — Full codebase review pass for published crate

## Scope

- Deep API correctness review for non-test Rust paths (Windows runtime behaviors, release safety edges, and cross-platform behavior).
- Prioritize defects that can affect publishability, API semantics, or production reliability.
- Leave release-gating/process policy files untouched unless directly impacted by runtime correctness.

## Planned items

- [x] Scan for high-confidence cross-platform and correctness defects in production code.
- [x] Fix launch defaults to avoid Windows-only assumptions on non-Windows targets.
- [x] Tighten AppContainer profile creation fallback behavior to avoid masking invalid parameter failures.
- [x] Replace global loopback confirmation latch with thread-local confirmation scope.
- [x] Re-scan changed files and document residual findings.

## Deliverables

- `src/launch/mod.rs`
- `src/profile.rs`
- `src/net.rs`
- `.AGENTS/todo.md`

## Review status

- `src/launch/mod.rs`: complete
- `src/profile.rs`: complete
- `src/net.rs`: complete
- `Documentation/notes`: complete

## Findings captured

- `src/launch/mod.rs`: default executable path was hard-coded to Windows on all targets.
- `src/profile.rs`: `ensure` now treats only `ERROR_ALREADY_EXISTS` as a non-fatal duplicate-path.
- `src/net.rs`: global confirmation state replaced with thread-local confirmation-set keyed by SID.

## 2026-03-05 — Follow-up polish and release-adjacent correctness

### Scope

- Finish the pass with additional high-confidence defects surfaced in docs/code/scripts review:
  - rustdoc pointer paths
  - unsafe-handle sentinel checks in FFI
  - local version parsing robustness in release scripts

### Planned items

- [x] Patch stale `docs/capabilities.md` reference to the current `docs/modules/capability.md`.
- [x] Replace raw `(h as isize) == -1` handle rejection with typed `HANDLE` invalid-handle sentinel.
- [x] Update `scripts/release_version_check.ps1` to parse `version` only from `[package]` section.
- [x] Record completion and residual risk in `.AGENTS/todo.md` and `.AGENTS/lessons.md`.

### Deliverables

- `src/lib.rs`
- `src/ffi/handles.rs`
- `scripts/release_version_check.ps1`
- `.AGENTS/todo.md`
- `.AGENTS/lessons.md`

### Review status

- `src/lib.rs`: complete
- `src/ffi/handles.rs`: complete
- `scripts/release_version_check.ps1`: complete
- `tracker/docs updates`: complete

### Findings captured

- `src/lib.rs`: stale rustdoc path (`docs/capabilities.md`) pointed to a non-existing file.
- `src/ffi/handles.rs`: `Handle::from_raw` compared raw pointer values directly to `-1`, which is fragile across handle representations.
- `scripts/release_version_check.ps1`: loose `version` regex could match dependency versions before `[package]` section was narrowed.

## 2026-03-05 — Continuation findings (runtime hardening)

### Scope

- Continue non-test correctness review with emphasis on runtime edge cases and fail-closed behavior.

### Planned items

- [x] Review remaining runtime surfaces (`src/launch/env.rs`, `src/capability.rs`) for silent-failure behavior.
- [x] Fix environment block termination for explicit empty environment overrides.
- [x] Ensure capability SID derivation does not silently degrade on SID conversion failures.
- [ ] Re-run full local gates after this patch set.

### Deliverables

- `src/launch/env.rs`
- `src/capability.rs`
- `.AGENTS/todo.md`
- `.AGENTS/lessons.md`

### Findings captured

- `src/launch/env.rs`: empty environment overrides were emitted as single-NUL blocks; explicit empty env maps now emit a valid double-NUL terminator.
- `src/capability.rs`: SID conversion failures from `ConvertSidToStringSidW` were ignored; derivation now fails closed with explicit `AcError::Win32` and mismatch checks.

## 2026-03-05 — Continuation findings (startup timeout + net SID matching)

### Scope

- Resolve remaining runtime ambiguity in two places identified in the follow-up pass:
  - `src/net.rs` SID matching semantics in loopback mutation
  - `src/launch/mod.rs` unused `startup_timeout` option

### Planned items

- [x] Replace `EqualSid(...).is_ok()/is_err()` decision logic with explicit SID string matching that cleanly separates inequality from API/conversion failures.
- [x] Wire `LaunchOptions.startup_timeout` into launch execution using `WaitForInputIdle` with safe non-GUI fallback probing.
- [x] Add required Windows API feature gate for `WaitForInputIdle`.
- [ ] Re-run full local gates after this patch set.

### Deliverables

- `src/net.rs`
- `src/launch/mod.rs`
- `Cargo.toml`
- `.AGENTS/todo.md`

### Findings captured

- `src/net.rs`: `EqualSid` result-based checks could conflate normal SID inequality with error paths; logic now compares normalized SID strings and fails closed on conversion errors.
- `src/launch/mod.rs`: `startup_timeout` was previously inert; launch now enforces timeout for GUI startup-idle readiness and reports early process exit during startup probing.

## 2026-03-05 — Continuation findings (profile path, ACL target validation, capability dedupe, release script hardening)

### Scope

- Continue static correctness sweep on remaining non-test modules and release helpers.
- Apply only high-confidence, low-risk fixes that improve runtime behavior and release reliability.

### Planned items

- [x] Correct fallback AppContainer folder synthesis in `src/profile.rs`.
- [x] Tighten file-resource prevalidation semantics in `src/acl.rs`.
- [x] Eliminate duplicate capability SID derivations in `src/capability.rs`.
- [x] Ensure release publish path honors selected crate package in `scripts/release.ps1`.
- [x] Fail release evidence capture early on git command failures in `scripts/release_gate.ps1`.
- [ ] Re-run full local gates after this patch set.

### Deliverables

- `src/profile.rs`
- `src/acl.rs`
- `src/capability.rs`
- `scripts/release.ps1`
- `scripts/release_gate.ps1`
- `.AGENTS/todo.md`

### Findings captured

- `src/profile.rs`: fallback path used SID text, which does not represent the profile/package folder name under `LOCALAPPDATA\\Packages`.
- `src/acl.rs`: `ResourcePath::File` accepted existing directories because it only checked `exists()`.
- `src/capability.rs`: repeated `.with_named(...)` / `.with_lpac_defaults()` entries caused duplicate SID derivations and duplicate capability entries.
- `scripts/release.ps1`: `$Crate` argument was ignored at publish execution time.
- `scripts/release_gate.ps1`: git evidence commands were not exit-code checked, allowing partial logging on command failure.

## 2026-03-05 — Continuation findings (release version comparator robustness)

### Scope

- Harden local release version floor checks for full semver correctness and predictable failure behavior.

### Planned items

- [x] Replace Int32-based semver numeric comparison (core + prerelease) with arbitrary-length decimal string comparison.
- [x] Validate prerelease numeric identifiers to reject leading-zero numeric tokens.
- [x] Add explicit crates.io request timeout and clearer error reporting around API fetch failures.
- [ ] Re-run full local gates after this patch set.

### Deliverables

- `scripts/release_version_check.ps1`
- `.AGENTS/todo.md`
- `.AGENTS/plans/rappct-full-codebase-review-pass-2026-03-05.md`

### Findings captured

- `scripts/release_version_check.ps1`: numeric ordering (core and prerelease) relied on Int32 parsing and could misorder large numeric identifiers.
- `scripts/release_version_check.ps1`: crates.io fetch path had no request timeout and surfaced generic transport errors.

## 2026-03-05 — Continuation findings (publish credential path resolution)

### Scope

- Ensure local publish credential detection is compatible with centralized cargo home layouts.

### Planned items

- [x] Audit `scripts/release.ps1` credential discovery logic for `CARGO_HOME` compatibility.
- [x] Update credential lookup order to include `CARGO_HOME\\credentials.toml` before user-profile fallback paths.
- [ ] Re-run full local gates after this patch set.

### Deliverables

- `scripts/release.ps1`
- `.AGENTS/todo.md`
- `.AGENTS/plans/rappct-full-codebase-review-pass-2026-03-05.md`

### Findings captured

- `scripts/release.ps1`: credential detection only checked profile-home `.cargo\\credentials.toml`, which can be incorrect when `CARGO_HOME` is redirected.

## Continuation patch set (2026-03-05)

### Patch A: `src/launch/mod.rs` handle lifetime hardening

- Wrapped `PROCESS_INFORMATION` handles (`hThread`, `hProcess`) into `ffi::handles::Handle` immediately after successful `CreateProcessW`.
- Switched `AssignProcessToJobObject` to use the owned process handle.
- Removed late manual close path for `hThread`; deterministic drop now handles closure on all paths.

### Why this matters

- Prevents raw-handle lifetime gaps in post-create error branches during job setup.
- Maintains consistent ownership semantics from process creation through launch completion.

### Patch B: `src/capability.rs` defensive FFI validation

- Added explicit validation for:
  - `group_count > 0 && group_sids == null`
  - `cap_count > 0 && cap_sids == null`
- Added sibling-array cleanup on those early error exits.

### Why this matters

- Defensively avoids potential null dereference in abnormal FFI return scenarios.
- Keeps failure behavior explicit and diagnosable with targeted error messages.

## 2026-03-05 continuation implementation slice

### Implemented
- `src/ffi/handles.rs`: Added raw-handle prevalidation in `duplicate_from_raw` for null and invalid sentinel handles before calling `BorrowedHandle::borrow_raw`.
- `scripts/release_version_check.ps1`: Treat crates.io 404 or empty non-yanked stable set as first-publish path, emit explicit informational output, and skip floor enforcement.
- `Justfile`: Introduced `crate_name` from `RAPPCT_CRATE` env override and applied to release check/log/publish recipes.
- `scripts/release_gate.ps1`: Exported `RAPPCT_CRATE` from `-Crate` so `just release-gate` uses the selected crate name.
- `examples/rappct_demo.rs`: Replaced localhost `expect`/`unwrap` with error propagation via contextual `AcError::Win32`.
- `examples/advanced_features.rs`: Replaced diagnostics guard `expect` with fallible profile access and propagated error.

### Remaining next candidates
- Continue full review of tests and examples for panic-style demo code that should be recoverable.
- Continue API ergonomics pass for avoidable stringly-typed Win32 error wrapping.
- Consider tightening release script invariants around branch policy and dirty tree behavior in non-clean targets.
