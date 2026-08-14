use rappct::{
    AcError, AppContainerProfile, CapabilityName, JobLimits, LaunchOptions, SecurityCapabilities,
    SecurityCapabilitiesBuilder, StdioConfig, UseCase, WELL_KNOWN_CAPABILITY_NAMES,
    launch_in_container,
};

fn assert_send_sync<T: Send + Sync>() {}
fn assert_error<T: std::error::Error + Send + Sync>() {}

fn main() {
    assert_send_sync::<LaunchOptions>();
    assert_error::<AcError>();
    let _ = JobLimits::default();
    let _ = StdioConfig::Pipe;
    let _ = UseCase::MinimalLpac;
    assert!(!WELL_KNOWN_CAPABILITY_NAMES.is_empty());
    let sid = rappct::AppContainerSid::from_sddl("S-1-15-2-1");
    let _ = SecurityCapabilitiesBuilder::new(&sid)
        .with_known(&[CapabilityName::InternetClient])
        .with_lpac_defaults();
    let _ = SecurityCapabilitiesBuilder::from_use_case(UseCase::Custom).with_profile_sid(&sid);
    let _ = launch_in_container
        as fn(&SecurityCapabilities, &LaunchOptions) -> rappct::Result<rappct::Launched>;
    let _ = AppContainerProfile::ensure
        as fn(&str, &str, Option<&str>) -> rappct::Result<AppContainerProfile>;
    let _ = rappct::diag::validate_configuration
        as fn(&SecurityCapabilities, &LaunchOptions) -> Vec<rappct::diag::ConfigWarning>;
    let _ = rappct::net::list_appcontainers
        as fn() -> rappct::Result<Vec<(rappct::AppContainerSid, String)>>;
}
