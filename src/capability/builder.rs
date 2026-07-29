use std::collections::BTreeSet;

use crate::Result;
use crate::sid::{AppContainerSid, SidAndAttributes};

use super::{KnownCapability, derive_named_capability_sids, known_caps_to_named};

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
        self.caps_named.extend(
            known_caps_to_named(caps)
                .into_iter()
                .map(std::string::ToString::to_string),
        );
        self
    }

    pub fn with_named(mut self, names: &[&str]) -> Self {
        self.caps_named
            .extend(names.iter().map(std::string::ToString::to_string));
        self
    }

    /// Opinionated minimal LPAC defaults: `registryRead` and `lpacCom`.
    pub fn with_lpac_defaults(mut self) -> Self {
        self.lpac = true;
        self.caps_named.push("registryRead".to_string());
        self.caps_named.push("lpacCom".to_string());
        self
    }

    /// Compatibility no-op to support older builder chains that called `.unwrap()`.
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
                push_known(&mut caps_named, KnownCapability::InternetClient)
            }
            UseCase::IsolatedBuildEnvironment | UseCase::MinimalLpac => {
                push_lpac_defaults(&mut caps_named);
                lpac = true;
            }
            UseCase::NetworkConstrainedTool => {
                push_known(&mut caps_named, KnownCapability::PrivateNetworkClientServer);
            }
            UseCase::FullDesktopApp => push_full_desktop_caps(&mut caps_named),
            UseCase::Custom => {}
        };
        UseCaseCapabilities { caps_named, lpac }
    }

    pub fn build(self) -> Result<SecurityCapabilities> {
        let deduped_caps = dedupe_named_caps(&self.caps_named);
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

fn push_known(caps_named: &mut Vec<String>, cap: KnownCapability) {
    caps_named.push(cap.as_str().to_string());
}

fn push_lpac_defaults(caps_named: &mut Vec<String>) {
    push_known(caps_named, KnownCapability::RegistryRead);
    push_known(caps_named, KnownCapability::LpacCom);
}

fn push_full_desktop_caps(caps_named: &mut Vec<String>) {
    for cap in [
        KnownCapability::InternetClient,
        KnownCapability::PrivateNetworkClientServer,
        KnownCapability::InternetClientServer,
        KnownCapability::UserAccountInformation,
    ] {
        push_known(caps_named, cap);
    }
}

fn dedupe_named_caps(caps: &[String]) -> Vec<&str> {
    let mut seen = BTreeSet::new();
    caps.iter()
        .filter_map(|cap| seen.insert(cap.as_str()).then_some(cap.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{SecurityCapabilitiesBuilder, UseCase};
    use crate::capability::{CapabilityName, KnownCapability, WELL_KNOWN_CAPABILITY_NAMES};
    use crate::sid::AppContainerSid;

    fn sample_sid() -> AppContainerSid {
        AppContainerSid::from_sddl("S-1-15-2-1")
    }

    fn builder_names(builder: &SecurityCapabilitiesBuilder) -> Vec<&str> {
        builder
            .named_caps_for_test()
            .iter()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn lpac_defaults_enable_flag_and_append_registry_and_lpaccom() {
        let builder = SecurityCapabilitiesBuilder::new(&sample_sid()).with_lpac_defaults();
        assert!(builder.lpac_enabled_for_test());
        assert_eq!(builder_names(&builder), vec!["registryRead", "lpacCom"]);
    }

    #[test]
    fn known_capabilities_are_mapped_to_expected_names() {
        let builder = SecurityCapabilitiesBuilder::new(&sample_sid())
            .with_known(&[
                KnownCapability::InternetClient,
                KnownCapability::InternetClientServer,
            ])
            .with_lpac_defaults();

        assert_eq!(
            builder_names(&builder),
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
        for (cap, expected) in capability_name_cases() {
            assert_eq!(cap.as_str(), expected);
            assert_eq!(cap.as_name(), expected);
            assert_eq!(cap.to_string(), expected);
        }
    }

    #[test]
    fn capability_name_lookup_matches_common_cases() {
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
        let builder = SecurityCapabilitiesBuilder::new(&sample_sid())
            .with_known(&[KnownCapability::InternetClient])
            .with_named(&[]);

        assert!(!builder.lpac_enabled_for_test());
        assert_eq!(builder_names(&builder), vec!["internetClient"]);
    }

    #[test]
    fn with_named_appends_verbatim_and_preserves_lpac_flag() {
        let builder = SecurityCapabilitiesBuilder::new(&sample_sid())
            .lpac(true)
            .with_named(&["alpha", "beta", "alpha"]);

        assert!(builder.lpac_enabled_for_test());
        assert_eq!(builder_names(&builder), vec!["alpha", "beta", "alpha"]);
    }

    #[test]
    fn from_use_case_creates_expected_caps() {
        assert_use_case_names(UseCase::SecureWebScraper, &["internetClient"]);
        assert_use_case_names(UseCase::MinimalLpac, &["registryRead", "lpacCom"]);
        assert_use_case_names(
            UseCase::FullDesktopApp,
            &[
                "internetClient",
                "privateNetworkClientServer",
                "internetClientServer",
                "userAccountInformation",
            ],
        );
    }

    #[test]
    fn additional_use_cases_set_expected_caps_and_flags() {
        assert_use_case_builder(
            UseCase::IsolatedBuildEnvironment,
            true,
            &["registryRead", "lpacCom"],
        );
        assert_use_case_builder(
            UseCase::NetworkConstrainedTool,
            false,
            &["privateNetworkClientServer"],
        );
        assert_use_case_builder(UseCase::Custom, false, &[]);
    }

    #[test]
    fn from_use_case_allows_profile_sid_to_finalize() {
        let builder = SecurityCapabilitiesBuilder::from_use_case(UseCase::MinimalLpac)
            .with_profile_sid(&sample_sid());
        #[cfg(not(windows))]
        {
            assert!(matches!(
                builder.build(),
                Err(crate::AcError::UnsupportedPlatform)
            ));
        }
        #[cfg(windows)]
        {
            let built = builder.build().expect("build from preset");
            assert!(built.lpac);
            assert_eq!(built.caps.len(), 2);
        }
    }

    #[test]
    fn known_capabilities_all_and_well_known_names_stay_in_sync() {
        assert_eq!(
            CapabilityName::ALL.len(),
            WELL_KNOWN_CAPABILITY_NAMES.len(),
            "known capability names should be defined for every enum variant"
        );
        for name in WELL_KNOWN_CAPABILITY_NAMES {
            assert!(KnownCapability::from_name(name).is_some());
        }
    }

    fn assert_use_case_names(use_case: UseCase, expected: &[&str]) {
        let preset = SecurityCapabilitiesBuilder::from_use_case(use_case);
        let names: Vec<&str> = preset
            .named_caps_for_test()
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(names, expected);
    }

    fn assert_use_case_builder(use_case: UseCase, lpac: bool, expected: &[&str]) {
        let preset = SecurityCapabilitiesBuilder::from_use_case(use_case);
        let names: Vec<&str> = preset
            .named_caps_for_test()
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(names, expected);
        assert_eq!(
            preset
                .with_profile_sid(&sample_sid())
                .lpac_enabled_for_test(),
            lpac
        );
    }

    fn capability_name_cases() -> [(KnownCapability, &'static str); 23] {
        [
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
        ]
    }
}
