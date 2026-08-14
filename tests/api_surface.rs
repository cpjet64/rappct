use std::mem::size_of;

use rappct::sid::AppContainerSid;
use rappct::{
    AcError, AppContainerProfile, JobLimits, KnownCapability, LaunchOptions, Launched, Result,
    SecurityCapabilities, SecurityCapabilitiesBuilder, StdioConfig, UseCase,
    WELL_KNOWN_CAPABILITY_NAMES, launch_in_container,
};

#[test]
fn core_public_api_is_accessible_on_every_platform() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_error<T: std::error::Error + Send + Sync>() {}

    let _ = size_of::<LaunchOptions>();
    assert_send_sync::<LaunchOptions>();
    assert_error::<AcError>();
    let _ = size_of::<JobLimits>();
    let _ = size_of::<SecurityCapabilities>();
    let _ = StdioConfig::Inherit;
    let _ = KnownCapability::InternetClient;
    let _ = KnownCapability::ALL;
    let _ = KnownCapability::InternetClient.as_name();
    let _ = KnownCapability::from_name("internetClient");
    assert!(!WELL_KNOWN_CAPABILITY_NAMES.is_empty());
    let _ = launch_in_container as fn(&SecurityCapabilities, &LaunchOptions) -> Result<Launched>;
    let _ = rappct::launch::launch_in_container_with_io
        as fn(&SecurityCapabilities, &LaunchOptions) -> Result<rappct::launch::LaunchedIo>;

    let sid = AppContainerSid::from_sddl("S-1-15-2-1");
    let builder = SecurityCapabilitiesBuilder::new(&sid)
        .with_known(&[KnownCapability::InternetClient])
        .lpac(false);
    let _ = SecurityCapabilitiesBuilder::from_use_case(UseCase::Custom).with_profile_sid(&sid);
    let _ = builder;
    let opts = LaunchOptions::default();
    let _ = (&sid, &opts);

    let _ = AppContainerProfile::delete as fn(AppContainerProfile) -> Result<()>;
    let _ = AppContainerProfile::open as fn(&str) -> Result<AppContainerProfile>;

    #[cfg(feature = "introspection")]
    {
        use rappct::diag::{ConfigWarning, validate_configuration};
        let _ = validate_configuration
            as fn(&SecurityCapabilities, &LaunchOptions) -> Vec<ConfigWarning>;
    }

    #[cfg(feature = "net")]
    {
        use rappct::net::{LoopbackAdd, add_loopback_exemption, list_appcontainers};
        let sid = AppContainerSid::from_sddl("S-1-15-2-1");
        let _ = list_appcontainers as fn() -> Result<Vec<(AppContainerSid, String)>>;
        let _ = add_loopback_exemption as fn(LoopbackAdd) -> Result<()>;
        let _ = LoopbackAdd::new(sid);
    }
}

#[cfg(not(windows))]
#[test]
fn platform_specific_launch_apis_fail_closed() {
    let caps = SecurityCapabilities {
        package: AppContainerSid::from_sddl("S-1-15-2-1"),
        caps: Vec::new(),
        lpac: false,
    };
    let options = LaunchOptions::default();

    assert!(matches!(
        launch_in_container(&caps, &options),
        Err(AcError::UnsupportedPlatform)
    ));
    assert!(matches!(
        rappct::launch::launch_in_container_with_io(&caps, &options),
        Err(AcError::UnsupportedPlatform)
    ));
}

#[cfg(windows)]
#[test]
fn windows_root_io_reexports_are_accessible() {
    let _ = rappct::launch_in_container_with_io
        as fn(&SecurityCapabilities, &LaunchOptions) -> Result<rappct::LaunchedIo>;
}
