use super::cmd_exe;
use rappct::diag::{ConfigWarning, validate_configuration};
use rappct::*;

#[test]
fn diagnostics_reports_missing_caps() {
    let name = format!("rappct.test.launch.diag.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");

    let caps_no_network = SecurityCapabilitiesBuilder::new(&prof.sid)
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        ..Default::default()
    };
    let warnings_no_net = validate_configuration(&caps_no_network, &opts);
    assert!(warnings_no_net.contains(&ConfigWarning::NoNetworkCaps));

    let caps_lpac_missing = SecurityCapabilitiesBuilder::new(&prof.sid)
        .lpac(true)
        .build()
        .expect("build caps");
    let warnings_lpac = validate_configuration(&caps_lpac_missing, &opts);
    assert!(warnings_lpac.contains(&ConfigWarning::LpacWithoutCommonCaps));

    prof.delete().ok();
}
