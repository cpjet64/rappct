# API Reference

See also: [Architecture](./ARCHITECTURE.md)

This document summarizes the current public API from `src/lib.rs` and public modules.

## Feature and Platform Gating

| Surface | Gate |
| --- | --- |
| `rappct::profile`, `capability`, `launch`, `token`, `acl`, `sid`, `util` | Always compiled (many operations return `UnsupportedPlatform` on non-Windows) |
| `rappct::diag` | `feature = "introspection"` |
| `rappct::net` | `feature = "net"` |
| crate-root re-export: `LaunchedIo`, `launch_in_container_with_io` | `#[cfg(windows)]` |
| `supports_lpac()` | Always available; returns `UnsupportedPlatform` on non-Windows |

## Crate Root Re-exports

Common imports available from `rappct` directly:

```rust
use rappct::{
    AcError, Result,
    AppContainerProfile, AppContainerSid, derive_sid_from_name,
    Capability, CapabilityCatalog, CapabilityName, KnownCapability,
    SecurityCapabilities, SecurityCapabilitiesBuilder, UseCase,
    JobLimits, LaunchOptions, Launched, StdioConfig, launch_in_container,
};
```

On Windows crate root also re-exports:

```rust
#[cfg(windows)]
use rappct::{LaunchedIo, launch_in_container_with_io};
```

## Module: `profile`

Public items:

- `AppContainerProfile { name: String, sid: AppContainerSid }`
- `AppContainerProfile::ensure(name, display, description)`
- `AppContainerProfile::open(name)`
- `AppContainerProfile::delete(self)`
- `AppContainerProfile::folder_path(&self)`
- `AppContainerProfile::named_object_path(&self)`
- `derive_sid_from_name(name)`

Typical sequence:

```rust,no_run
use rappct::{AppContainerProfile, Result};

fn main() -> Result<()> {
    let profile = AppContainerProfile::ensure("rappct.sample", "rappct", Some("demo"))?;
    let _folder = profile.folder_path()?;
    let _named_objects = profile.named_object_path()?;
    profile.delete()?;
    Ok(())
}
```

## Module: `capability`

Public items:

- Capability naming/types: `CapabilityName` (`KnownCapability` alias), `Capability`, `CapabilityCatalog`
- Constants/helpers: `WELL_KNOWN_CAPABILITY_NAMES`, `known_caps_to_named`, `derive_named_capability_sids`
- Launch payload types: `SecurityCapabilities`, `SecurityCapabilitiesBuilder`
- Presets: `UseCase`, `UseCaseCapabilities`

Common builder sequence:

```rust,no_run
use rappct::{AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder, Result};

fn main() -> Result<()> {
    let profile = AppContainerProfile::ensure("rappct.cap", "cap", None)?;
    let sec = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults()
        .build()?;
    let _lpac_enabled = sec.lpac;
    profile.delete()?;
    Ok(())
}
```

Use-case preset sequence:

```rust,no_run
use rappct::{AppContainerProfile, SecurityCapabilitiesBuilder, UseCase, Result};

fn main() -> Result<()> {
    let profile = AppContainerProfile::ensure("rappct.uc", "usecase", None)?;
    let sec = SecurityCapabilitiesBuilder::from_use_case(UseCase::MinimalLpac)
        .with_profile_sid(&profile.sid)
        .build()?;
    let _ = sec.caps.len();
    profile.delete()?;
    Ok(())
}
```

## Module: `launch`

Public items:

- `StdioConfig` (`Inherit`, `Null`, `Pipe`)
- `JobLimits { memory_bytes, cpu_rate_percent, kill_on_job_close }`
- `LaunchOptions { exe, cmdline, cwd, env, stdio, suspended, join_job, startup_timeout, .. }`
- `Launched { pid }`
- `launch_in_container(&SecurityCapabilities, &LaunchOptions)`
- `merge_parent_env(Vec<(OsString, OsString)>)`
- `launch_in_container_with_io(...)` (available from module on all platforms; returns unsupported on non-Windows)
- `LaunchedIo::wait(timeout)` and `JobGuard::as_handle()` (Windows)
- `JobObjectDropGuard` (Windows)

Typical launch with pipes:

```rust,no_run
use rappct::{
    AppContainerProfile, KnownCapability, LaunchOptions,
    SecurityCapabilitiesBuilder, StdioConfig, launch_in_container, Result,
};

fn main() -> Result<()> {
    let profile = AppContainerProfile::ensure("rappct.launch", "launch", None)?;
    let sec = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;

    let opts = LaunchOptions {
        exe: "C:/Windows/System32/cmd.exe".into(),
        cmdline: Some(" /C echo hello".into()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    };

    let child = launch_in_container(&sec, &opts)?;
    let _pid = child.pid;

    profile.delete()?;
    Ok(())
}
```

Environment merge helper sequence:

```rust
use std::ffi::OsString;
use rappct::launch::merge_parent_env;

let merged = merge_parent_env(vec![(OsString::from("RAPPCT_X"), OsString::from("1"))]);
assert!(merged.iter().any(|(k, _)| k == "RAPPCT_X"));
```

## Module: `token`

Public items:

- `TokenInfo { is_appcontainer, is_lpac, package_sid, capability_sids }`
- `query_current_process_token()`

Typical sequence:

```rust,no_run
use rappct::{token::query_current_process_token, Result};

fn main() -> Result<()> {
    let info = query_current_process_token()?;
    let _ = (info.is_appcontainer, info.is_lpac, info.package_sid);
    Ok(())
}
```

## Module: `acl`

Public items:

- `AceInheritance` and constants:
  - `SUB_CONTAINERS_AND_OBJECTS`, `SUB_CONTAINERS_ONLY`, `OBJECTS_ONLY`, `NONE`
- `ResourcePath`:
  - `File(PathBuf)`, `Directory(PathBuf)`, `DirectoryCustom(PathBuf, AceInheritance)`, `RegistryKey(String)`
- `AccessMask` and constants:
  - `GENERIC_ALL`, `FILE_GENERIC_READ`, `FILE_GENERIC_WRITE`
- `grant_to_package(target, &AppContainerSid, AccessMask)`
- `grant_to_capability(target, capability_sid_sddl, AccessMask)`

Typical sequence:

```rust,no_run
use rappct::{
    AccessMask, AppContainerProfile,
    acl::{ResourcePath, grant_to_package},
    Result,
};

fn main() -> Result<()> {
    let profile = AppContainerProfile::ensure("rappct.acl", "acl", None)?;
    let dir = std::path::PathBuf::from("C:/temp/rappct-acl");
    std::fs::create_dir_all(&dir).ok();

    grant_to_package(
        ResourcePath::Directory(dir),
        &profile.sid,
        AccessMask::FILE_GENERIC_READ,
    )?;

    profile.delete()?;
    Ok(())
}
```

## Module: `sid`

Public items:

- `AppContainerSid`
  - `from_sddl(...)` (unchecked)
  - `try_from_sddl(...) -> Result<_>` (validates AppContainer SID format)
  - `as_string()`
- `SidAndAttributes { sid_sddl, attributes }`

Typical sequence:

```rust
use rappct::AppContainerSid;

let sid = AppContainerSid::try_from_sddl("S-1-15-2-1").unwrap();
assert_eq!(sid.as_string(), "S-1-15-2-1");
```

## Module: `diag` (`feature = "introspection"`)

Public items:

- `ConfigWarning` (`LpacWithoutCommonCaps`, `NoNetworkCaps`)
- `validate_configuration(&SecurityCapabilities, &LaunchOptions)`

Typical sequence:

```rust,no_run
#[cfg(feature = "introspection")]
fn validate(sec: &rappct::SecurityCapabilities, opts: &rappct::LaunchOptions) {
    let warnings = rappct::diag::validate_configuration(sec, opts);
    for w in warnings {
        println!("{:?}", w);
    }
}
```

## Module: `net` (`feature = "net"`)

Public items:

- `list_appcontainers()`
- `LoopbackAdd(AppContainerSid)` + `confirm_debug_only()`
- `add_loopback_exemption(LoopbackAdd)`
- `remove_loopback_exemption(&AppContainerSid)`
- `LoopbackExemptionGuard::new(&AppContainerSid)` and `disable()`

Typical safe sequence:

```rust,no_run
#[cfg(feature = "net")]
fn main() -> rappct::Result<()> {
    use rappct::{AppContainerProfile, net};

    let profile = AppContainerProfile::ensure("rappct.net", "net", None)?;

    net::add_loopback_exemption(
        net::LoopbackAdd(profile.sid.clone()).confirm_debug_only()
    )?;

    net::remove_loopback_exemption(&profile.sid)?;
    profile.delete()?;
    Ok(())
}
```

## Module: `util`

Public items:

- UTF-16 helpers: `to_utf16(&str)`, `to_utf16_os(&OsStr)`
- On Windows, deprecated wrappers are still exported:
  - `OwnedHandle`, `LocalFreeGuard<T>`, `FreeSidGuard`

`util` is primarily compatibility/interop support; new internal code paths use `crate::ffi` wrappers instead.

## Root Helper Function

- `supports_lpac() -> Result<()>`

Behavior:

- Windows: checks OS build via `RtlGetVersion` and requires build `>= 15063`.
- Non-Windows: returns `AcError::UnsupportedPlatform`.
- Test override: environment variable `RAPPCT_TEST_LPAC_STATUS=ok|unsupported`.
