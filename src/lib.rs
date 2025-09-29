//! rappct — Rust AppContainer / LPAC toolkit (Windows)
//!
//! Windows implementations for AppContainer profiles, capabilities, secure process launch (AC/LPAC),
//! token introspection, ACLs, optional network isolation helpers, and diagnostics.
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
pub mod token;
pub mod util;

// Re-exports
pub use capability::{KnownCapability, SecurityCapabilities, SecurityCapabilitiesBuilder};
pub use launch::{launch_in_container, JobLimits, LaunchOptions, Launched, StdioConfig};
#[cfg(windows)]
pub use launch::{launch_in_container_with_io, LaunchedIo};
pub use profile::{derive_sid_from_name, AppContainerProfile};

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
        struct OSVERSIONINFOW {
            size: u32,
            major: u32,
            minor: u32,
            build: u32,
            platform: u32,
            csd: [u16; 128],
        }
        #[link(name = "ntdll")]
        extern "system" {
            fn RtlGetVersion(info: *mut OSVERSIONINFOW) -> i32;
        }
        unsafe {
            let mut v = OSVERSIONINFOW {
                size: std::mem::size_of::<OSVERSIONINFOW>() as u32,
                major: 0,
                minor: 0,
                build: 0,
                platform: 0,
                csd: [0u16; 128],
            };
            let st = RtlGetVersion(&mut v as *mut _);
            if st != 0 {
                return Err(AcError::UnsupportedLpac);
            }
            // Windows 10 build 15063 (1703) or later required
            if v.major < 10 {
                return Err(AcError::UnsupportedLpac);
            }
            if v.build < 15063 {
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
