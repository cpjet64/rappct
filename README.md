# rappct - Rust AppContainer / LPAC toolkit (Windows)

**rappct** is a focused Rust crate for working with **Windows AppContainer** and **LPAC**:
profile lifecycle, capability derivation, secure process launch (AC/LPAC), token introspection,
network loopback exemptions (debug-only), ACL helpers, and diagnostics.

> Status: Core Windows functionality implemented, with remaining Windows-only work in progress. The public API remains stable.
> Windows implementations now cover profiles, capabilities, secure launch (AC/LPAC), token
> introspection, ACL helpers, optional network isolation helpers (feature: `net`), and diagnostics
> (feature: `introspection`). On non-Windows, APIs return `UnsupportedPlatform`.

## Features
- AppContainer profile create/open/delete; resolve profile & named-object paths.
- Capability SIDs (known + named) using `DeriveCapabilitySidsFromName`.
- Process launch (AC/LPAC) using `STARTUPINFOEX` + `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES` and
  `ALL_APPLICATION_PACKAGES_POLICY=OPT_OUT` for LPAC.
- Token introspection: is AC/LPAC, package SID, capability SIDs.
- Network helpers (feature: `net`): list AppContainers; add/remove loopback exemptions (**dev-only**, requires explicit confirm).
- ACL helpers for files/dirs/registry.
- Optional Job Objects (resource limits).
- Diagnostics & configuration validation (feature: `introspection`).

## Feature gating and platform behaviour
- Default builds enable the core AppContainer and LPAC APIs only. Optional modules stay
  dormant until their feature is requested.
- `net` turns on firewall loopback helpers and AppContainer enumeration. When this feature is
  disabled the corresponding functions return `AcError::Unimplemented`, and on non-Windows hosts
  they return `AcError::UnsupportedPlatform`.
- `introspection` enables diagnostics such as `validate_configuration` and capability name
  suggestions. Without it, suggestion hints are omitted and diagnostics that rely on capability
  derivation fall back to `AcError::Unimplemented`/`UnsupportedPlatform` where appropriate.
- `tracing` adds optional instrumentation via the `tracing` crate.

## Capability cheat sheet & migration notes
- **Internet access**: `KnownCapability::InternetClient` (outbound) or
  `KnownCapability::InternetClientServer` (outbound + inbound) map to the manifest values used by
  UWP/AppContainer apps. Use `SecurityCapabilitiesBuilder::with_known` to add them.
- **Private network access**: `KnownCapability::PrivateNetworkClientServer` mirrors the manifest
  `privateNetworkClientServer` capability for LAN scenarios.
- **LPAC defaults**: Call `SecurityCapabilitiesBuilder::with_lpac_defaults()` to append the
  documented LPAC capability pair (`registryRead`, `lpacCom`) and flip the `lpac` flag. These cover
  common LPAC APIs that fail without the extra privileges.
- **Named capabilities**: For Microsoft-defined capability strings that are not yet wrapped in
  `KnownCapability`, call `SecurityCapabilitiesBuilder::with_named(&["capabilityName"])`. The builder
  will derive the SID at launch time.
- **Migration from appxmanifest**: Translate each `<Capability Name="..."/>` entry to the matching
  known capability or pass the literal string to `with_named`. Loopback exemptions previously
  declared via manifest are handled at runtime with `net::add_loopback_exemption` when the `net`
  feature is enabled.

## Toolchain prerequisites (Windows hosts)
- Install the **Windows 10/11 SDK** (10.0.19041 or newer). The SDK provides headers and import
  libraries consumed by the `windows` crate when linking against Win32 functions such as
  `DeriveCapabilitySidsFromName`.
- Ensure the **MSVC build tools** (Visual Studio Build Tools 17.x or newer) are present. `cargo`
  will invoke `link.exe` from this toolchain when compiling with the default `stable-x86_64-pc-windows-msvc`
  target.
- Ship artefacts with the **Microsoft Visual C++ Redistributable** (same major version as your
  build tools) or document it as a prerequisite for end users running binaries produced from this
  crate.
- When packaging signed binaries, confirm the host includes the required certificate toolchain
  (e.g. `signtool.exe`) and that the Windows SDK path is available in `PATH` during CI builds.

## Minimum Windows requirements
- AppContainer: Windows 8+
- LPAC: Windows 10 **1703** or later

## Quick start
```bash
# Library build (examples are separate)
cargo build
# Build and run the CLI example
cargo build --examples
cargo run --example acrun -- --help
```

## Examples
```rust
use rappct::{AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder, launch_in_container, LaunchOptions};

# fn main() -> rappct::Result<()> {
let prof = AppContainerProfile::ensure("demo.rappct", "Demo", Some("rappct example"))?;
let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
    .with_known(&[KnownCapability::InternetClient])?
    .build()?;
let child = launch_in_container(&caps, &LaunchOptions::default())?;
println!("PID: {}", child.pid);
# Ok(()) }
```

## Tracing
- Enable feature: build/test with `--features tracing`.
- Initialize a subscriber in your binary/tests to view logs:
  - Add dependency: `tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }`
  - Init early in `main`/tests:
    ```rust
    tracing_subscriber::fmt()
        .with_env_filter("rappct=trace")
        .init();
    ```
- What’s logged (Windows):
  - Attribute list size/ptr, SECURITY_CAPABILITIES pointers/count
  - UpdateProcThreadAttribute calls (security/AAPolicy/handles) with sizes
  - CreateProcessW params (flags, inherit_handles, env size) and GetLastError on failures

### Launch with pipes and job guard (Windows)
```rust
#[cfg(windows)]
fn launch_demo() -> rappct::Result<()> {
    use std::io::Read;
    let p = AppContainerProfile::ensure("demo.rappct", "Demo", None)?;
    let caps = SecurityCapabilitiesBuilder::new(&p.sid)
        .with_known(&[KnownCapability::InternetClient])?
        .with_lpac_defaults()? // optional
        .build()?;
    let opts = LaunchOptions { exe: "C:/Windows/System32/cmd.exe".into(), cmdline: Some(" /C echo hello".into()), stdio: rappct::StdioConfig::Pipe, join_job: Some(rappct::JobLimits{ memory_bytes: Some(32*1024*1024), cpu_rate_percent: None, kill_on_job_close: true }), ..Default::default() };
    let child = rappct::launch_in_container_with_io(&caps, &opts)?;
    let mut s = String::new();
    child.stdout.as_ref().unwrap().try_clone()?.read_to_string(&mut s)?;
    assert!(s.to_lowercase().contains("hello"));
    // Dropping child.job_guard (if present) will terminate the process if still running.
    Ok(())
}
```

### ACL grant
```rust
use rappct::{acl::{ResourcePath, AccessMask}, AppContainerProfile};
let p = AppContainerProfile::ensure("demo.rappct", "Demo", None)?;
// Allow read to a file for this package SID
rappct::acl::grant_to_package(ResourcePath::File("C:/temp/data.txt".into()), &p.sid, AccessMask(0x120089))?;
```

### Loopback exemption (debug-only)
```rust
// Feature `net` required
#[cfg(all(windows, feature = "net"))]
{
    use rappct::net::{add_loopback_exemption, LoopbackAdd};
    add_loopback_exemption(LoopbackAdd(p.sid.clone()).confirm_debug_only())?;
}
```

### `whoami --json` contract
- `is_appcontainer`: `bool`, true when the effective token belongs to any AppContainer.
- `is_lpac`: `bool`, true when the token originated from an LPAC profile.
- `package_sid`: `Option<String>` containing the SDDL of the package SID when present; omitted
  (`null`) for non-AppContainer tokens.
- `capabilities`: `Vec<String>` with the SDDL form of derived capability SIDs. The array order
  matches what the token enumeration API returns.

## Safety and security
- LPAC is **off by default**; you must opt in.
- **Localhost** is blocked by design. Loopback exemptions are provided for **debug only**.
- No silent fallbacks: when a capability or OS requirement is missing, fail with a clear error.
- Call `supports_lpac()` to ensure the OS is Windows 10 1703+ before opting in to LPAC.

## Repo layout
- `src/` core library modules
- `examples/acrun.rs` developer CLI
- `tests/` integration tests (gated by OS/features)
- `TODO.md` prioritized work
- `RULES.md` engineering rules
- `PROMPT.md` agent kickoff prompt

## License
MIT


