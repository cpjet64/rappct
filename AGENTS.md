# Repository Guidelines

## Project Structure & Module Organization
The crate is organized as a standard Rust library. Core code lives under `src/`, with modules for launch tooling (`src/launch/`), capability handling (`src/capability.rs`), profiles (`src/profile.rs`), networking helpers (`src/net.rs`, feature gated), and diagnostics (`src/diag.rs`, `introspection` feature). Examples that demonstrate end-to-end usage are under `examples/`, while integration-style checks belong in `tests/`. Workspace metadata is managed by `Cargo.toml` and `Cargo.lock` at the repo root.

## Build, Test, and Development Commands
Use `cargo build` for a debug build and `cargo build --release` when you need optimized artifacts. Run `cargo test` to execute unit and integration tests; add `--features net,introspection` when validating optional modules. Examples double as smoke tests: `cargo run --example network_demo --features net` or `cargo run --example comprehensive_demo`. Clippy and formatting are part of CI, so run `cargo fmt` and `cargo clippy --all-targets --all-features` before proposing changes.

## Local Quality Gates (mandatory)
Before every commit, push, or merge, you must run the same checks CI enforces:

- Size guard: `python scripts/check_code_size.py`
- Repository hygiene: `python scripts/hygiene.py`
- Formatting: `cargo fmt --all -- --check`
- Lints: `cargo clippy --all-targets --all-features --locked -- -D warnings`
- Tests: `cargo test --all-targets --locked` (repeat with feature sets as needed, e.g. `--features net,introspection`)

This repository includes Git hooks and helper scripts to make this easy:

- Enable hooks locally: `git config core.hooksPath .githooks`
- Pre-commit runs the size guard, fmt, clippy, and tests for the current toolchain.
- Pre-push runs the full local CI script (stable + MSRV 1.88.0–1.95.0 across feature matrix):
  - PowerShell: `scripts/ci-local.ps1`
- Supply-chain evidence: `just security` (includes deterministic CycloneDX SBOM generation).

Do not bypass hooks with `--no-verify` unless the current user explicitly
authorizes that specific bypass after receiving the exact failing gate and the
reason a normal fix or rerun cannot be used. Record any authorized bypass and
the resulting evidence gap in the task handoff.

## Coding Style & Naming Conventions
Follow idiomatic Rust style with `rustfmt` (default configuration). Use `snake_case` for functions and modules, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep public APIs documented with Rustdoc comments. Prefer explicit module paths over glob imports, except where the library intentionally re-exports helper types (e.g., `rappct::*` in examples).

## Testing Guidelines
Unit tests typically sit alongside the code they cover (e.g., `src/capability.rs`). Cross-module scenarios belong in `tests/` or in dedicated examples. When adding features guarded by `net` or `introspection`, include feature-flagged tests to avoid breaking default builds. Favor descriptive test names such as `lpac_defaults_enable_flag` and ensure new tests run cleanly with `cargo test --all-features` on Windows hosts.

## Code Size Guardrails
Hand-authored source, tests, scripts, and executable configuration must stay at or below 500 physical lines per file. Functions, methods, handlers, block-bodied closures, tests, fixtures, and helpers must stay at or below 75 logical lines. Split by cohesive responsibility and never game these limits with compression, vague wrappers, visibility changes, or broad exclusions. Integration, end-to-end, contract, system, and workflow tests belong in dedicated test files; unit tests may stay co-located only when ecosystem-idiomatic, justified, and under the file limit. Generated, vendored, lockfile, snapshot, fixture, data, and migration files are inspected but not edited solely for line count; any exception requires a documented reason. Enforce with `python scripts/check_code_size.py`, `python scripts/hygiene.py`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, and `cargo test --all-targets --locked` with required feature sets.

## Commit & Pull Request Guidelines
Follow the existing history: short, lowercase, imperative subject lines with optional scopes (`ci:`, `test(windows):`). Reference related issues in the body when applicable. Pull requests should summarize the change, list any feature flags or examples to run, mention testing performed, and include screenshots or logs for user-facing demos. Keep PRs focused; split unrelated changes into separate submissions.

## Security & Configuration Tips
Many modules are Windows-only. Clearly mark new APIs with `#[cfg(windows)]` or feature gates, and guard LPAC or firewall operations behind explicit checks (`supports_lpac()`, `LoopbackAdd::confirm_debug_only()`). Avoid introducing network calls in tests unless guarded behind the `net` feature to keep CI deterministic.

## Additional Context

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rappct is a Rust toolkit for working with Windows AppContainer (AC) and Low Privilege AppContainer (LPAC) security boundaries. It wraps Windows APIs to enable creating, managing, and launching AppContainer-aware workloads with minimal boilerplate.

**Platform**: Windows-only (non-Windows hosts return `UnsupportedPlatform`)
**MSRV**: Rust 1.88 (stable)
**Edition**: 2024

## Build & Development Commands

```powershell
# Build the library
cargo build

# Build with all features
cargo build --all-features

# Run tests (requires Windows, some tests need elevation)
cargo test --all-targets --all-features

# Run a specific test
cargo test <test_name>

# Run tests for a specific module
cargo test --test windows_launch

# Lint
cargo clippy --all-targets --all-features

# Format
cargo fmt

# Run example CLI
cargo run --example acrun -- --help
```

## Local Quality Gates & Hooks (required before commit/push/merge)

Run the same checks that CI enforces locally, every time:

- `python scripts/check_code_size.py`
- `cargo fmt --all -- --check`
- `python scripts/hygiene.py`
- `cargo clippy --all-targets --all-features --locked -- -D warnings`
- `cargo test --all-targets --locked` (repeat with feature sets as needed: `--features net`, `--features introspection`, or both)

Repository-provided hooks and scripts:

- Enable hooks once: `git config core.hooksPath .githooks`
- `.githooks/pre-commit` runs the size guard, fmt, clippy, and tests.
- `.githooks/pre-push` runs the full local CI matrix via:
  - `scripts/ci-local.ps1` (PowerShell)

Do not use `git push --no-verify` unless the current user explicitly authorizes
that specific bypass after receiving the exact failing gate and the reason a
normal fix or rerun cannot be used. Record any authorized bypass and the
resulting evidence gap in the task handoff. Keeping local gates green will keep
CI green.

**Note**: Some tests require elevated PowerShell when they involve loopback exemptions or ACL adjustments.

## Architecture Overview

### Core Module Structure

The crate is organized into focused modules that compose together:

1. **profile** (`src/profile.rs`): AppContainer profile lifecycle
   - Create/open/delete profiles via `AppContainerProfile::ensure()`
   - Derives package SIDs from profile names
   - Resolves folder paths and named-object paths

2. **capability** (`src/capability.rs`): Capability SID derivation
   - Maps `KnownCapability` enum to Windows capability names
   - Calls `DeriveCapabilitySidsFromName` (manually bound FFI)
   - Builder pattern via `SecurityCapabilitiesBuilder` to compose capabilities + LPAC flag
   - **Important**: LPAC capabilities are opt-in via `with_lpac_defaults()`

3. **launch** (`src/launch/mod.rs`, `src/launch/attr.rs`): Process launch in AC/LPAC context
   - Constructs `STARTUPINFOEX` with `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES`
   - Handles stdio redirection (Inherit/Null/Pipe) with proper handle inheritance
   - Optional job object integration (memory limits, CPU caps, kill-on-close)
   - `AttributeContext` manages lifetime of SID guards, SECURITY_CAPABILITIES struct, and attribute lists
   - `launch_in_container()` returns basic `Launched` with PID
   - `launch_in_container_with_io()` returns `LaunchedIo` with stdio handles and optional `JobGuard`

4. **token** (`src/token.rs`): Token introspection
   - Queries current process token for AppContainer/LPAC status
   - Extracts package SID and capability SIDs from token

5. **acl** (`src/acl.rs`): DACL grant helpers
   - Grant filesystem or registry access to package SIDs or capability SIDs
   - Supports `File`, `Directory`, `RegistryKey` targets
   - Registry keys support `HKCU\` and `HKLM\` roots (case-insensitive)

6. **sid** (`src/sid.rs`): SID wrappers
   - `AppContainerSid` wraps SDDL strings (e.g., "S-1-15-2-...")
   - `SidAndAttributes` pairs SID SDDL with attribute flags

7. **net** (`src/net.rs`, feature-gated): Firewall loopback exemptions
   - **WARNING**: Changes global firewall state; debug-only
   - Must call `LoopbackAdd::confirm_debug_only()` before `add_loopback_exemption()`
   - Always restore with `remove_loopback_exemption()` when done

8. **diag** (`src/diag.rs`, feature-gated): Diagnostics and validation

9. **ffi** (`src/ffi/*`): crate-private FFI RAII helpers
   - `handles::Handle`, `mem::{LocalAllocGuard, CoTaskMem}`, `sid::OwnedSid`, `wstr::WideString`, `sec_caps::OwnedSecurityCapabilities`, `attr_list::AttrList`
   - Prefer these over legacy `util` guards; `src/util.rs` remains for compatibility but should not be used in new code

### Key Architectural Patterns

**Lifetime Management via Guards**: All Windows API memory (SIDs, ACLs, handles) is wrapped in RAII guards that call appropriate cleanup functions (`LocalFree`, `FreeSid`, `CloseHandle`) on drop. The `AttributeContext` struct in `launch/mod.rs` is a critical example—it holds all the SID guards and keeps them alive while `CreateProcessW` executes.

**Builder Pattern for Capabilities**: `SecurityCapabilitiesBuilder` accumulates named capabilities and LPAC flag, then calls `derive_named_capability_sids()` in `build()`. This separates the ergonomic API from the unsafe FFI.

**FFI Boundary Isolation**: Windows APIs not exposed by the `windows` crate are manually bound (e.g., `DeriveCapabilitySidsFromName`, `CreateAppContainerProfile`) in `extern "system"` blocks. All FFI calls are `unsafe` and isolated to platform-specific `#[cfg(windows)]` sections.

**Error Handling**: `AcError` enum provides context-rich variants:
- `LaunchFailed { stage, hint, source }` for launch failures
- `UnknownCapability { name, suggestion }` with optional fuzzy suggestions (when `introspection` feature enabled)
- `UnsupportedLpac` vs `UnsupportedPlatform` for OS/platform checks

**LPAC Detection**: `supports_lpac()` queries OS build via `ntdll!RtlGetVersion` (Windows 10 build 15063+). Test-only builds that enable the private `_test_helpers` feature can override detection with `RAPPCT_TEST_LPAC_STATUS`.

## Feature Flags

- `net`: Enable loopback exemption helpers (requires `Win32_NetworkManagement_WindowsFirewall`)
- `introspection`: Enable diagnostics and capability name suggestions (adds `strsim` dependency)
- `tracing`: Emit structured logs via `tracing` crate
- `serde`: Enable Serialize/Deserialize derives on SecurityCapabilities, AppContainerSid, and SidAndAttributes

## Testing Conventions

- Integration tests in `tests/` are prefixed by platform: `windows_*.rs` for Windows-only, `api_surface.rs` for cross-platform API checks
- Tests that modify global state (firewall, registry) must clean up in `Drop`.
  File-backed tests may use `tempfile::Builder::tempdir_in`, but its parent
  must be a unique task-owned directory below the active worktree's `.tmp/`;
  never use `tempfile`'s default system-temp location.
- Use `#[cfg_attr(not(windows), ignore)]` for Windows-only tests
- CI may set `RAPPCT_TEST_LPAC_STATUS=ok` only for `_test_helpers` feature jobs that need deterministic LPAC coverage on older CI images

## Important Constraints

1. **LPAC requires Windows 10 1703+ (build 15063)**: Call `supports_lpac()` before using LPAC features
2. **Security capabilities must outlive `CreateProcessW`**: `AttributeContext` ensures this via lifetimes
3. **Handle inheritance requires explicit handle list**: When using `StdioConfig::Pipe`, pass child ends in `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`
4. **Registry ACL grants only support HKCU/HKLM**: Other roots return error
5. **Loopback exemptions are debug-only**: Never use in production

## Common Gotchas

- **Forgetting `with_lpac_defaults()`**: LPAC capabilities are opt-in; without them, the process won't have `registryRead` or `lpacCom`
- **Not waiting for child process**: `LaunchedIo` has a `wait()` method; dropping it without waiting may leave orphaned processes if `kill_on_job_close` is false
- **ACL grant failures on non-existent paths**: Ensure target file/directory/registry key exists before calling `grant_to_package()`
- **Mixing `&str` and `&OsStr` UTF-16 conversions**: Use `util::to_utf16()` for `&str`, `util::to_utf16_os()` for `&OsStr`
- **Custom environment blocks (Error 203)**: When passing `LaunchOptions::env`, it **completely replaces** the parent environment. Copy required Windows variables such as `SystemRoot`, `ComSpec`, `PATHEXT`, and `PATH`, but set `TEMP` and `TMP` explicitly to a unique task-owned directory below the active worktree's `.tmp/`. When using parent-environment inheritance, set those variables only for the invoking process. See `advanced_features.rs` Demo 5 for the pattern.
- **PowerShell console buffer errors in AppContainer (Error 0x5)**: PowerShell tries to access the console output buffer for formatting, which AppContainers restrict. Redirect output to files inside a unique task-owned directory below the active worktree's `.tmp/`, grant the AppContainer access only to that directory, read the files back with `type`, and clean up the exact task directory. Never grant an AppContainer access to the system or user temp directory. See `comprehensive_demo.rs` Demo 4 for the pattern.

## Debug Flags

- `RAPPCT_DEBUG_LAUNCH=1`: Print CreateProcessW failure details to stderr (no tracing subscriber required)
- `RAPPCT_TEST_LPAC_STATUS=ok|unsupported`: Override LPAC detection only when the private `_test_helpers` feature is enabled

## External API Bindings

These Windows APIs are manually bound because they're not fully exposed in `windows-rs`:

- `Userenv.dll`: `CreateAppContainerProfile`, `DeleteAppContainerProfile`, `DeriveAppContainerSidFromAppContainerName`, `DeriveCapabilitySidsFromName`, `GetAppContainerFolderPath`, `GetAppContainerNamedObjectPath`
- `ntdll.dll`: `RtlGetVersion` (for LPAC OS version check)
- `Advapi32.dll`: `OpenProcessToken`

## Environment

### Host Configuration Inheritance

- Inherit host-level authentication, toolchain, and package-download cache
  configuration. Do not copy host paths, credentials, or global settings into
  repository config or scripts.
- Keep build outputs repository-local. In particular, use the default
  per-project `./target/`; never configure a shared Cargo target directory.
- Do not add repository-local `CARGO_HOME`, `RUSTUP_HOME`, `SCCACHE_DIR`,
  `RUSTC_WRAPPER`, or `CARGO_INCREMENTAL` overrides unless an explicit
  repository requirement makes one necessary.
- Per-project `.cargo/config.toml` remains appropriate for genuine
  repository-specific linker flags, aliases, targets, source replacement,
  rustflags, and profile overrides.

### Project-Local Agent State

- Resolve the active repository and worktree root at runtime; do not rely on a
  hard-coded checkout path.
- Put agent worktrees in `<repo>/.worktrees/`, transient files in
  `<repo>/.tmp/`, task-specific caches in `<repo>/.cache/`, and agent logs in
  `<repo>/.agent-logs/`. These locations are gitignored.
- When a tool must use `TEMP` or `TMP`, point those variables at a task-specific
  directory under `<repo>/.tmp/` for that process only when the tool supports
  it. Do not change the host environment.
- Do not use system temp, Downloads, or an unrelated directory for project
  work. Treat insufficient disk space as a hard blocker and verify adequate
  capacity before large builds, downloads, or generation.

## Workflow Orchestration

### 1. Plan Node Default
- Use an in-session plan for non-trivial tasks when it materially improves
  execution or verification; do not create repository plan files unless the
  current request authorizes them.
- If evidence invalidates the approach, stop that approach and re-plan before
  continuing.
- Include verification in the plan, not just implementation.
- Resolve safe, reversible details from repository evidence. Ask the user only
  when missing authority, irreducible ambiguity, or a material choice prevents
  safe progress.

### 2. Subagent Strategy
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One tack per subagent for focused execution

### 3. Self-Improvement Loop
- Treat user corrections as guidance for the current task immediately.
- Update `<repo>/.AGENTS/lessons.md`, `AGENTS.md`, or another durable policy
  surface only when the current request authorizes that repository write and
  the lesson belongs in this repository. Otherwise, report the proposed durable
  lesson without writing it.
- Review existing authorized lessons at session start when they are relevant.

### 4. Verification Before Done
- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Run tests, check logs, demonstrate correctness

### 5. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes - don't over-engineer
- Challenge your own work before presenting it

### 6. Autonomous Bug Fixing
- When the current request authorizes a bug fix, investigate and implement it
  without asking for details that repository evidence can resolve safely.
- Point at logs, errors, failing tests - then resolve them
- Zero context switching required from the user
- Fix failing CI only when implementation is within the current authorized
  scope; diagnosis or review alone remains read-only.

## Hosted CI Ownership

- GitLab is the primary CI and release provider. Ordinary merge-request and
  branch pipelines run blocking non-E2E checks on the explicit Debian and
  macOS unprotected runner boundaries.
- Because this crate exercises Windows-only APIs, GitLab also reproduces the
  hosted Windows Rust/toolchain and feature matrix on the explicit Windows
  unprotected boundary. Beta and nightly remain advisory; stable and the
  supported MSRV range are blocking.
- Keep `.github/workflows/ci.yml` and `.github/workflows/codeql.yml` active as
  mirror/fallback coverage until an exact-SHA GitLab parity pipeline is green
  and its required outputs are verified. GitHub CodeQL is not implied by
  ordinary GitLab lint/test jobs.
- Tag publication is accepted only from a protected tag on the explicit
  Windows protected runner boundary. Release credentials must remain protected
  CI variables and must never be exposed to branch or merge-request jobs.
- CI helpers must use repository-local scratch where safe and stop before work
  when the repository volume has less than the helper's declared minimum free
  space. Windows AppContainer test jobs must not override process-wide
  `TEMP`/`TMP`; file-backed tests create their own repository-local `.tmp/`
  scratch. Missing runner toolchains or tools are provisioning blockers; CI
  must not install or mutate shared runner toolchains during a job.

## Task Management

1. **Initialize**: Check for the existence of and read the contents of the Justfile if present.
2. **Plan First**: Keep a checkable in-session plan for non-trivial work.
3. **Persist Conditionally**: Write plans or task state under `<repo>/.AGENTS/`
   only when the current request authorizes those repository artifacts.
4. **Verify Plan**: Continue autonomously when scope and authority are clear.
   Ask the user only when missing authority, irreducible ambiguity, or a
   material choice prevents safe progress.
5. **Track Progress**: Mark items complete in the active plan as you go.
6. **Explain Changes**: High-level summary at each step
7. **Document Results**: Add a durable repository review only when the current
   request authorizes it; otherwise include results in the handoff.
8. **Capture Lessons**: Update `<repo>/.AGENTS/lessons.md` only when explicitly
   authorized for this task.

## Core Principles

- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No Laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimal Impact**: Changes should only touch what's necessary. Avoid introducing bugs.
