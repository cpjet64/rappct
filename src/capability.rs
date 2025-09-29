//! Capability mapping and builders.

use crate::sid::{AppContainerSid, SidAndAttributes};
#[cfg(windows)]
use crate::util::LocalFreeGuard;
use crate::{AcError, Result};

// windows::Win32::Security::SE_GROUP_ENABLED is not consistently available across crate versions.
// Use the documented value directly.
#[cfg(windows)]
const SE_GROUP_ENABLED_CONST: u32 = 0x0000_0004;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum KnownCapability {
    InternetClient,
    InternetClientServer,
    PrivateNetworkClientServer,
}

fn known_to_name(cap: KnownCapability) -> &'static str {
    match cap {
        KnownCapability::InternetClient => "internetClient",
        KnownCapability::InternetClientServer => "internetClientServer",
        KnownCapability::PrivateNetworkClientServer => "privateNetworkClientServer",
    }
}

pub fn known_caps_to_named(caps: &[KnownCapability]) -> Vec<&'static str> {
    caps.iter().map(|c| known_to_name(*c)).collect()
}

// Static list of known/supported capability names for suggestions.
#[cfg(feature = "introspection")]
static KNOWN_CAP_NAMES: &[&str] = &[
    "internetClient",
    "internetClientServer",
    "privateNetworkClientServer",
    // LPAC common
    "registryRead",
    "lpacCom",
];

#[cfg(feature = "introspection")]
fn suggest_capability_name(name: &str) -> Option<&'static str> {
    let mut best = 0.0f64;
    let mut suggestion = None;
    for &candidate in KNOWN_CAP_NAMES {
        let score = strsim::jaro_winkler(name, candidate);
        if score > best {
            best = score;
            suggestion = Some(candidate);
        }
    }
    if best < 0.80 {
        None
    } else {
        suggestion
    }
}

/// Derive capability SIDs from names. TODO: implement via DeriveCapabilitySidsFromName (LocalFree).
pub fn derive_named_capability_sids(names: &[&str]) -> Result<Vec<SidAndAttributes>> {
    if names.is_empty() {
        return Ok(vec![]);
    }
    #[cfg(windows)]
    {
        use windows::core::{PCWSTR, PWSTR};
        use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
        // Some toolchains don't surface DeriveCapabilitySidsFromName via windows-rs; bind manually.
        #[link(name = "Userenv")]
        extern "system" {
            fn DeriveCapabilitySidsFromName(
                CapName: PCWSTR,
                CapGroupSids: *mut *mut *mut core::ffi::c_void,
                CapGroupSidCount: *mut u32,
                CapabilitySids: *mut *mut *mut core::ffi::c_void,
                CapabilitySidCount: *mut u32,
            ) -> i32;
        }
        let mut out: Vec<SidAndAttributes> = Vec::new();
        unsafe {
            for &name in names {
                #[cfg(feature = "tracing")]
                tracing::trace!("derive_named_capability_sids: name={}", name);
                let wide: Vec<u16> = crate::util::to_utf16(name);
                let mut group_sids: *mut *mut std::ffi::c_void = std::ptr::null_mut();
                let mut group_count: u32 = 0;
                let mut cap_sids: *mut *mut std::ffi::c_void = std::ptr::null_mut();
                let mut cap_count: u32 = 0;
                let ok = DeriveCapabilitySidsFromName(
                    PCWSTR(wide.as_ptr()),
                    &mut group_sids as *mut _ as *mut _,
                    &mut group_count,
                    &mut cap_sids as *mut _ as *mut _,
                    &mut cap_count,
                );
                if ok == 0 {
                    #[cfg(feature = "tracing")]
                    {
                        use windows::Win32::Foundation::GetLastError;
                        let gle = GetLastError().0;
                        tracing::error!(
                            "DeriveCapabilitySidsFromName failed: name={}, GLE={}",
                            name,
                            gle
                        );
                    }
                    // Try to suggest a close capability name using strsim if available
                    #[cfg(feature = "introspection")]
                    let suggestion: Option<&'static str> = suggest_capability_name(name);
                    #[cfg(not(feature = "introspection"))]
                    let suggestion: Option<&'static str> = None;
                    let name_with_hint = match suggestion {
                        Some(s) => format!("{} (did you mean '{s}'?)", name),
                        None => name.to_string(),
                    };
                    return Err(AcError::UnknownCapability {
                        name: name_with_hint,
                        suggestion,
                    });
                }
                #[cfg(feature = "tracing")]
                tracing::trace!(
                    "DeriveCapabilitySidsFromName: name={}, group_count={}, cap_count={}",
                    name,
                    group_count,
                    cap_count
                );
                // Convert capability group SIDs (ignored for SECURITY_CAPABILITIES; free them)
                for i in 0..group_count as isize {
                    let sid_ptr = *group_sids.offset(i) as *mut std::ffi::c_void;
                    if !sid_ptr.is_null() {
                        let _guard = LocalFreeGuard::<std::ffi::c_void>::new(sid_ptr);
                    }
                }
                // Convert capability SIDs
                for i in 0..cap_count as isize {
                    let sid_ptr = *cap_sids.offset(i) as *mut std::ffi::c_void;
                    if !sid_ptr.is_null() {
                        let sid_guard = LocalFreeGuard::<std::ffi::c_void>::new(sid_ptr);
                        let mut sddl = PWSTR::null();
                        if ConvertSidToStringSidW(
                            windows::Win32::Security::PSID(sid_guard.as_ptr()),
                            &mut sddl,
                        )
                        .is_ok()
                        {
                            let sddl_guard = LocalFreeGuard::<u16>::new(sddl.0);
                            let s = sddl_guard.to_string_lossy();
                            out.push(SidAndAttributes {
                                sid_sddl: s,
                                attributes: SE_GROUP_ENABLED_CONST,
                            });
                        }
                    }
                }
                // Free arrays
                if !group_sids.is_null() {
                    let _ = LocalFreeGuard::<*mut std::ffi::c_void>::new(group_sids);
                }
                if !cap_sids.is_null() {
                    let _ = LocalFreeGuard::<*mut std::ffi::c_void>::new(cap_sids);
                }
            }
        }
        Ok(out)
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

pub struct SecurityCapabilities {
    pub package: AppContainerSid,
    pub caps: Vec<SidAndAttributes>,
    pub lpac: bool,
}

pub struct SecurityCapabilitiesBuilder {
    package: AppContainerSid,
    caps_named: Vec<String>,
    lpac: bool,
}

impl SecurityCapabilitiesBuilder {
    pub fn new(pkg: &AppContainerSid) -> Self {
        Self {
            package: pkg.clone(),
            caps_named: vec![],
            lpac: false,
        }
    }
    pub fn with_known(mut self, caps: &[KnownCapability]) -> Self {
        let names = known_caps_to_named(caps);
        self.caps_named
            .extend(names.into_iter().map(|s| s.to_string()));
        self
    }
    pub fn with_named(mut self, names: &[&str]) -> Self {
        if names.is_empty() {
            return self;
        }
        self.caps_named.extend(names.iter().map(|s| s.to_string()));
        self
    }
    /// Opinionated minimal LPAC defaults (skeleton). Add "registryRead", "lpacCom".
    pub fn with_lpac_defaults(mut self) -> Self {
        self.lpac = true;
        self.caps_named.push("registryRead".to_string());
        self.caps_named.push("lpacCom".to_string());
        self
    }
    pub fn lpac(mut self, enabled: bool) -> Self {
        self.lpac = enabled;
        self
    }
    pub fn build(self) -> Result<SecurityCapabilities> {
        let caps = derive_named_capability_sids(
            &self
                .caps_named
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )?;
        Ok(SecurityCapabilities {
            package: self.package,
            caps,
            lpac: self.lpac,
        })
    }

    #[cfg(test)]
    fn named_caps_for_test(&self) -> &[String] {
        &self.caps_named
    }

    #[cfg(test)]
    fn lpac_enabled_for_test(&self) -> bool {
        self.lpac
    }
}

#[cfg(all(test, feature = "introspection"))]
mod tests {
    use super::suggest_capability_name;

    #[test]
    fn suggests_known_capability_when_above_threshold() {
        let suggestion = suggest_capability_name("internetClientt");
        assert_eq!(suggestion, Some("internetClient"));
    }

    #[test]
    fn suppresses_suggestion_below_threshold() {
        assert_eq!(
            suggest_capability_name("internetServer"),
            Some("internetClientServer")
        );
    }

    #[test]
    fn prefers_highest_similarity_match() {
        let suggestion = suggest_capability_name("privateNetworkClientserve");
        assert_eq!(suggestion, Some("privateNetworkClientServer"));
    }
}

#[cfg(test)]
mod builder_tests {
    use super::{KnownCapability, SecurityCapabilitiesBuilder};
    use crate::sid::AppContainerSid;

    fn sample_sid() -> AppContainerSid {
        AppContainerSid::from_sddl("S-1-15-2-1")
    }

    #[test]
    fn lpac_defaults_enable_flag_and_append_registry_and_lpaccom() {
        let sid = sample_sid();
        let builder = SecurityCapabilitiesBuilder::new(&sid).with_lpac_defaults();
        assert!(builder.lpac_enabled_for_test());
        let names: Vec<&str> = builder
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(names, vec!["registryRead", "lpacCom"]);
    }

    #[test]
    fn known_capabilities_are_mapped_to_expected_names() {
        let sid = sample_sid();
        let builder = SecurityCapabilitiesBuilder::new(&sid)
            .with_known(&[
                KnownCapability::InternetClient,
                KnownCapability::InternetClientServer,
            ])
            .with_lpac_defaults();
        let names: Vec<&str> = builder
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "internetClient",
                "internetClientServer",
                "registryRead",
                "lpacCom"
            ]
        );
    }
}
