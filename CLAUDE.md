# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rappct is a Rust toolkit for working with Windows AppContainer (AC) and Low Privilege AppContainer (LPAC) security boundaries. It wraps Windows APIs to enable creating, managing, and launching AppContainer-aware workloads with minimal boilerplate.

**Platform**: Windows-only (non-Windows hosts return `UnsupportedPlatform`)
**MSRV**: Rust 1.90 (stable)
**Edition**: 2024

## Build & Development Commands

```bash
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

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets` (repeat with feature sets as needed: `--features net`, `--features introspection`, or both)

Repository-provided hooks and scripts:

- Enable hooks once: `git config core.hooksPath .githooks`
- `.githooks/pre-commit` runs fmt, clippy, and tests.
- `.githooks/pre-push` runs the full local CI matrix via:
  - `scripts/ci-local.sh` (Bash) or `scripts/ci-local.ps1` (PowerShell)

Do not bypass hooks except in emergencies (`git push --no-verify`). Keeping local gates green will keep CI green.

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

**LPAC Detection**: `supports_lpac()` queries OS build via `ntdll!RtlGetVersion` (Windows 10 build 15063+). Can be overridden in tests via `RAPPCT_TEST_LPAC_STATUS` env var.

## Feature Flags

- `net`: Enable loopback exemption helpers (requires `Win32_NetworkManagement_WindowsFirewall`)
- `introspection`: Enable diagnostics and capability name suggestions (adds `strsim` dependency)
- `tracing`: Emit structured logs via `tracing` crate
- `serde`: Enable Serialize/Deserialize derives on SecurityCapabilities, AppContainerSid, and SidAndAttributes

## Testing Conventions

- Integration tests in `tests/` are prefixed by platform: `windows_*.rs` for Windows-only, `api_surface.rs` for cross-platform API checks
- Tests that modify global state (firewall, registry) should clean up in `Drop` or use `tempfile`
- Use `#[cfg_attr(not(windows), ignore)]` for Windows-only tests
- CI sets `RAPPCT_TEST_LPAC_STATUS=ok` to bypass LPAC detection on older CI images

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
- **Custom environment blocks (Error 203)**: When passing `LaunchOptions::env`, it **completely replaces** the parent environment. Windows processes require essential system variables (SystemRoot, ComSpec, PATHEXT, TEMP, TMP) to function. Always copy these from the parent environment before adding custom variables. See `advanced_features.rs` Demo 5 for the pattern.
- **PowerShell console buffer errors in AppContainer (Error 0x5)**: PowerShell tries to access the console output buffer for formatting, which AppContainers restrict. Redirect PowerShell output to temporary files using `Out-File -FilePath`, read back with `type`, and clean up with `del`. Must grant ACL access to temp directory for the AppContainer. See `network_demo.rs` and `comprehensive_demo.rs` Demo 4 for examples.

## Debug Flags

- `RAPPCT_DEBUG_LAUNCH=1`: Print CreateProcessW failure details to stderr (no tracing subscriber required)
- `RAPPCT_TEST_LPAC_STATUS=ok|unsupported`: Override LPAC detection for tests

## External API Bindings

These Windows APIs are manually bound because they're not fully exposed in `windows-rs`:

- `Userenv.dll`: `CreateAppContainerProfile`, `DeleteAppContainerProfile`, `DeriveAppContainerSidFromAppContainerName`, `DeriveCapabilitySidsFromName`, `GetAppContainerFolderPath`, `GetAppContainerNamedObjectPath`
- `ntdll.dll`: `RtlGetVersion` (for LPAC OS version check)
- `Advapi32.dll`: `OpenProcessToken`


## Environment

### Cache Locations
All package manager caches are consolidated under `C:\Dev\cache\`:

| Cache | Path | Env Variable |
|---|---|---|
| Cargo registry/git/bin | `C:\Dev\cache\cargo` | `CARGO_HOME` |
| Rustup toolchains | `C:\Dev\cache\rustup` | `RUSTUP_HOME` |
| sccache | `C:\Dev\cache\sccache` | `SCCACHE_DIR` |
| npm | `C:\Dev\cache\npm` | `npm_config_cache` |
| pnpm store | `C:\Dev\cache\pnpm-store` | pnpm config |
| pip | `C:\Dev\cache\pip` | `PIP_CACHE_DIR` |
| uv | `C:\Dev\cache\uv` | `UV_CACHE_DIR` |
| NuGet | `C:\Dev\cache\nuget` | `NUGET_PACKAGES` |
| Yarn | `C:\Dev\cache\yarn` | `YARN_CACHE_FOLDER` |

### Agent Temp Directory
If you need a temporary working directory, use `C:\Dev\agent-temp`. Do NOT use system temp or create temp dirs inside the project.

## Workflow Orchestration

### 1. Plan Node Default
- Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
- If something goes sideways, STOP and re-plan immediately - don't keep pushing
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity

### 2. Subagent Strategy
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One tack per subagent for focused execution

### 3. Self-Improvement Loop
- After ANY correction from the user: update `tasks/lessons.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project

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
- When given a bug report: just fix it. Don't ask for hand-holding
- Point at logs, errors, failing tests - then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

## Task Management

1. **Plan First**: Write plan to `tasks/todo.md` with checkable items
2. **Verify Plan**: Check in before starting implementation
3. **Track Progress**: Mark items complete as you go
4. **Explain Changes**: High-level summary at each step
5. **Document Results**: Add review section to `tasks/todo.md`
6. **Capture Lessons**: Update `tasks/lessons.md` after corrections

## Core Principles

- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No Laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimal Impact**: Changes should only touch what's necessary. Avoid introducing bugs.

