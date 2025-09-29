#![cfg(windows)]

use std::mem::size_of;

use rappct::sid::AppContainerSid;
use rappct::{
    launch_in_container, launch_in_container_with_io, AppContainerProfile, JobLimits,
    KnownCapability, LaunchOptions, Launched, LaunchedIo, Result, SecurityCapabilities,
    SecurityCapabilitiesBuilder, StdioConfig,
};

#[test]
fn api_reexports_are_accessible() {
    // Ensure core types are Sized and reachable from the crate root.
    let _ = size_of::<LaunchOptions>();
    let _ = size_of::<JobLimits>();
    let _ = size_of::<SecurityCapabilities>();
    let _ = StdioConfig::Inherit;
    let _ = KnownCapability::InternetClient;
    let _ = launch_in_container as fn(&SecurityCapabilities, &LaunchOptions) -> Result<Launched>;
    let _ = launch_in_container_with_io
        as fn(&SecurityCapabilities, &LaunchOptions) -> Result<LaunchedIo>;

    // Builders should be constructible without hitting Windows APIs (empty capability list).
    let sid = AppContainerSid::from_sddl("S-1-15-2-1");
    let builder = SecurityCapabilitiesBuilder::new(&sid);
    let caps = builder
        .with_named(&[])
        .build()
        .expect("build empty capability set");
    let opts = LaunchOptions::default();
    let _ = (&caps, &opts);

    // AppContainerProfile is reachable; avoid invoking OS APIs by only referencing associated items.
    let _ = AppContainerProfile::delete as fn(AppContainerProfile) -> Result<()>;

    #[cfg(feature = "introspection")]
    {
        use rappct::diag::{validate_configuration, ConfigWarning};
        let _ = validate_configuration
            as fn(&SecurityCapabilities, &LaunchOptions) -> Vec<ConfigWarning>;
    }

    #[cfg(feature = "net")]
    {
        use rappct::net::{add_loopback_exemption, list_appcontainers, LoopbackAdd};
        let sid = AppContainerSid::from_sddl("S-1-15-2-1");
        let _ = list_appcontainers as fn() -> Result<Vec<(AppContainerSid, String)>>;
        let _ = add_loopback_exemption as fn(LoopbackAdd) -> Result<()>;
        let _ = LoopbackAdd(sid);
    }
}
