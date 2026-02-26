//! rappct — Rust AppContainer / LPAC toolkit (Windows)
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//!
//! Windows implementations for AppContainer profiles, capabilities, secure process launch (AC/LPAC),
//! token introspection, ACLs, optional network isolation helpers, and diagnostics.
//!
//! MSRV: Rust 1.90. See README for tooling/CI details.
//!
//! Tour:
//! - Capabilities: derive SIDs by known or named capability strings.
//! - Launch: start AC/LPAC processes with optional I/O pipes and job limits.
//! - Diagnostics: feature `introspection` surfaces helpful configuration warnings.
//! - Network helpers: feature `net` for enumeration and loopback RAII guard.
//!
//! Capability catalog: see `docs/capabilities.md` in the repository for common capability SIDs
//! and starter sets, plus links to Microsoft documentation.
//!
//! Quick example: launch with pipes and job limits
//!
//! ```no_run
//! use rappct::{
//!     AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
//!     launch::LaunchOptions, launch::StdioConfig, launch::JobLimits,
//!     launch_in_container,
//! };
//! # fn main() -> rappct::Result<()> {
//! let profile = AppContainerProfile::ensure("rappct.sample", "rappct", Some("demo"))?;
//! let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
//!     .with_known(&[KnownCapability::InternetClient])
//!     .build()?;
//! let opts = LaunchOptions {
//!     exe: "C:/Windows/System32/cmd.exe".into(),
//!     cmdline: Some(" /C echo hello".into()),
//!     stdio: StdioConfig::Pipe,
//!     join_job: Some(JobLimits { memory_bytes: Some(32 * 1024 * 1024), cpu_rate_percent: None, kill_on_job_close: true }),
//!     ..Default::default()
//! };
//! let child = launch_in_container(&caps, &opts)?;
//! # let _ = child.pid; profile.delete()?; Ok(()) }
//! ```
//!
//! Testing note: in CI or local tests you can force LPAC support detection via the
//! `RAPPCT_TEST_LPAC_STATUS` environment variable (`ok` or `unsupported`).
//!
//! Refer to `CONTRIBUTING.md` for engineering conventions and contribution guidance.

mod error;
pub use error::{AcError, Result};

pub mod acl;
pub mod capability;
#[cfg(feature = "introspection")]
pub mod diag;
pub mod launch;
#[cfg(feature = "net")]
pub mod net;
pub mod profile;
pub mod sid;
#[cfg(windows)]
#[doc(hidden)]
pub mod test_support;
pub mod token;
pub mod util;
// Internal FFI safety helpers (crate-private)
pub(crate) mod ffi;

// Re-exports
pub use capability::{
    Capability, CapabilityCatalog, CapabilityName, KnownCapability, SecurityCapabilities,
    SecurityCapabilitiesBuilder, UseCase, WELL_KNOWN_CAPABILITY_NAMES,
};
pub use launch::{JobLimits, LaunchOptions, Launched, StdioConfig, launch_in_container};
#[cfg(windows)]
pub use launch::{LaunchedIo, launch_in_container_with_io};
pub use profile::{AppContainerProfile, derive_sid_from_name};
pub use sid::AppContainerSid;

/// Returns Ok(()) if LPAC is supported on this OS (Windows 10 1703+).
pub fn supports_lpac() -> Result<()> {
    #[cfg(windows)]
    {
        // Test/CI override: allow forcing LPAC support status
        if let Ok(val) = std::env::var("RAPPCT_TEST_LPAC_STATUS") {
            match val.as_str() {
                "ok" => return Ok(()),
                "unsupported" => return Err(AcError::UnsupportedLpac),
                _ => {}
            }
        }
        // Use ntdll!RtlGetVersion to query build number reliably
        #[repr(C)]
        struct OsVersionInfoW {
            size: u32,
            major: u32,
            minor: u32,
            build: u32,
            platform: u32,
            csd: [u16; 128],
        }
        #[link(name = "ntdll")]
        unsafe extern "system" {
            fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
        }
        // SAFETY: Calls a documented OS function (`RtlGetVersion`) with a valid, writable
        // pointer to our stack-allocated struct and reads the returned fields.
        // No aliasing beyond this call; struct is fully initialized before use.
        unsafe {
            let mut v = OsVersionInfoW {
                size: std::mem::size_of::<OsVersionInfoW>() as u32,
                major: 0,
                minor: 0,
                build: 0,
                platform: 0,
                csd: [0u16; 128],
            };
            let st = RtlGetVersion(&mut v as *mut _);
            if st != 0 {
                // Defensive return for `RtlGetVersion` failures.
                // This branch is not practically reachable in normal test hosts and would
                // require either OS-level fault injection of a core Win32 API failure or a
                // severely misconfigured runtime to observe.
                // We keep it explicit to preserve fail-closed behavior if version detection
                // cannot be completed at startup.
                return Err(AcError::UnsupportedLpac);
            }
            // Windows 10 build 15063 (1703) or later required
            if v.major < 10 {
                // Older Windows major versions are intentionally unsupported.
                // This keeps LPAC detection deterministic on unsupported environments
                // and is documented as a defensive boundary for deployment targets.
                return Err(AcError::UnsupportedLpac);
            }
            if v.build < 15063 {
                // Windows 10 build 15063 (1703) is the minimum supported LPAC baseline.
                // This check is platform-constraint dependent and cannot be driven safely
                // from normal crate tests without introducing mutable global shims.
                return Err(AcError::UnsupportedLpac);
            }
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(test)]
#[cfg(not(windows))]
mod tests {
    use super::*;

    #[test]
    fn supports_lpac_returns_unsupported_platform_on_non_windows() {
        let r = supports_lpac();
        assert!(r.is_err());
        assert!(matches!(r.unwrap_err(), AcError::UnsupportedPlatform));
    }
}
