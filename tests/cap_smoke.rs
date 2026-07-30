#![cfg(windows)]

use rappct::{
    AppContainerProfile, KnownCapability, LaunchOptions, Result, SecurityCapabilitiesBuilder,
    StdioConfig, launch_in_container_with_io, supports_lpac,
};
use std::time::Duration;

#[test]
fn lpac_launch_with_known_caps() -> Result<()> {
    if std::env::var("RAPPCT_ITESTS").ok().as_deref() != Some("1") || supports_lpac().is_err() {
        println!("skipped LPAC smoke test (requires RAPPCT_ITESTS=1 and native LPAC support)");
        return Ok(());
    }

    let profile =
        AppContainerProfile::ensure("rappct.cap_smoke", "rappct", Some("capability smoke test"))?;
    let result = run_lpac_smoke(&profile);
    let cleanup = profile.delete();
    result.and(cleanup)
}

fn run_lpac_smoke(profile: &AppContainerProfile) -> Result<()> {
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[
            KnownCapability::InternetClient,
            KnownCapability::PrivateNetworkClientServer,
        ])
        .with_lpac_defaults()
        .build()?;
    let options = LaunchOptions {
        exe: "C:\\Windows\\System32\\cmd.exe".into(),
        cmdline: Some(" /C ver".into()),
        stdio: StdioConfig::Null,
        ..Default::default()
    };
    let child = launch_in_container_with_io(&caps, &options)?;
    let exit_code = child.wait(Some(Duration::from_secs(30)))?;
    assert_eq!(exit_code, 0, "cmd /C ver should exit successfully");
    Ok(())
}
