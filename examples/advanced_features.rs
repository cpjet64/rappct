//! Advanced rappct features demonstration
//!
//! This example covers the less commonly used but powerful features of rappct:
//! - Profile path resolution (folder_path, named_object_path)
//! - Custom named capabilities
//! - Configuration diagnostics
//! - Advanced launch options
//! - Network enumeration
//! - Direct SID derivation

use rappct::{
    acl::{grant_to_capability, AccessMask, ResourcePath},
    launch::{JobLimits, LaunchOptions},
    launch_in_container,
    profile::derive_sid_from_name,
    supports_lpac, AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
};

#[cfg(windows)]
use rappct::launch::{launch_in_container_with_io, StdioConfig};

#[cfg(feature = "introspection")]
use rappct::diag::{validate_configuration, ConfigWarning};

#[cfg(feature = "net")]
use rappct::net::list_appcontainers;

use std::{env, ffi::OsString, fs, path::PathBuf, thread, time::Duration};

#[cfg(windows)]
use std::io::{BufRead, BufReader};

type DemoEntry = (&'static str, fn() -> rappct::Result<()>);

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

    fn profile(&self) -> &AppContainerProfile {
        self.profile
            .as_ref()
            .expect("profile available during diagnostics demo")
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
                Err(e) => println!("⚠ Failed to delete profile {}: {}", name, e),
            }
        }
    }
}

#[cfg(windows)]
fn set_env_override(key: &str, value: &str) {
    // Mutates process environment; run single-threaded or before starting worker threads.
    // Note: Environment mutation is unsafe on recent Rust; keep calls scoped.
    unsafe {
        std::env::set_var(key, value);
    }
}

#[cfg(not(windows))]
fn set_env_override(key: &str, value: &str) {
    // Mutates process environment; run single-threaded or before starting worker threads.
    let _ = (key, value);
}

#[cfg(windows)]
fn clear_env_override(key: &str) {
    // See note in set_env_override.
    unsafe {
        std::env::remove_var(key);
    }
}

#[cfg(not(windows))]
fn clear_env_override(key: &str) {
    // See note in set_env_override.
    let _ = key;
}

fn resolve_cmd_exe() -> PathBuf {
    if let Ok(comspec) = env::var("ComSpec") {
        let p = PathBuf::from(comspec);
        if p.exists() {
            return p;
        }
    }
    if let Ok(root) = env::var("SystemRoot") {
        let p = PathBuf::from(format!(r"{}\System32\cmd.exe", root));
        if p.exists() {
            return p;
        }
    }
    let candidates = [
        PathBuf::from(r"C:\\Windows\\Sysnative\\cmd.exe"),
        PathBuf::from(r"C:\\Windows\\System32\\cmd.exe"),
    ];
    for p in candidates {
        if p.exists() {
            return p;
        }
    }
    PathBuf::from(r"C:\\Windows\\System32\\cmd.exe")
}

fn main() -> rappct::Result<()> {
    println!("rappct Advanced Features Demo");
    println!("=============================\n");

    println!("This demo showcases advanced and lesser-known rappct features.");
    println!("Some features require specific feature flags to be enabled.\n");

    // Run each demo and continue on failure to provide full coverage
    let demos: Vec<DemoEntry> = vec![
        ("Profile Path Resolution", demo_profile_paths),
        ("Direct SID Derivation", demo_sid_derivation),
        ("Custom Named Capabilities", demo_custom_capabilities),
        ("Configuration Diagnostics", demo_diagnostics),
        ("Advanced Launch Options", demo_advanced_launch),
        ("Enhanced I/O with Error Handling", demo_enhanced_io),
        ("Network Container Enumeration", demo_network_enumeration),
        ("Capability-based ACLs", demo_capability_acls),
        ("LPAC Testing Environment", demo_lpac_testing),
    ];
    for (name, f) in demos {
        match f() {
            Ok(_) => println!("\n✓ {} completed", name),
            Err(e) => {
                println!("\n⚠ {} failed: {}", name, e);
                if let Some(src) = std::error::Error::source(&e) {
                    println!("   OS error: {}", src);
                }
                println!("   Continuing with next demo...\n");
            }
        }
    }

    println!("\n🎉 Advanced Features Demo Complete!");
    println!("====================================");
    println!("You've seen rappct's advanced capabilities for:");
    println!("• Profile path resolution and named objects");
    println!("• Custom capability configuration");
    println!("• Configuration validation and diagnostics");
    println!("• Advanced process launching with custom environments");
    println!("• Network container enumeration and management");
    println!("• Capability-based access control");

    Ok(())
}

/// Demo 1: Profile Path Resolution
fn demo_profile_paths() -> rappct::Result<()> {
    println!("=== DEMO 1: Profile Path Resolution ===");
    println!("AppContainer profiles have associated file system and named object paths");

    let profile = AppContainerProfile::ensure(
        "rappct.paths.demo",
        "Path Demo",
        Some("Demonstration of profile path resolution"),
    )?;

    println!("✓ Created profile: {}", profile.name);

    // Get the profile's folder path
    match profile.folder_path() {
        Ok(folder_path) => {
            println!("✓ Profile folder path: {}", folder_path.display());
            println!("  This is where the AppContainer can store persistent data");
        }
        Err(e) => {
            println!("⚠ Could not get folder path: {}", e);
            println!("  This may be normal if the profile hasn't been used yet");
        }
    }

    // Get the named object path
    match profile.named_object_path() {
        Ok(named_path) => {
            println!("✓ Named object path: {}", named_path);
            println!("  This prefix is used for named kernel objects (mutexes, events, etc.)");
        }
        Err(e) => {
            println!("⚠ Could not get named object path: {}", e);
        }
    }

    profile.delete()?;
    println!("✓ Profile cleaned up\n");

    Ok(())
}

/// Demo 2: Direct SID Derivation
fn demo_sid_derivation() -> rappct::Result<()> {
    println!("=== DEMO 2: Direct SID Derivation ===");
    println!("You can derive AppContainer SIDs without creating full profiles");

    let profile_name = "rappct.sid.demo";

    println!("→ Deriving SID for profile name: '{}'", profile_name);
    let derived_sid = derive_sid_from_name(profile_name)?;
    println!("✓ Derived SID: {}", derived_sid.as_string());

    // Compare with full profile creation
    let profile = AppContainerProfile::ensure(profile_name, "SID Demo", None)?;
    println!("✓ Profile SID: {}", profile.sid.as_string());

    if derived_sid.as_string() == profile.sid.as_string() {
        println!("✓ SIDs match - derivation is consistent");
    } else {
        println!("✗ SIDs don't match - unexpected!");
    }

    profile.delete()?;
    println!("✓ Profile cleaned up\n");

    Ok(())
}

/// Demo 3: Custom Named Capabilities
fn demo_custom_capabilities() -> rappct::Result<()> {
    println!("=== DEMO 3: Custom Named Capabilities ===");
    println!("Beyond known capabilities, you can specify custom ones by name");

    let profile = AppContainerProfile::ensure(
        "rappct.custom.caps",
        "Custom Caps",
        Some("Custom capabilities demo"),
    )?;

    // Build capabilities with custom names
    println!("→ Building capabilities with custom names...");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_named(&[
            "documentsLibrary",         // Access to Documents folder
            "picturesLibrary",          // Access to Pictures folder
            "videosLibrary",            // Access to Videos folder
            "musicLibrary",             // Access to Music folder
            "enterpriseAuthentication", // Enterprise authentication
        ])
        .build();

    match caps {
        Ok(security_caps) => {
            println!("✓ Successfully built custom capabilities");
            println!("  • Package SID: {}", security_caps.package.as_string());
            println!("  • LPAC mode: {}", security_caps.lpac);
            println!("  • Capabilities count: {}", security_caps.caps.len());

            for (i, cap) in security_caps.caps.iter().enumerate() {
                println!(
                    "    {}. {} (attributes: 0x{:x})",
                    i + 1,
                    cap.sid_sddl,
                    cap.attributes
                );
            }
        }
        Err(e) => {
            println!("⚠ Custom capabilities failed: {}", e);
            println!("  Some capability names may not be recognized on this system");
        }
    }

    profile.delete()?;
    println!("✓ Profile cleaned up\n");

    Ok(())
}

/// Demo 4: Configuration Diagnostics
#[cfg(not(feature = "introspection"))]
fn demo_diagnostics() -> rappct::Result<()> {
    println!("=== DEMO 4: Configuration Diagnostics ===");
    println!("? Diagnostics require the 'introspection' feature");
    println!("  Run with: --features introspection");
    println!();
    Ok(())
}

#[cfg(feature = "introspection")]
fn demo_diagnostics() -> rappct::Result<()> {
    demo_diagnostics_old()
}

#[cfg(feature = "introspection")]
fn demo_diagnostics_old() -> rappct::Result<()> {
    println!("=== DEMO 4: Configuration Diagnostics ===");

    #[cfg(not(feature = "introspection"))]
    {
        println!("⚠ Diagnostics require the 'introspection' feature");
        println!("  Run with: --features introspection");
        println!();
        Ok(())
    }

    #[cfg(feature = "introspection")]
    {
        println!("The introspection feature provides configuration validation");

        let profile_guard = ProfileCleanupGuard::new(AppContainerProfile::ensure(
            "rappct.diag.demo",
            "Diagnostics Demo",
            None,
        )?);
        let profile_sid = profile_guard.profile().sid.clone();

        // Test 1: LPAC without common capabilities (should warn)
        println!("\n→ Test 1: LPAC without common capabilities");
        let lpac_caps = SecurityCapabilitiesBuilder::new(&profile_sid)
            .lpac(true) // Enable LPAC but don't add defaults
            .build()?;

        let launch_opts = LaunchOptions {
            exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            ..Default::default()
        };

        let warnings = validate_configuration(&lpac_caps, &launch_opts);
        if warnings.contains(&ConfigWarning::LpacWithoutCommonCaps) {
            println!("✓ Detected: LPAC without common capabilities");
            println!("  Recommendation: Use .with_lpac_defaults()");
        }

        // Test 2: No network capabilities (should warn)
        println!("\n→ Test 2: Configuration without network capabilities");
        let no_net_caps = SecurityCapabilitiesBuilder::new(&profile_sid).build()?;
        let warnings = validate_configuration(&no_net_caps, &launch_opts);
        if warnings.contains(&ConfigWarning::NoNetworkCaps) {
            println!("✓ Detected: No network capabilities");
            println!("  Recommendation: Add network capabilities if needed");
        }

        // Test 3: Well-configured LPAC (should not warn)
        println!("\n→ Test 3: Well-configured LPAC");
        let good_caps = SecurityCapabilitiesBuilder::new(&profile_sid)
            .with_known(&[KnownCapability::InternetClient])
            .with_lpac_defaults()
            .build()?;
        let warnings = validate_configuration(&good_caps, &launch_opts);
        if warnings.is_empty() {
            println!("✓ No warnings - configuration looks good");
        } else {
            println!("⚠ Warnings found: {:?}", warnings);
        }

        drop(profile_guard);
        Ok(())
    }
}

/// Demo 5: Advanced Launch Options
fn demo_advanced_launch() -> rappct::Result<()> {
    println!("=== DEMO 5: Advanced Launch Options ===");
    println!("Demonstrating suspended launch, custom environment, and timeouts");

    // First show normal process launching for comparison
    println!("\n→ Baseline: Normal process with custom environment");
    use std::process::Command;
    match Command::new("cmd")
        .arg("/C")
        .arg("echo Normal process: RAPPCT_DEMO=%RAPPCT_DEMO% && echo Normal process: PATH accessible")
        .env("RAPPCT_DEMO", "normal")
        .output() {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            println!("✓ Normal process: Custom environment and PATH work normally");
            if result.contains("normal") {
                println!("  • Environment variable: SUCCESS");
            }
        }
        Err(e) => println!("⚠ Normal process test error: {}", e),
    }

    println!("\n→ Now comparing with AppContainer restrictions:");

    let profile = AppContainerProfile::ensure("rappct.advanced.launch", "Advanced Launch", None)?;

    // Create custom environment
    let custom_env = vec![
        (OsString::from("RAPPCT_DEMO"), OsString::from("advanced")),
        (
            OsString::from("ISOLATION_LEVEL"),
            OsString::from("appcontainer"),
        ),
        (
            OsString::from("PATH"),
            OsString::from("C:\\Windows\\System32"),
        ),
    ];

    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;

    println!("→ Launching with custom environment and timeout...");
    let opts = LaunchOptions {
        exe: resolve_cmd_exe(),
        cmdline: Some("/C echo Environment: %RAPPCT_DEMO% && echo Isolation: %ISOLATION_LEVEL% && echo Advanced launch demo completed".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        env: Some(custom_env),
        suspended: false, // Could set to true for debugging
        startup_timeout: Some(Duration::from_secs(10)),
        join_job: Some(JobLimits {
            memory_bytes: Some(64 * 1024 * 1024), // 64 MB
            cpu_rate_percent: Some(25),           // 25% CPU
            kill_on_job_close: true,
        }),
        ..Default::default()
    };

    match launch_in_container(&caps, &opts) {
        Ok(child) => {
            println!("✓ Advanced launch successful, PID: {}", child.pid);
            println!("  Process has custom environment and resource limits");
            thread::sleep(Duration::from_secs(3));
        }
        Err(e) => {
            println!("⚠ Advanced launch failed: {}", e);
            println!("  This is normal in restricted AppContainer environments");
            println!("  The advanced APIs still work for profile/SID management");
        }
    }

    profile.delete()?;
    println!("✓ Profile cleaned up\n");

    Ok(())
}

/// Demo 6: Enhanced I/O with Error Handling
#[cfg(windows)]
fn demo_enhanced_io() -> rappct::Result<()> {
    println!("=== DEMO 6: Enhanced I/O with Error Handling ===");
    println!("Using launch_in_container_with_io for full process interaction");

    let profile = AppContainerProfile::ensure("rappct.io.demo", "I/O Demo", None)?;
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;

    println!("→ Launching process with full I/O redirection...");
    let opts = LaunchOptions {
        exe: resolve_cmd_exe(),
        cmdline: Some("/C echo [STDOUT] Hello from AppContainer && echo [STDERR] This is an error message 1>&2 && echo [STDOUT] Process completed".to_string()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    };

    let mut child_io = launch_in_container_with_io(&caps, &opts)?;
    println!("✓ Process launched with PID: {}", child_io.pid);

    if let Some(stdout) = child_io.stdout.take() {
        println!("\n→ Reading stdout:");
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(content) => println!("  STDOUT: {}", content),
                Err(e) => println!("  STDOUT read error: {}", e),
            }
        }
    }

    if let Some(stderr) = child_io.stderr.take() {
        println!("\n→ Reading stderr:");
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(content) => println!("  STDERR: {}", content),
                Err(e) => println!("  STDERR read error: {}", e),
            }
        }
    }

    child_io.wait(Some(Duration::from_secs(5)))?;

    profile.delete()?;
    println!("✓ Profile cleaned up\n");

    Ok(())
}

#[cfg(not(windows))]
fn demo_enhanced_io() -> rappct::Result<()> {
    println!("=== DEMO 6: Enhanced I/O with Error Handling ===");
    println!("Using launch_in_container_with_io for full process interaction");
    println!("⚠ Enhanced I/O demo requires Windows");
    Err(rappct::AcError::UnsupportedPlatform)
}

/// Demo 7: Network Container Enumeration (feature-gated wrappers)
#[cfg(not(feature = "net"))]
fn demo_network_enumeration() -> rappct::Result<()> {
    println!("=== DEMO 7: Network Container Enumeration ===");
    println!("? Network enumeration requires the 'net' feature");
    println!("  Run with: --features net");
    println!();
    Ok(())
}

#[cfg(feature = "net")]
fn demo_network_enumeration() -> rappct::Result<()> {
    demo_network_enumeration_impl()
}

#[cfg(feature = "net")]
fn demo_network_enumeration_impl() -> rappct::Result<()> {
// (moved into demo_network_enumeration_impl wrapper)
    println!("=== DEMO 7: Network Container Enumeration ===");

    #[cfg(not(feature = "net"))]
    {
        println!("⚠ Network enumeration requires the 'net' feature");
        println!("  Run with: --features net");
        println!();
        Ok(())
    }

    #[cfg(feature = "net")]
    {
        println!("Enumerating existing AppContainer profiles with network configuration");

        match list_appcontainers() {
            Ok(containers) => {
                println!("✓ Found {} AppContainer profiles:", containers.len());

                if containers.is_empty() {
                    println!("  (No containers found - this is normal on a fresh system)");
                } else {
                    for (i, (sid, display_name)) in containers.iter().enumerate().take(10) {
                        println!("  {}. {} - {}", i + 1, display_name, sid.as_string());
                    }

                    if containers.len() > 10 {
                        println!("  ... and {} more", containers.len() - 10);
                    }
                }
            }
            Err(e) => {
                println!("⚠ Enumeration failed: {}", e);
                println!("  This may require Administrator privileges");
            }
        }
        println!();
        Ok(())
    }
}

/// Demo 8: Capability-based ACLs
fn demo_capability_acls() -> rappct::Result<()> {
    println!("=== DEMO 8: Capability-based ACLs ===");
    println!("Granting file access to specific capabilities rather than the container");

    let profile = AppContainerProfile::ensure("rappct.cap.acl", "Capability ACL", None)?;

    // Create test file
    let test_file = env::temp_dir().join("capability_test.txt");
    fs::write(&test_file, "This file requires specific capability access").map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to create test file {}: {}",
            test_file.display(),
            e
        ))
    })?;

    // Build capabilities and get a specific capability SID
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;

    if !caps.caps.is_empty() {
        let cap_sid = &caps.caps[0].sid_sddl;
        println!("→ Granting access to capability: {}", cap_sid);

        match grant_to_capability(
            ResourcePath::File(test_file.clone()),
            cap_sid,
            AccessMask(0x00120089), // FILE_GENERIC_READ
        ) {
            Ok(_) => println!("✓ Capability-based ACL applied successfully"),
            Err(e) => println!("⚠ Capability ACL failed: {}", e),
        }
    } else {
        println!("⚠ No capabilities available for ACL demo");
    }

    // Cleanup
    fs::remove_file(&test_file).ok();
    profile.delete()?;
    println!("✓ Profile and test file cleaned up\n");

    Ok(())
}

/// Demo 9: LPAC Testing Environment
fn demo_lpac_testing() -> rappct::Result<()> {
    println!("=== DEMO 9: LPAC Testing Environment ===");
    println!("Demonstrating LPAC testing environment variable");

    // Show current LPAC support
    match supports_lpac() {
        Ok(_) => println!("✓ LPAC is natively supported on this system"),
        Err(_) => println!("✗ LPAC is not natively supported"),
    }

    // Demonstrate environment variable override
    println!("\n→ Testing environment variable override...");

    // This demo mutates process environment; run single-threaded or before starting any worker threads.
    set_env_override("RAPPCT_TEST_LPAC_STATUS", "unsupported");
    match supports_lpac() {
        Ok(_) => println!("✗ Expected LPAC to be unsupported with env var"),
        Err(_) => println!("✓ LPAC correctly forced as unsupported"),
    }

    // Test forcing LPAC as supported
    set_env_override("RAPPCT_TEST_LPAC_STATUS", "ok");
    match supports_lpac() {
        Ok(_) => println!("✓ LPAC correctly forced as supported"),
        Err(_) => println!("✗ Expected LPAC to be supported with env var"),
    }

    // Clear the environment variable
    clear_env_override("RAPPCT_TEST_LPAC_STATUS");
    match supports_lpac() {
        Ok(_) => println!("✓ Back to native LPAC support detection"),
        Err(_) => println!("✓ Back to native LPAC support detection (unsupported)"),
    }

    println!("✓ Environment variable testing complete\n");

    Ok(())
}
