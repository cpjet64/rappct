//! Capability catalog and builders for AppContainer and LPAC scenarios.
//!
//! AppContainer capabilities describe which system resources a sandboxed process may access,
//! whereas LPAC (Low Privilege AppContainer) builds on that surface with a more restrictive
//! default policy. The catalog provided here focuses on the common AppContainer capabilities
//! published by Microsoft and is used both for friendly name resolution and for constructing
//! `SECURITY_CAPABILITIES` structures at the FFI boundary.
//! See: <https://learn.microsoft.com/windows/win32/secauthz/appcontainer-capabilities>

use std::collections::BTreeMap;
#[cfg(windows)]
use std::collections::btree_map::Entry;

#[cfg(windows)]
use crate::ffi::sid::OwnedSid;
#[cfg(windows)]
use crate::ffi::wstr::WideString;
use crate::sid::SidAndAttributes;
use crate::{AcError, Result};

mod builder;
mod derive;

pub use builder::{
    SecurityCapabilities, SecurityCapabilitiesBuilder, UseCase, UseCaseCapabilities,
};
pub use derive::derive_named_capability_sids;

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

pub fn known_caps_to_named(caps: &[CapabilityName]) -> Vec<&'static str> {
    caps.iter().map(|c| c.as_str()).collect()
}

// Static list of known/supported capability names for suggestions.
#[cfg(feature = "introspection")]
#[allow(dead_code)]
static KNOWN_CAP_NAMES: &[&str] = WELL_KNOWN_CAPABILITY_NAMES;

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
            let derived = derive::derive_single_capability_sids(name.as_str())?;
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
        // SAFETY: The SDDL string provided originates from Windows via DeriveCapabilitySidsFromName.
        unsafe {
            ConvertStringSidToSidW(PCWSTR(wide.as_pcwstr().0), &mut psid)
                .map_err(|e| AcError::Win32(format!("ConvertStringSidToSidW failed: {e:?}")))?;
            OwnedSid::from_localfree_psid(psid.0)
        }
    }
}

impl CapabilityCatalog {
    pub fn new() -> Result<Self> {
        Self::from_names(CapabilityName::ALL)
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
