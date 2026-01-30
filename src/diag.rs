//! Diagnostics and configuration validation (skeleton). Feature: `introspection`
#![allow(clippy::undocumented_unsafe_blocks)]

use crate::capability::{SecurityCapabilities, derive_named_capability_sids};
use crate::launch::LaunchOptions;

/// A diagnostic warning about potentially misconfigured security capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigWarning {
    /// LPAC mode is enabled but common LPAC defaults (`registryRead`, `lpacCom`) are missing.
    LpacWithoutCommonCaps,
    /// No networking capabilities (`internetClient`, etc.) are present.
    NoNetworkCaps,
}

/// Validates [`SecurityCapabilities`] and [`LaunchOptions`] for common misconfigurations.
pub fn validate_configuration(
    sec: &SecurityCapabilities,
    opts: &LaunchOptions,
) -> Vec<ConfigWarning> {
    let mut out = Vec::new();
    let _ = opts;
    // LPAC: advise enabling common defaults if not present
    if sec.lpac {
        // registryRead + lpacCom
        if let Ok(required) = derive_named_capability_sids(&["registryRead", "lpacCom"]) {
            let have = &sec.caps;
            let mut missing = 0;
            for r in required {
                if !have.iter().any(|h| h.sid_sddl == r.sid_sddl) {
                    missing += 1;
                }
            }
            if missing > 0 {
                out.push(ConfigWarning::LpacWithoutCommonCaps);
            }
        }
    }
    // Network caps: warn if none of the known networking caps are present
    if let Ok(net_caps) = derive_named_capability_sids(&[
        "internetClient",
        "internetClientServer",
        "privateNetworkClientServer",
    ]) {
        let any = sec
            .caps
            .iter()
            .any(|h| net_caps.iter().any(|n| n.sid_sddl == h.sid_sddl));
        if !any {
            out.push(ConfigWarning::NoNetworkCaps);
        }
    }
    out
}
