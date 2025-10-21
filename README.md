# rappct

**Stable (main branch)**

[![CI](https://github.com/cpjet64/rappct/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cpjet64/rappct/actions/workflows/ci.yml?query=branch%3Amain)
[![Release](https://img.shields.io/github/v/release/cpjet64/rappct)](https://github.com/cpjet64/rappct/releases)
[![Crates.io](https://img.shields.io/crates/v/rappct)](https://crates.io/crates/rappct)
[![Downloads](https://img.shields.io/crates/d/rappct)](https://crates.io/crates/rappct)

**Development (dev branch)**

[![CI](https://github.com/cpjet64/rappct/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/cpjet64/rappct/actions/workflows/ci.yml?query=branch%3Adev)
[![Dev Release](https://img.shields.io/github/v/release/cpjet64/rappct?include_prereleases&filter=*-dev.*&label=dev%20release)](https://github.com/cpjet64/rappct/releases?q=prerelease%3Atrue)

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

The crate is structured as a binary-agnostic library. Add it to your project from crates.io:

```bash
cargo add rappct
```

Or use the latest development version from git:

```bash
cargo add rappct --git https://github.com/cpjet64/rappct.git
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

### Feature Flags

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
