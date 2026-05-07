//! Capability catalog and builders for AppContainer and LPAC scenarios.
//!
//! AppContainer capabilities describe which system resources a sandboxed process may access,
//! whereas LPAC (Low Privilege AppContainer) builds on that surface with a more restrictive
//! default policy. The catalog provided here focuses on the common AppContainer capabilities
//! published by Microsoft and is used both for friendly name resolution and for constructing
//! `SECURITY_CAPABILITIES` structures at the FFI boundary.
//! See: <https://learn.microsoft.com/windows/win32/secauthz/appcontainer-capabilities>

#[cfg(windows)]
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(windows)]
use crate::ffi::mem::LocalAllocGuard;
use crate::ffi::sid::OwnedSid;
#[cfg(windows)]
use crate::ffi::wstr::WideString;
use crate::sid::{AppContainerSid, SidAndAttributes};
use crate::{AcError, Result};

// windows::Win32::Security::SE_GROUP_ENABLED is not consistently available across crate versions.
// Use the documented value directly.
#[cfg(windows)]
const SE_GROUP_ENABLED_CONST: u32 = 0x0000_0004;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum CapabilityName {
    InternetClient,
    InternetClientServer,
    PrivateNetworkClientServer,
    EnterpriseAuthentication,
    SharedUserCertificates,
    UserAccountInformation,
    DocumentsLibrary,
    PicturesLibrary,
    VideosLibrary,
    MusicLibrary,
    Appointments,
    Contacts,
    PhoneCall,
    VoipCall,
    Location,
    Microphone,
    Webcam,
    LowLevelDevices,
    HumanInterfaceDevice,
    InputInjectionBrokered,
    RemovableStorage,
    RegistryRead,
    LpacCom,
}

/// Back-compat alias for the previous enum name.
pub type KnownCapability = CapabilityName;

impl CapabilityName {
    /// Back-compat list used by previous public API and tests.
    pub const ALL: &'static [Self] = &[
        Self::InternetClient,
        Self::InternetClientServer,
        Self::PrivateNetworkClientServer,
        Self::EnterpriseAuthentication,
        Self::SharedUserCertificates,
        Self::UserAccountInformation,
        Self::DocumentsLibrary,
        Self::PicturesLibrary,
        Self::VideosLibrary,
        Self::MusicLibrary,
        Self::Appointments,
        Self::Contacts,
        Self::PhoneCall,
        Self::VoipCall,
        Self::Location,
        Self::Microphone,
        Self::Webcam,
        Self::LowLevelDevices,
        Self::HumanInterfaceDevice,
        Self::InputInjectionBrokered,
        Self::RemovableStorage,
        Self::RegistryRead,
        Self::LpacCom,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            CapabilityName::InternetClient => "internetClient",
            CapabilityName::InternetClientServer => "internetClientServer",
            CapabilityName::PrivateNetworkClientServer => "privateNetworkClientServer",
            CapabilityName::EnterpriseAuthentication => "enterpriseAuthentication",
            CapabilityName::SharedUserCertificates => "sharedUserCertificates",
            CapabilityName::UserAccountInformation => "userAccountInformation",
            CapabilityName::DocumentsLibrary => "documentsLibrary",
            CapabilityName::PicturesLibrary => "picturesLibrary",
            CapabilityName::VideosLibrary => "videosLibrary",
            CapabilityName::MusicLibrary => "musicLibrary",
            CapabilityName::Appointments => "appointments",
            CapabilityName::Contacts => "contacts",
            CapabilityName::PhoneCall => "phoneCall",
            CapabilityName::VoipCall => "voipCall",
            CapabilityName::Location => "location",
            CapabilityName::Microphone => "microphone",
            CapabilityName::Webcam => "webcam",
            CapabilityName::LowLevelDevices => "lowLevelDevices",
            CapabilityName::HumanInterfaceDevice => "humanInterfaceDevice",
            CapabilityName::InputInjectionBrokered => "inputInjectionBrokered",
            CapabilityName::RemovableStorage => "removableStorage",
            CapabilityName::RegistryRead => "registryRead",
            CapabilityName::LpacCom => "lpacCom",
        }
    }

    /// Back-compat alias retained for existing callers.
    pub const fn as_name(self) -> &'static str {
        self.as_str()
    }

    /// Back-compat alias retained for existing callers.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|cap| cap.as_str() == name)
    }
}

/// Back-compat constant retained for existing callers and tests.
pub const WELL_KNOWN_CAPABILITY_NAMES: &[&str] = &[
    "internetClient",
    "internetClientServer",
    "privateNetworkClientServer",
    "enterpriseAuthentication",
    "sharedUserCertificates",
    "userAccountInformation",
    "documentsLibrary",
    "picturesLibrary",
    "videosLibrary",
    "musicLibrary",
    "appointments",
    "contacts",
    "phoneCall",
    "voipCall",
    "location",
    "microphone",
    "webcam",
    "lowLevelDevices",
    "humanInterfaceDevice",
    "inputInjectionBrokered",
    "removableStorage",
    "registryRead",
    "lpacCom",
];

impl core::fmt::Display for CapabilityName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

const ALL_CAPABILITY_NAMES: &[CapabilityName] = &[
    CapabilityName::InternetClient,
    CapabilityName::InternetClientServer,
    CapabilityName::PrivateNetworkClientServer,
    CapabilityName::EnterpriseAuthentication,
    CapabilityName::SharedUserCertificates,
    CapabilityName::UserAccountInformation,
    CapabilityName::DocumentsLibrary,
    CapabilityName::PicturesLibrary,
    CapabilityName::VideosLibrary,
    CapabilityName::MusicLibrary,
    CapabilityName::Appointments,
    CapabilityName::Contacts,
    CapabilityName::PhoneCall,
    CapabilityName::VoipCall,
    CapabilityName::Location,
    CapabilityName::Microphone,
    CapabilityName::Webcam,
    CapabilityName::LowLevelDevices,
    CapabilityName::HumanInterfaceDevice,
    CapabilityName::InputInjectionBrokered,
    CapabilityName::RemovableStorage,
    CapabilityName::RegistryRead,
    CapabilityName::LpacCom,
];

#[cfg_attr(not(feature = "introspection"), allow(dead_code))]
const CAPABILITY_NAME_STRINGS: &[&str] = &[
    "internetClient",
    "internetClientServer",
    "privateNetworkClientServer",
    "enterpriseAuthentication",
    "sharedUserCertificates",
    "userAccountInformation",
    "documentsLibrary",
    "picturesLibrary",
    "videosLibrary",
    "musicLibrary",
    "appointments",
    "contacts",
    "phoneCall",
    "voipCall",
    "location",
    "microphone",
    "webcam",
    "lowLevelDevices",
    "humanInterfaceDevice",
    "inputInjectionBrokered",
    "removableStorage",
    "registryRead",
    "lpacCom",
];

pub fn known_caps_to_named(caps: &[CapabilityName]) -> Vec<&'static str> {
    caps.iter().map(|c| c.as_str()).collect()
}

// Static list of known/supported capability names for suggestions.
#[cfg(feature = "introspection")]
#[allow(dead_code)]
static KNOWN_CAP_NAMES: &[&str] = CAPABILITY_NAME_STRINGS;

#[cfg(feature = "introspection")]
#[allow(dead_code)]
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
    if best < 0.80 { None } else { suggestion }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capability {
    name: CapabilityName,
    sid: SidAndAttributes,
}

pub struct CapabilityCatalog {
    by_name: BTreeMap<CapabilityName, Capability>,
    by_friendly: BTreeMap<&'static str, CapabilityName>,
    by_sid: BTreeMap<String, CapabilityName>,
}

impl Capability {
    #[cfg_attr(not(windows), allow(dead_code))]
    fn from_name(name: CapabilityName) -> Result<Self> {
        #[cfg(windows)]
        {
            let derived = derive_single_capability_sids(name.as_str())?;
            let sid = derived
                .into_iter()
                .next()
                .ok_or_else(|| AcError::UnknownCapability {
                    name: name.as_str().to_string(),
                    suggestion: None,
                })?;
            Ok(Self { name, sid })
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            Err(AcError::UnsupportedPlatform)
        }
    }

    pub fn try_from_str(friendly: &str) -> Result<Self> {
        CapabilityCatalog::new().and_then(|catalog| Self::try_from_catalog(&catalog, friendly))
    }

    pub fn try_from_catalog(catalog: &CapabilityCatalog, friendly: &str) -> Result<Self> {
        catalog.lookup(friendly).cloned()
    }

    pub fn name(&self) -> CapabilityName {
        self.name
    }

    pub fn sid(&self) -> &SidAndAttributes {
        &self.sid
    }

    #[cfg(windows)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn to_sid(&self) -> Result<OwnedSid> {
        use windows::Win32::Security::Authorization::ConvertStringSidToSidW;
        use windows::Win32::Security::PSID;
        use windows::core::PCWSTR;

        let wide = WideString::from_str(&self.sid.sid_sddl);
        let mut psid = PSID::default();
        // SAFETY: The SDDL string provided originates from Windows via DeriveCapabilitySidsFromName,
        // so `ConvertStringSidToSidW` receives a well-formed SID description and returns a pointer
        // allocated through LocalAlloc. We wrap the resulting PSID in `OwnedSid` to ensure it is
        // freed exactly once.
        unsafe {
            ConvertStringSidToSidW(PCWSTR(wide.as_pcwstr().0), &mut psid)
                .map_err(|e| AcError::Win32(format!("ConvertStringSidToSidW failed: {e:?}")))?;
            // SAFETY: ConvertStringSidToSidW allocates via LocalAlloc; wrap in OwnedSid to release.
            OwnedSid::from_localfree_psid(psid.0)
        }
    }

    #[cfg(not(windows))]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn to_sid(&self) -> Result<OwnedSid> {
        let _ = self;
        Err(AcError::UnsupportedPlatform)
    }
}

impl CapabilityCatalog {
    pub fn new() -> Result<Self> {
        Self::from_names(ALL_CAPABILITY_NAMES)
    }

    pub fn from_names(names: &[CapabilityName]) -> Result<Self> {
        #[cfg(windows)]
        {
            let mut by_name = BTreeMap::new();
            let mut by_sid = BTreeMap::new();
            for &name in names {
                if let Entry::Vacant(slot) = by_name.entry(name) {
                    let capability = Capability::from_name(name)?;
                    let sid_key = capability.sid().sid_sddl.clone();
                    slot.insert(capability);
                    by_sid.entry(sid_key).or_insert(name);
                }
            }
            let mut by_friendly = BTreeMap::new();
            for &name in names {
                by_friendly.entry(name.as_str()).or_insert(name);
            }
            Ok(Self {
                by_name,
                by_friendly,
                by_sid,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = names;
            Err(AcError::UnsupportedPlatform)
        }
    }

    pub fn capability(&self, name: CapabilityName) -> Option<&Capability> {
        self.by_name.get(&name)
    }

    pub fn lookup(&self, friendly: &str) -> Result<&Capability> {
        match self
            .by_friendly
            .get(friendly)
            .and_then(|name| self.by_name.get(name))
        {
            Some(cap) => Ok(cap),
            None => {
                #[cfg(feature = "introspection")]
                let suggestion = suggest_capability_name(friendly);
                #[cfg(not(feature = "introspection"))]
                let suggestion: Option<&'static str> = None;
                Err(AcError::UnknownCapability {
                    name: friendly.to_string(),
                    suggestion,
                })
            }
        }
    }

    pub fn lookup_sid(&self, sid_sddl: &str) -> Option<CapabilityName> {
        self.by_sid.get(sid_sddl).copied()
    }
}

/// Derive capability SIDs from names.
pub fn derive_named_capability_sids(names: &[&str]) -> Result<Vec<SidAndAttributes>> {
    if names.is_empty() {
        return Ok(vec![]);
    }
    #[cfg(windows)]
    {
        let mut out = Vec::new();
        for &name in names {
            let mut sids = derive_single_capability_sids(name)?;
            out.append(&mut sids);
        }
        Ok(out)
    }
    #[cfg(not(windows))]
    {
        let _ = names;
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
fn derive_single_capability_sids(name: &str) -> Result<Vec<SidAndAttributes>> {
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::core::{PCWSTR, PWSTR};
    // Some toolchains don't surface DeriveCapabilitySidsFromName via windows-rs; bind manually.
    #[link(name = "Userenv")]
    unsafe extern "system" {
        fn DeriveCapabilitySidsFromName(
            CapName: PCWSTR,
            CapGroupSids: *mut *mut *mut core::ffi::c_void,
            CapGroupSidCount: *mut u32,
            CapabilitySids: *mut *mut *mut core::ffi::c_void,
            CapabilitySidCount: *mut u32,
        ) -> i32;
    }
    let mut out: Vec<SidAndAttributes> = Vec::new();
    // SAFETY: For each capability name, we pass a valid PCWSTR and receive LocalAlloc-managed
    // SID arrays. We immediately wrap returned pointers in LocalAllocGuard to ensure single free
    // and only dereference within reported bounds. See Windows API docs for contracts.
    unsafe {
        #[cfg(feature = "tracing")]
        tracing::trace!("derive_single_capability_sids: name={}", name);
        let wide = WideString::from_str(name);
        let mut group_sids: *mut *mut std::ffi::c_void = std::ptr::null_mut();
        let mut group_count: u32 = 0;
        let mut cap_sids: *mut *mut std::ffi::c_void = std::ptr::null_mut();
        let mut cap_count: u32 = 0;
        let ok = DeriveCapabilitySidsFromName(
            wide.as_pcwstr(),
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
            if !group_sids.is_null() {
                let _ = LocalAllocGuard::<*mut std::ffi::c_void>::from_raw(group_sids);
            }
            if !cap_sids.is_null() {
                let _ = LocalAllocGuard::<*mut std::ffi::c_void>::from_raw(cap_sids);
            }
            return Err(AcError::UnknownCapability {
                name: name.to_string(),
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
        if group_count > 0 && group_sids.is_null() {
            if !cap_sids.is_null() {
                let _ = LocalAllocGuard::<*mut std::ffi::c_void>::from_raw(cap_sids);
            }
            return Err(AcError::Win32(format!(
                "DeriveCapabilitySidsFromName returned null group SID array for '{name}' (count={group_count})"
            )));
        }
        if cap_count > 0 && cap_sids.is_null() {
            if !group_sids.is_null() {
                let _ = LocalAllocGuard::<*mut std::ffi::c_void>::from_raw(group_sids);
            }
            return Err(AcError::Win32(format!(
                "DeriveCapabilitySidsFromName returned null capability SID array for '{name}' (count={cap_count})"
            )));
        }
        // Convert capability group SIDs (ignored for SECURITY_CAPABILITIES; free them)
        for i in 0..group_count as isize {
            let sid_ptr: *mut std::ffi::c_void = *group_sids.offset(i);
            if !sid_ptr.is_null() {
                let _guard = LocalAllocGuard::<std::ffi::c_void>::from_raw(sid_ptr);
            }
        }
        // Convert capability SIDs
        let mut conversion_error: Option<AcError> = None;
        for i in 0..cap_count as isize {
            let sid_ptr: *mut std::ffi::c_void = *cap_sids.offset(i);
            if !sid_ptr.is_null() {
                let sid_guard = LocalAllocGuard::<std::ffi::c_void>::from_raw(sid_ptr);
                let mut sddl = PWSTR::null();
                match ConvertSidToStringSidW(
                    windows::Win32::Security::PSID(sid_guard.as_ptr()),
                    &mut sddl,
                ) {
                    Ok(()) => {
                        let sddl_guard = LocalAllocGuard::<u16>::from_raw(sddl.0);
                        let s = sddl_guard.to_string_lossy();
                        out.push(SidAndAttributes {
                            sid_sddl: s,
                            attributes: SE_GROUP_ENABLED_CONST,
                        });
                    }
                    Err(e) => {
                        if conversion_error.is_none() {
                            conversion_error = Some(AcError::Win32(format!(
                                "ConvertSidToStringSidW failed for capability '{name}': {e:?}"
                            )));
                        }
                    }
                }
            }
        }
        // Free arrays
        if !group_sids.is_null() {
            let _ = LocalAllocGuard::<*mut std::ffi::c_void>::from_raw(group_sids);
        }
        if !cap_sids.is_null() {
            let _ = LocalAllocGuard::<*mut std::ffi::c_void>::from_raw(cap_sids);
        }

        if let Some(err) = conversion_error {
            return Err(err);
        }
        if out.len() != cap_count as usize {
            return Err(AcError::Win32(format!(
                "Derived {} capability SID(s) for '{name}' but converted {}",
                cap_count,
                out.len()
            )));
        }
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SecurityCapabilities {
    pub package: AppContainerSid,
    pub caps: Vec<SidAndAttributes>,
    pub lpac: bool,
}

#[derive(Clone, Debug)]
pub struct SecurityCapabilitiesBuilder {
    package: AppContainerSid,
    caps_named: Vec<String>,
    lpac: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum UseCase {
    /// Internet-enabled scrape-like workloads with minimal extra rights.
    SecureWebScraper,
    /// LPAC default baseline with registry-focused access pattern.
    IsolatedBuildEnvironment,
    /// Networked tool with private network capability, typically paired with loopback checks.
    NetworkConstrainedTool,
    /// Minimal LPAC-only capability set.
    MinimalLpac,
    /// Safer desktop-style baseline for interactive workloads.
    FullDesktopApp,
    /// No preset; callers should add capabilities explicitly.
    Custom,
}

pub struct UseCaseCapabilities {
    caps_named: Vec<String>,
    lpac: bool,
}

impl UseCaseCapabilities {
    pub fn with_profile_sid(self, sid: &AppContainerSid) -> SecurityCapabilitiesBuilder {
        SecurityCapabilitiesBuilder {
            package: sid.clone(),
            caps_named: self.caps_named,
            lpac: self.lpac,
        }
    }

    #[cfg(test)]
    fn named_caps_for_test(&self) -> &[String] {
        &self.caps_named
    }
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
    /// Compatibility no-op to support older builder chains that called `.unwrap()`.
    /// Returns `self` unchanged.
    pub fn unwrap(self) -> Self {
        self
    }
    pub fn lpac(mut self, enabled: bool) -> Self {
        self.lpac = enabled;
        self
    }
    pub fn from_use_case(use_case: UseCase) -> UseCaseCapabilities {
        let mut caps_named = Vec::new();
        let mut lpac = false;
        match use_case {
            UseCase::SecureWebScraper => {
                caps_named.push(KnownCapability::InternetClient.as_str().to_string());
            }
            UseCase::IsolatedBuildEnvironment => {
                caps_named.extend([
                    KnownCapability::RegistryRead.as_str().to_string(),
                    KnownCapability::LpacCom.as_str().to_string(),
                ]);
                lpac = true;
            }
            UseCase::NetworkConstrainedTool => {
                caps_named.push(
                    KnownCapability::PrivateNetworkClientServer
                        .as_str()
                        .to_string(),
                );
            }
            UseCase::MinimalLpac => {
                caps_named.extend([
                    KnownCapability::RegistryRead.as_str().to_string(),
                    KnownCapability::LpacCom.as_str().to_string(),
                ]);
                lpac = true;
            }
            UseCase::FullDesktopApp => {
                caps_named.extend([
                    KnownCapability::InternetClient.as_str().to_string(),
                    KnownCapability::PrivateNetworkClientServer
                        .as_str()
                        .to_string(),
                    KnownCapability::InternetClientServer.as_str().to_string(),
                    KnownCapability::UserAccountInformation.as_str().to_string(),
                ]);
            }
            UseCase::Custom => {}
        };
        UseCaseCapabilities { caps_named, lpac }
    }
    pub fn build(self) -> Result<SecurityCapabilities> {
        let mut seen = BTreeSet::new();
        let deduped_caps: Vec<&str> = self
            .caps_named
            .iter()
            .filter_map(|s| seen.insert(s.as_str()).then_some(s.as_str()))
            .collect();
        let caps = derive_named_capability_sids(&deduped_caps)?;
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
    use super::{
        ALL_CAPABILITY_NAMES, KnownCapability, SecurityCapabilitiesBuilder, UseCase,
        WELL_KNOWN_CAPABILITY_NAMES,
    };
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

    #[test]
    fn capability_variants_report_expected_names_and_display() {
        let cases = [
            (KnownCapability::InternetClient, "internetClient"),
            (
                KnownCapability::InternetClientServer,
                "internetClientServer",
            ),
            (
                KnownCapability::PrivateNetworkClientServer,
                "privateNetworkClientServer",
            ),
            (
                KnownCapability::EnterpriseAuthentication,
                "enterpriseAuthentication",
            ),
            (
                KnownCapability::SharedUserCertificates,
                "sharedUserCertificates",
            ),
            (
                KnownCapability::UserAccountInformation,
                "userAccountInformation",
            ),
            (KnownCapability::DocumentsLibrary, "documentsLibrary"),
            (KnownCapability::PicturesLibrary, "picturesLibrary"),
            (KnownCapability::VideosLibrary, "videosLibrary"),
            (KnownCapability::MusicLibrary, "musicLibrary"),
            (KnownCapability::Appointments, "appointments"),
            (KnownCapability::Contacts, "contacts"),
            (KnownCapability::PhoneCall, "phoneCall"),
            (KnownCapability::VoipCall, "voipCall"),
            (KnownCapability::Location, "location"),
            (KnownCapability::Microphone, "microphone"),
            (KnownCapability::Webcam, "webcam"),
            (KnownCapability::LowLevelDevices, "lowLevelDevices"),
            (
                KnownCapability::HumanInterfaceDevice,
                "humanInterfaceDevice",
            ),
            (
                KnownCapability::InputInjectionBrokered,
                "inputInjectionBrokered",
            ),
            (KnownCapability::RemovableStorage, "removableStorage"),
            (KnownCapability::RegistryRead, "registryRead"),
            (KnownCapability::LpacCom, "lpacCom"),
        ];

        for (cap, expected) in cases {
            assert_eq!(cap.as_str(), expected);
            assert_eq!(cap.as_name(), expected);
            assert_eq!(cap.to_string(), expected);
        }

        assert_eq!(
            KnownCapability::from_name("internetClient"),
            Some(KnownCapability::InternetClient)
        );
        assert_eq!(
            KnownCapability::from_name("internetClientServer"),
            Some(KnownCapability::InternetClientServer)
        );
        assert_eq!(
            KnownCapability::from_name("privateNetworkClientServer"),
            Some(KnownCapability::PrivateNetworkClientServer)
        );
    }

    #[test]
    fn with_named_empty_is_noop() {
        let sid = sample_sid();
        let builder = SecurityCapabilitiesBuilder::new(&sid)
            .with_known(&[KnownCapability::InternetClient])
            .with_named(&[]);

        assert!(!builder.lpac_enabled_for_test());
        let names: Vec<&str> = builder
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(names, vec!["internetClient"]);
    }

    #[test]
    fn with_named_appends_verbatim_and_preserves_lpac_flag() {
        let sid = sample_sid();
        let builder = SecurityCapabilitiesBuilder::new(&sid)
            .lpac(true)
            .with_named(&["alpha", "beta", "alpha"]);

        assert!(builder.lpac_enabled_for_test());
        let names: Vec<&str> = builder
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "alpha"]);
    }

    #[test]
    fn from_use_case_creates_expected_caps() {
        let secure = SecurityCapabilitiesBuilder::from_use_case(UseCase::SecureWebScraper);
        let names: Vec<&str> = secure
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(names, vec!["internetClient"]);

        let minimal = SecurityCapabilitiesBuilder::from_use_case(UseCase::MinimalLpac);
        let names: Vec<&str> = minimal
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(names, vec!["registryRead", "lpacCom"]);

        let desktop = SecurityCapabilitiesBuilder::from_use_case(UseCase::FullDesktopApp);
        let names: Vec<&str> = desktop
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "internetClient",
                "privateNetworkClientServer",
                "internetClientServer",
                "userAccountInformation"
            ]
        );
    }

    #[test]
    fn additional_use_cases_set_expected_caps_and_flags() {
        let isolated =
            SecurityCapabilitiesBuilder::from_use_case(UseCase::IsolatedBuildEnvironment);
        let isolated_names: Vec<&str> = isolated
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(isolated_names, vec!["registryRead", "lpacCom"]);
        assert!(
            isolated
                .with_profile_sid(&sample_sid())
                .lpac_enabled_for_test(),
            "isolated build environment should enable LPAC"
        );

        let network = SecurityCapabilitiesBuilder::from_use_case(UseCase::NetworkConstrainedTool);
        let network_names: Vec<&str> = network
            .named_caps_for_test()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(network_names, vec!["privateNetworkClientServer"]);
        assert!(
            !network
                .with_profile_sid(&sample_sid())
                .lpac_enabled_for_test(),
            "network-constrained tool should not force LPAC"
        );

        let custom = SecurityCapabilitiesBuilder::from_use_case(UseCase::Custom);
        assert!(custom.named_caps_for_test().is_empty());
        assert!(
            !custom
                .with_profile_sid(&sample_sid())
                .lpac_enabled_for_test(),
            "custom use case should remain non-LPAC by default"
        );
    }

    #[test]
    fn from_use_case_allows_profile_sid_to_finalize() {
        let sid = sample_sid();
        let builder =
            SecurityCapabilitiesBuilder::from_use_case(UseCase::MinimalLpac).with_profile_sid(&sid);
        let built = builder.build().expect("build from preset");
        assert!(built.lpac);
        assert_eq!(built.caps.len(), 2);
    }

    #[test]
    fn known_capabilities_all_and_well_known_names_stay_in_sync() {
        assert_eq!(
            ALL_CAPABILITY_NAMES.len(),
            WELL_KNOWN_CAPABILITY_NAMES.len(),
            "known capability names should be defined for every enum variant"
        );

        for name in WELL_KNOWN_CAPABILITY_NAMES {
            assert!(
                KnownCapability::from_name(name).is_some(),
                "known capability constant missing lookup implementation: {name}"
            );
        }
    }
}
