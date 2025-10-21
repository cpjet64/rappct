#[cfg(all(windows, feature = "introspection"))]
use rappct::capability::{KnownCapability, SecurityCapabilitiesBuilder};
#[cfg(all(windows, feature = "introspection"))]
use rappct::diag::{validate_configuration, ConfigWarning};
#[cfg(all(windows, feature = "introspection"))]
use rappct::AppContainerProfile;

#[cfg(all(windows, feature = "introspection"))]
#[test]
fn diag_no_warnings_for_basic_appcontainer() {
    let profile =
        AppContainerProfile::ensure("rappct.test.diag.basic", "rappct diag", Some("diag test"))
            .expect("ensure profile");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps");
    let opts = rappct::launch::LaunchOptions::default();
    let warnings = validate_configuration(&caps, &opts);
    assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
    profile.delete().ok();
}

#[cfg(all(windows, feature = "introspection"))]
#[test]
fn diag_warns_when_network_caps_missing() {
    let profile = AppContainerProfile::ensure(
        "rappct.test.diag.nonetwork",
        "rappct diag",
        Some("diag test"),
    )
    .expect("ensure profile");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .build()
        .expect("build caps");
    let opts = rappct::launch::LaunchOptions::default();
    let warnings = validate_configuration(&caps, &opts);
    assert!(warnings.contains(&ConfigWarning::NoNetworkCaps));
    profile.delete().ok();
}

#[cfg(all(windows, feature = "introspection"))]
#[test]
fn diag_warns_when_lpac_missing_defaults() {
    let profile =
        AppContainerProfile::ensure("rappct.test.diag.lpac", "rappct diag", Some("diag test"))
            .expect("ensure profile");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .lpac(true)
        .build()
        .expect("build caps");
    let opts = rappct::launch::LaunchOptions::default();
    let warnings = validate_configuration(&caps, &opts);
    assert!(warnings.contains(&ConfigWarning::LpacWithoutCommonCaps));
    profile.delete().ok();
}
