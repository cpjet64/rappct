//! rappct — Rust AppContainer / LPAC toolkit (Windows)
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//!
//! Windows implementations for AppContainer profiles, capabilities, secure process launch (AC/LPAC),
//! token introspection, ACLs, optional network isolation helpers, and diagnostics.
//!
//! MSRV: Rust 1.88. See README for tooling/CI details.
//!
//! Tour:
//! - Capabilities: derive SIDs by known or named capability strings.
//! - Launch: start AC/LPAC processes with optional I/O pipes and job limits.
//! - Diagnostics: feature `introspection` surfaces helpful configuration warnings.
//! - Network helpers: feature `net` for enumeration and loopback RAII guard.
//!
//! Capability catalog: see `docs/modules/capability.md` in the repository for common capability SIDs
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
//! Refer to `CONTRIBUTING.md` for engineering conventions and contribution guidance.

mod error;
pub use error::{AcError, Result};
mod lpac;

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
pub use lpac::supports_lpac;
pub use profile::{AppContainerProfile, derive_sid_from_name};
pub use sid::AppContainerSid;
