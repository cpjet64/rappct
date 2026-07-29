//! Advanced rappct features demonstration
//!
//! This example covers the less commonly used but powerful features of rappct:
//! - Profile path resolution (folder_path, named_object_path)
//! - Custom named capabilities
//! - Configuration diagnostics
//! - Advanced launch options (custom environment - see Demo 5 for Error 203 fix)
//! - Network enumeration
//! - Direct SID derivation
//!
//! ## Important: Custom Environment Pattern (Demo 5)
//!
//! When using `LaunchOptions::env`, Windows **completely replaces** the parent environment.
//! You MUST include essential system variables (SystemRoot, ComSpec, PATHEXT, PATH)
//! or Windows processes will fail with Error 203: "The system could not find the environment option".
//! Set TEMP and TMP explicitly to unique task-owned paths below the active worktree's `.tmp/`
//! instead of copying a system or user temp path.
//!
//! See Demo 5 for the correct pattern: copy essential vars from parent, then add custom vars.

use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
    profile::derive_sid_from_name,
};

use std::{env, fs, path::PathBuf};

type DemoEntry = (&'static str, fn() -> rappct::Result<()>);

#[path = "advanced_features/diagnostics.rs"]
mod diagnostics;
#[path = "advanced_features/launching.rs"]
mod launching;
#[path = "advanced_features/network_acl.rs"]
mod network_acl;

fn repo_local_tempdir(scope: &str) -> rappct::Result<tempfile::TempDir> {
    let scratch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join("examples")
        .join(scope);
    fs::create_dir_all(&scratch_root).map_err(|error| {
        rappct::AcError::Win32(format!(
            "Failed to create repository-local scratch {}: {error}",
            scratch_root.display()
        ))
    })?;
    tempfile::Builder::new()
        .prefix("run-")
        .tempdir_in(&scratch_root)
        .map_err(|error| {
            rappct::AcError::Win32(format!(
                "Failed to create repository-local task directory in {}: {error}",
                scratch_root.display()
            ))
        })
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
        let p = PathBuf::from(format!(r"{root}\System32\cmd.exe"));
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
        ("Configuration Diagnostics", diagnostics::demo_diagnostics),
        ("Advanced Launch Options", launching::demo_advanced_launch),
        (
            "Enhanced I/O with Error Handling",
            launching::demo_enhanced_io,
        ),
        (
            "Network Container Enumeration",
            network_acl::demo_network_enumeration,
        ),
        ("Capability-based ACLs", network_acl::demo_capability_acls),
        ("LPAC Testing Environment", network_acl::demo_lpac_testing),
    ];
    for (name, f) in demos {
        match f() {
            Ok(_) => println!("\n✓ {name} completed"),
            Err(e) => {
                println!("\n⚠ {name} failed: {e}");
                if let Some(src) = std::error::Error::source(&e) {
                    println!("   OS error: {src}");
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
            println!("⚠ Could not get folder path: {e}");
            println!("  This may be normal if the profile hasn't been used yet");
        }
    }

    // Get the named object path
    match profile.named_object_path() {
        Ok(named_path) => {
            println!("✓ Named object path: {named_path}");
            println!("  This prefix is used for named kernel objects (mutexes, events, etc.)");
        }
        Err(e) => {
            println!("⚠ Could not get named object path: {e}");
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

    println!("→ Deriving SID for profile name: '{profile_name}'");
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
            println!("⚠ Custom capabilities failed: {e}");
            println!("  Some capability names may not be recognized on this system");
        }
    }

    profile.delete()?;
    println!("✓ Profile cleaned up\n");

    Ok(())
}
