use super::{clear_env_override, repo_local_tempdir, set_env_override};
use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
    acl::{AccessMask, ResourcePath, grant_to_capability},
    supports_lpac,
};
use std::fs;

#[cfg(feature = "net")]
use rappct::net::list_appcontainers;

/// Demo 7: Network Container Enumeration
#[cfg(not(feature = "net"))]
pub(crate) fn demo_network_enumeration() -> rappct::Result<()> {
    println!("=== DEMO 7: Network Container Enumeration ===");
    println!("? Network enumeration requires the 'net' feature");
    println!("  Run with: --features net");
    println!();
    Ok(())
}

#[cfg(feature = "net")]
pub(crate) fn demo_network_enumeration() -> rappct::Result<()> {
    println!("=== DEMO 7: Network Container Enumeration ===");
    println!("Enumerating existing AppContainer profiles with network configuration");

    match list_appcontainers() {
        Ok(containers) => print_container_list(&containers),
        Err(e) => {
            println!("⚠ Enumeration failed: {e}");
            println!("  This may require Administrator privileges");
        }
    }
    println!();
    Ok(())
}

#[cfg(feature = "net")]
fn print_container_list(containers: &[(rappct::sid::AppContainerSid, String)]) {
    println!("✓ Found {} AppContainer profiles:", containers.len());
    if containers.is_empty() {
        println!("  (No containers found - this is normal on a fresh system)");
        return;
    }
    for (i, (sid, display_name)) in containers.iter().enumerate().take(10) {
        println!("  {}. {} - {}", i + 1, display_name, sid.as_string());
    }
    if containers.len() > 10 {
        println!("  ... and {} more", containers.len() - 10);
    }
}

/// Demo 8: Capability-based ACLs
pub(crate) fn demo_capability_acls() -> rappct::Result<()> {
    println!("=== DEMO 8: Capability-based ACLs ===");
    println!("Granting file access to specific capabilities rather than the container");

    let profile = AppContainerProfile::ensure("rappct.cap.acl", "Capability ACL", None)?;
    let scratch = repo_local_tempdir("capability-acls")?;
    let test_file = scratch.path().join("capability_test.txt");
    fs::write(&test_file, "This file requires specific capability access").map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to create test file {}: {}",
            test_file.display(),
            e
        ))
    })?;

    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;
    grant_demo_capability_access(&caps.caps, &test_file);

    fs::remove_file(&test_file).ok();
    profile.delete()?;
    scratch.close().map_err(|error| {
        rappct::AcError::Win32(format!(
            "Failed to clean repository-local capability ACL scratch: {error}"
        ))
    })?;
    println!("✓ Profile and test file cleaned up\n");
    Ok(())
}

fn grant_demo_capability_access(
    caps: &[rappct::sid::SidAndAttributes],
    test_file: &std::path::Path,
) {
    if caps.is_empty() {
        println!("⚠ No capabilities available for ACL demo");
        return;
    }

    let cap_sid = &caps[0].sid_sddl;
    println!("→ Granting access to capability: {cap_sid}");
    match grant_to_capability(
        ResourcePath::File(test_file.to_path_buf()),
        cap_sid,
        AccessMask::FILE_GENERIC_READ,
    ) {
        Ok(_) => println!("✓ Capability-based ACL applied successfully"),
        Err(e) => println!("⚠ Capability ACL failed: {e}"),
    }
}

/// Demo 9: LPAC Testing Environment
#[cfg(not(feature = "_test_helpers"))]
pub(crate) fn demo_lpac_testing() -> rappct::Result<()> {
    println!("=== DEMO 9: LPAC Testing Environment ===");
    println!("LPAC test overrides are available only with the private '_test_helpers' feature.");
    match supports_lpac() {
        Ok(_) => println!("✓ LPAC is natively supported on this system"),
        Err(_) => println!("✓ Native LPAC detection reports unsupported on this system"),
    }
    println!("✓ Native LPAC detection complete\n");
    Ok(())
}

/// Demo 9: LPAC Testing Environment
#[cfg(feature = "_test_helpers")]
pub(crate) fn demo_lpac_testing() -> rappct::Result<()> {
    println!("=== DEMO 9: LPAC Testing Environment ===");
    println!("Demonstrating test-only LPAC environment variable override");
    match supports_lpac() {
        Ok(_) => println!("✓ LPAC is natively supported on this system"),
        Err(_) => println!("✗ LPAC is not natively supported"),
    }

    println!("\n→ Testing environment variable override...");
    set_env_override("RAPPCT_TEST_LPAC_STATUS", "unsupported");
    match supports_lpac() {
        Ok(_) => println!("✗ Expected LPAC to be unsupported with env var"),
        Err(_) => println!("✓ LPAC correctly forced as unsupported"),
    }

    set_env_override("RAPPCT_TEST_LPAC_STATUS", "ok");
    match supports_lpac() {
        Ok(_) => println!("✓ LPAC correctly forced as supported"),
        Err(_) => println!("✗ Expected LPAC to be supported with env var"),
    }

    clear_env_override("RAPPCT_TEST_LPAC_STATUS");
    match supports_lpac() {
        Ok(_) => println!("✓ Back to native LPAC support detection"),
        Err(_) => println!("✓ Back to native LPAC support detection (unsupported)"),
    }

    println!("✓ Environment variable testing complete\n");
    Ok(())
}
