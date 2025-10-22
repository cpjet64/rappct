# rappct

**Stable (main branch)**

[![CI stable](https://img.shields.io/github/actions/workflow/status/cpjet64/rappct/ci.yml?branch=main&label=CI%20stable)](https://github.com/cpjet64/rappct/actions/workflows/ci.yml?query=branch%3Amain)
[![CI 1.90.0](https://img.shields.io/github/actions/workflow/status/cpjet64/rappct/ci.yml?branch=main&label=CI%201.90.0)](https://github.com/cpjet64/rappct/actions/workflows/ci.yml?query=branch%3Amain)
[![CI (beta)](https://img.shields.io/github/actions/workflow/status/cpjet64/rappct/ci.yml?branch=main&label=CI%20beta)](https://github.com/cpjet64/rappct/actions/workflows/ci.yml?query=branch%3Amain)
[![CI (nightly)](https://img.shields.io/github/actions/workflow/status/cpjet64/rappct/ci.yml?branch=main&label=CI%20nightly)](https://github.com/cpjet64/rappct/actions/workflows/ci.yml?query=branch%3Amain)
[![Release](https://img.shields.io/github/v/release/cpjet64/rappct)](https://github.com/cpjet64/rappct/releases?q=exclude-prereleases%3Atrue)
[![Crates.io](https://img.shields.io/crates/v/rappct)](https://crates.io/crates/rappct)
[![Downloads](https://img.shields.io/crates/d/rappct)](https://crates.io/crates/rappct)


> Rust toolkit for working with Windows AppContainer (AC) and Low Privilege AppContainer (LPAC) security boundaries.

rappct packages the underlying Windows APIs into a cohesive crate so that you can create, manage, and launch
AppContainer-aware workloads from Rust with minimal boilerplate. It is designed for security-sensitive automation
that needs to compose profiles, capabilities, process launches, ACL helpers, and diagnostics in one place.

- **Status**: Actively developed. Windows paths are implemented first; non-Windows hosts return `UnsupportedPlatform`.
- **MSRV**: Rust 1.90 (stable).

## Highlights

- AppContainer profile lifecycle helpers (create, open, delete) and profile path resolution.
- Capability derivation via `DeriveCapabilitySidsFromName`, with ergonomic builders for known and custom capability SIDs.
- Secure process launch helpers (AC/LPAC) with `STARTUPINFOEX`, optional job object integration, and stdio redirection.
- Token inspection helpers to understand the effective AppContainer/LPAC context at runtime.
- Optional modules for diagnostics (`introspection`) and network loopback management (`net`).
- ACL utilities to grant/revoke filesystem and registry access for package SIDs.

## What To Expect

- Network isolation by default; use capability sets and optional loopback exemption for localhost-only testing.
- DNS/HTTP behavior varies by capability and Windows version; LPAC is stricter.
- File system and registry are isolated; grant specific paths/keys via ACL helpers when needed.
- Examples and docs show cleanup patterns to avoid lingering policy changes.

## Release & Branching

- Single-branch flow. Pushes to `main` run CI; when green, the release workflow cuts a GitHub release and publishes to crates.io automatically.

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| Windows 10 1703+ | LPAC support requires at least Windows 10 1703. AppContainer APIs are available on Windows 8+. |
| Windows SDK 10.0.19041+ | Required so the `windows` crate can link against the necessary Win32 symbols. |
| MSVC build tools 17.x+ | `cargo` uses the MSVC linker when targeting `x86_64-pc-windows-msvc`. |
| Rust toolchain | Install via [rustup](https://rustup.rs). Run `rustup target add x86_64-pc-windows-msvc` if needed. |

## Getting Started

```bash
# Clone the repository
git clone https://github.com/cpjet64/rappct.git
cd rappct

# Build the library
cargo build

# Run the example CLI
cargo run --example acrun -- --help
```

The crate is structured as a binary-agnostic library. Add it to your project:

```bash
# Stable release from crates.io (recommended)
cargo add rappct

# Pin a specific version (optional)
# See the latest on crates.io or the release badge
cargo add rappct@<x.y.z>

# Development version from git (optional)
cargo add rappct --git https://github.com/cpjet64/rappct.git --branch main
```

## Usage Snapshot

```rust
use rappct::{AppContainerProfile, KnownCapability, LaunchOptions, SecurityCapabilitiesBuilder, launch_in_container};

fn main() -> rappct::Result<()> {
    let profile = AppContainerProfile::ensure("demo.rappct", "Demo", Some("rappct example"))?;
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults() // opt in to LPAC defaults when required
        .build()?;

    let opts = LaunchOptions { exe: "C:/Windows/System32/notepad.exe".into(), ..Default::default() };
    let child = launch_in_container(&caps, &opts)?;
    println!("child pid: {}", child.pid);
    Ok(())
}
```

## Examples

The `examples/` directory contains runnable demonstrations of rappct features:

### [rappct_demo.rs](examples/rappct_demo.rs)

Simple demonstration of essential features:

- Creating AppContainer profiles
- Launching sandboxed processes
- Granting specific capabilities
- Automatic network configuration (with `net` feature)

```bash
cargo run --example rappct_demo --all-features
```

### [comprehensive_demo.rs](examples/comprehensive_demo.rs)

Comprehensive demonstrations with isolated examples for each capability:

- Individual demos for filesystem, registry, network, and COM capabilities
- PowerShell in AppContainers (output redirection pattern)
- Combined multi-capability example
- Best for understanding each feature in isolation

```bash
cargo run --example comprehensive_demo --all-features
```

### [advanced_features.rs](examples/advanced_features.rs)

Advanced and less common features:

- Profile path resolution (folder_path, named_object_path)
- Custom named capabilities
- Configuration diagnostics
- Advanced launch options with custom environment variables
- Network enumeration
- Direct SID derivation

```bash
cargo run --example advanced_features --all-features
```

### [network_demo.rs](examples/network_demo.rs)

Network capability demonstration with automatic firewall configuration:

- Built-in firewall loopback exemption functionality
- PowerShell network testing in AppContainers
- Automatic cleanup patterns

```bash
cargo run --example network_demo --features net
```

### [acrun.rs](examples/acrun.rs)

Developer CLI tool for managing AppContainer profiles and launching sandboxed processes:

```bash
# Create a profile
cargo run --example acrun -- ensure demo.app

# Launch a process in an AppContainer
cargo run --example acrun -- launch demo.app notepad.exe

# View help for all commands
cargo run --example acrun -- --help
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `net` | Enable AppContainer enumeration and firewall loopback exemption helpers.<br><br>⚠️ This feature changes global Windows Firewall state. Always call `LoopbackAdd::confirm_debug_only()` before `add_loopback_exemption`, and use `remove_loopback_exemption` to restore the original configuration when finished. |
| `introspection` | Toggle diagnostics, configuration validation, and capability suggestions. |
| `tracing` | Emit structured tracing spans/logs; integrate with `tracing-subscriber`. |
| `serde` | Enable Serialize/Deserialize support for core types (SecurityCapabilities, AppContainerSid, SidAndAttributes). Useful for config files or JSON APIs. |

Disable unused features for the leanest runtime surface; APIs gracefully return `AcError::Unimplemented` when a
feature is not compiled in.

## Diagnostics & Security Considerations

- LPAC capabilities are **opt-in**; call `SecurityCapabilitiesBuilder::with_lpac_defaults()` explicitly.
- Loopback exemptions via the `net` feature are meant for **debug scenarios only**. Production use should rely on
  standard firewall policy.
- When something fails due to missing capabilities or OS prerequisites, rappct surfaces detailed error messages instead
  of falling back silently. Use `supports_lpac()` to guard LPAC-specific code paths.
  For tests/CI, you can set `RAPPCT_TEST_LPAC_STATUS=ok|unsupported` to force detection.

See also: docs/capabilities.md for common capability SIDs and starter sets.

## Repository Layout

- `src/` &mdash; core library modules (capabilities, launch, ACLs, diagnostics).
- `examples/` &mdash; runnable samples such as `acrun` for quick CLI exploration.
- `tests/` &mdash; integration tests covering launch/ACL/token behaviours on Windows.

## Development Workflow

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
```

Run Windows-specific scenarios in an elevated PowerShell session when the tests require loopback exemptions or ACL
adjustments.

## Contributing

Contributions are welcome! Please:

1. Open an issue using the provided template before starting major work.
2. Discuss API-affecting changes early to avoid churn.
3. Include tests and documentation updates alongside code changes.
4. Run the checks listed in the PR template before submitting.

See `CONTRIBUTING.md` for style and review guidelines.

## Security

Please report vulnerabilities privately through the [GitHub Security Advisory workflow](https://github.com/cpjet64/rappct/security/policy).

## License

This project is licensed under the [MIT license](LICENSE).
