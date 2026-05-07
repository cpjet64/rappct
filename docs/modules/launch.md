# Launch Module (`src/launch/mod.rs`)

## Purpose

Launches child processes inside AppContainer/LPAC security boundaries with configurable stdio, environment, and optional job object limits.

## Key Types and Functions

- `LaunchOptions`
- `StdioConfig`
- `JobLimits`
- `Launched`
- `LaunchedIo` (Windows)
- `launch_in_container(...)`
- `launch_in_container_with_io(...)` (Windows)

## Responsibilities

- Build `STARTUPINFOEX` security attributes for containerized process creation.
- Manage handle inheritance and stdio pipeline behavior.
- Optionally attach job-object constraints and kill-on-close semantics.

## Typical Flow

```rust
use rappct::{launch_in_container, LaunchOptions, StdioConfig};

fn run(sec: &rappct::SecurityCapabilities) -> rappct::Result<u32> {
    let opts = LaunchOptions {
        exe: "C:/Windows/System32/cmd.exe".into(),
        cmdline: Some(" /C echo rappct".into()),
        stdio: StdioConfig::Null,
        ..Default::default()
    };
    let child = launch_in_container(sec, &opts)?;
    Ok(child.pid)
}
```

## Related Docs

- [Capability Module](./capability.md)
- [ACL Module](./acl.md)
- [Rustdoc: launch module](../../target/doc/rappct/launch/index.html)
