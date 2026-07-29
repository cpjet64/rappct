use super::cmd_exe;
use rappct::*;

fn launch_profile(scope: &str) -> AppContainerProfile {
    let name = format!("rappct.test.launch.{scope}.{}", std::process::id());
    AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure")
}

fn internet_caps(profile: &AppContainerProfile) -> SecurityCapabilities {
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps")
}

#[test]
fn launch_nonexistent_exe_fails() {
    let prof = launch_profile("noexe");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: std::path::PathBuf::from("C:\\__rappct_nonexistent_exe__.exe"),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    };
    let err = launch_in_container(&caps, &opts).unwrap_err();
    assert!(
        matches!(err, AcError::LaunchFailed { .. }),
        "expected LaunchFailed, got: {err:?}"
    );
    prof.delete().ok();
}

#[test]
fn launch_invalid_cwd_fails() {
    let prof = launch_profile("badcwd");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        cwd: Some(std::path::PathBuf::from("C:\\__rappct_nonexistent_cwd__")),
        ..Default::default()
    };
    let err = launch_in_container(&caps, &opts).unwrap_err();
    assert!(
        matches!(err, AcError::LaunchFailed { .. }),
        "expected LaunchFailed, got: {err:?}"
    );
    prof.delete().ok();
}

#[test]
fn launch_ac_cmd_exits() {
    let prof = launch_profile("basic");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts).expect("launch ac");
    assert!(child.pid > 0);
    prof.delete().ok();
}

#[test]
fn launch_lpac_cmd_exits_if_supported() {
    if supports_lpac().is_err() {
        return;
    }
    let prof = launch_profile("lpac");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults()
        .lpac(true)
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts).expect("launch lpac");
    assert!(child.pid > 0);
    prof.delete().ok();
}
