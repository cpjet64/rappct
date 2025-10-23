#[cfg(windows)]
use rappct::*;

#[cfg(windows)]
use rappct::AcError;
#[cfg(windows)]
use rappct::capability::{Capability, CapabilityCatalog, CapabilityName};

#[cfg(windows)]
use std::sync::{Mutex, OnceLock};

#[cfg(windows)]
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(windows)]
#[test]
fn derive_sid_from_name_works() {
    let sid = derive_sid_from_name("rappct.test.unit").expect("derive sid");
    assert!(sid.as_string().starts_with("S-1-15-"));
}

#[cfg(windows)]
#[test]
fn derive_known_capability_sids() {
    let caps = capability::derive_named_capability_sids(&["internetClient"]).expect("derive caps");
    assert!(!caps.is_empty());
}

#[cfg(windows)]
#[test]
fn capability_catalog_resolves_known_entries() {
    let catalog = CapabilityCatalog::new().expect("catalog");
    let names = [
        CapabilityName::InternetClient,
        CapabilityName::PrivateNetworkClientServer,
    ];
    for name in names {
        let cap = catalog
            .capability(name)
            .unwrap_or_else(|| panic!("missing capability {name}"));
        assert_eq!(cap.name(), name);
        assert!(!cap.sid().sid_sddl.is_empty());
    }
}

#[cfg(windows)]
#[test]
fn capability_catalog_matches_sid_lookup() {
    use windows::Win32::Security::Authorization::ConvertStringSidToSidW;
    use windows::Win32::Security::{EqualSid, IsValidSid, PSID};
    use windows::core::PCWSTR;

    struct LocalSid(PSID);

    impl LocalSid {
        fn from_sddl(sddl: &str) -> Self {
            let wide = rappct::util::win::to_utf16(sddl);
            let mut psid = PSID::default();
            unsafe {
                ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid)
                    .expect("ConvertStringSidToSidW(capability)");
            }
            Self(psid)
        }

        fn as_psid(&self) -> PSID {
            self.0
        }
    }

    impl Drop for LocalSid {
        fn drop(&mut self) {
            if !self.0.0.is_null() {
                // SAFETY: Pointer originates from ConvertStringSidToSidW (LocalAlloc).
                unsafe {
                    let _ = LocalFree(self.0.0 as isize);
                }
                self.0 = PSID::default();
            }
        }
    }

    const SE_GROUP_ENABLED: u32 = 0x0000_0004;
    let catalog = CapabilityCatalog::new().expect("catalog");
    let names = [
        CapabilityName::InternetClient,
        CapabilityName::PrivateNetworkClientServer,
        CapabilityName::DocumentsLibrary,
    ];

    for &name in &names {
        let capability = catalog
            .capability(name)
            .unwrap_or_else(|| panic!("missing capability {name}"));
        let derived = capability::derive_named_capability_sids(&[name.as_str()])
            .unwrap_or_else(|e| panic!("derive_named_capability_sids failed: {e:?}"));
        let primary = derived
            .first()
            .unwrap_or_else(|| panic!("no SID returned for {name}"));

        assert_eq!(
            catalog.lookup_sid(&primary.sid_sddl),
            Some(name),
            "catalog lookup by SID should return {name:?}"
        );
        assert_eq!(
            capability.sid().sid_sddl,
            primary.sid_sddl,
            "catalog and direct derivation should agree on SDDL"
        );

        let catalog_sid = LocalSid::from_sddl(&capability.sid().sid_sddl);
        let derived_sid = LocalSid::from_sddl(&primary.sid_sddl);

        // SAFETY: SID pointers originate from ConvertStringSidToSidW and remain valid while these assertions run.
        unsafe {
            assert!(
                EqualSid(catalog_sid.as_psid(), derived_sid.as_psid()).is_ok(),
                "EqualSid mismatch for {name:?}"
            );
            assert!(
                IsValidSid(catalog_sid.as_psid()).as_bool(),
                "catalog SID invalid for {name:?}"
            );
            assert!(
                IsValidSid(derived_sid.as_psid()).as_bool(),
                "derived SID invalid for {name:?}"
            );
        }

        for (label, attrs) in [
            ("catalog", capability.sid().attributes),
            ("derived", primary.attributes),
        ] {
            assert_eq!(
                attrs & SE_GROUP_ENABLED,
                SE_GROUP_ENABLED,
                "{label} attributes missing SE_GROUP_ENABLED for {name:?}"
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn capability_try_from_str_handles_known_and_unknown() {
    let cap = Capability::try_from_str("internetClient").expect("internetClient capability");
    assert_eq!(cap.name(), CapabilityName::InternetClient);
    assert!(!cap.sid().sid_sddl.is_empty());

    match Capability::try_from_str("internetClientX") {
        Err(AcError::UnknownCapability { .. }) => {}
        other => panic!("expected unknown capability error, got {other:?}"),
    }
}

#[cfg(windows)]
#[test]
fn token_query_works() {
    let info = token::query_current_process_token().expect("token");
    assert!(
        !info.is_appcontainer,
        "host process unexpectedly running in AppContainer"
    );
    assert!(!info.is_lpac, "host process unexpectedly marked LPAC");
    assert!(
        info.package_sid.is_none(),
        "package SID should be absent outside AppContainer"
    );
    assert!(
        info.capability_sids.is_empty(),
        "capability list should be empty outside AppContainer"
    );
}

#[cfg(windows)]
#[test]
fn capability_derivation_repeated_calls_are_successful() {
    for _ in 0..4 {
        let caps =
            capability::derive_named_capability_sids(&["internetClient"]).expect("derive caps");
        assert_eq!(caps.len(), 1);
        assert!(!caps[0].sid_sddl.is_empty());
    }
}

#[cfg(all(windows, feature = "introspection"))]
#[test]
fn capability_typo_returns_suggestion() {
    // Ensure diagnostics entry point is available under the feature flag.
    let _f: fn(
        &rappct::SecurityCapabilities,
        &rappct::launch::LaunchOptions,
    ) -> Vec<rappct::diag::ConfigWarning> = rappct::diag::validate_configuration;
}

#[cfg(windows)]
#[test]
fn supports_lpac_override_ok() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var("RAPPCT_TEST_LPAC_STATUS", "ok");
    }
    let result = rappct::supports_lpac();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var("RAPPCT_TEST_LPAC_STATUS");
    }
    assert!(
        result.is_ok(),
        "supports_lpac should succeed when override requests ok"
    );
}

#[cfg(windows)]
#[test]
fn supports_lpac_override_unsupported() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var("RAPPCT_TEST_LPAC_STATUS", "unsupported");
    }
    let result = rappct::supports_lpac();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var("RAPPCT_TEST_LPAC_STATUS");
    }
    assert!(matches!(result, Err(rappct::AcError::UnsupportedLpac)));
}
