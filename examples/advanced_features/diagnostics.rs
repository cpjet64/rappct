#[cfg(feature = "introspection")]
use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder, launch::LaunchOptions,
    sid::AppContainerSid,
};
#[cfg(feature = "introspection")]
use std::path::PathBuf;

#[cfg(feature = "introspection")]
use rappct::diag::{ConfigWarning, validate_configuration};

#[cfg(feature = "introspection")]
struct ProfileCleanupGuard {
    profile: Option<AppContainerProfile>,
}

#[cfg(feature = "introspection")]
impl ProfileCleanupGuard {
    fn new(profile: AppContainerProfile) -> Self {
        Self {
            profile: Some(profile),
        }
    }

    fn profile(&self) -> rappct::Result<&AppContainerProfile> {
        self.profile.as_ref().ok_or_else(|| {
            rappct::AcError::Win32(
                "diagnostics profile guard was consumed before profile access".into(),
            )
        })
    }
}

#[cfg(feature = "introspection")]
impl Drop for ProfileCleanupGuard {
    fn drop(&mut self) {
        if let Some(profile) = self.profile.take() {
            let name = profile.name.clone();
            match profile.delete() {
                Ok(_) => {
                    println!("✓ Profile cleaned up");
                    println!();
                }
                Err(e) => println!("⚠ Failed to delete profile {name}: {e}"),
            }
        }
    }
}

/// Demo 4: Configuration Diagnostics
#[cfg(not(feature = "introspection"))]
pub(crate) fn demo_diagnostics() -> rappct::Result<()> {
    println!("=== DEMO 4: Configuration Diagnostics ===");
    println!("? Diagnostics require the 'introspection' feature");
    println!("  Run with: --features introspection");
    println!();
    Ok(())
}

#[cfg(feature = "introspection")]
pub(crate) fn demo_diagnostics() -> rappct::Result<()> {
    println!("=== DEMO 4: Configuration Diagnostics ===");
    println!("The introspection feature provides configuration validation");

    let profile_guard = ProfileCleanupGuard::new(AppContainerProfile::ensure(
        "rappct.diag.demo",
        "Diagnostics Demo",
        None,
    )?);
    let profile_sid = profile_guard.profile()?.sid.clone();
    let launch_opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        ..Default::default()
    };

    check_lpac_without_common_caps(&profile_sid, &launch_opts)?;
    check_no_network_caps(&profile_sid, &launch_opts)?;
    check_well_configured_lpac(&profile_sid, &launch_opts)?;
    drop(profile_guard);
    Ok(())
}

#[cfg(feature = "introspection")]
fn check_lpac_without_common_caps(
    profile_sid: &AppContainerSid,
    launch_opts: &LaunchOptions,
) -> rappct::Result<()> {
    println!("\n→ Test 1: LPAC without common capabilities");
    let lpac_caps = SecurityCapabilitiesBuilder::new(profile_sid)
        .lpac(true)
        .build()?;
    let warnings = validate_configuration(&lpac_caps, launch_opts);
    if warnings.contains(&ConfigWarning::LpacWithoutCommonCaps) {
        println!("✓ Detected: LPAC without common capabilities");
        println!("  Recommendation: Use .with_lpac_defaults()");
    }
    Ok(())
}

#[cfg(feature = "introspection")]
fn check_no_network_caps(
    profile_sid: &AppContainerSid,
    launch_opts: &LaunchOptions,
) -> rappct::Result<()> {
    println!("\n→ Test 2: Configuration without network capabilities");
    let no_net_caps = SecurityCapabilitiesBuilder::new(profile_sid).build()?;
    let warnings = validate_configuration(&no_net_caps, launch_opts);
    if warnings.contains(&ConfigWarning::NoNetworkCaps) {
        println!("✓ Detected: No network capabilities");
        println!("  Recommendation: Add network capabilities if needed");
    }
    Ok(())
}

#[cfg(feature = "introspection")]
fn check_well_configured_lpac(
    profile_sid: &AppContainerSid,
    launch_opts: &LaunchOptions,
) -> rappct::Result<()> {
    println!("\n→ Test 3: Well-configured LPAC");
    let good_caps = SecurityCapabilitiesBuilder::new(profile_sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults()
        .build()?;
    let warnings = validate_configuration(&good_caps, launch_opts);
    if warnings.is_empty() {
        println!("✓ No warnings - configuration looks good");
    } else {
        println!("⚠ Warnings found: {warnings:?}");
    }
    Ok(())
}
