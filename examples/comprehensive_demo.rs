//! Comprehensive rappct demonstration with individual capability examples
//!
//! This example provides clear, isolated demonstrations of each rappct capability
//! followed by a combined example showing how to use multiple features together.
//! Designed for easy developer adoption and understanding.
//!
//! ## Important: PowerShell in AppContainers (Demo 4)
//!
//! Demo 4 (Network Capabilities) redirects PowerShell output to temporary files
//! to avoid Error 0x5 "Access is denied" when accessing the console output buffer.
//! AppContainers restrict console buffer access for security. The pattern used:
//! 1. Create a unique task directory below this worktree's `.tmp/`
//! 2. Grant ACL access only to that directory for the AppContainer profile
//! 3. Redirect PowerShell output: `Out-File -FilePath '{temp_file}' -Encoding ASCII`
//! 4. Read back with cmd: `type "{temp_file}"`
//! 5. Clean up the exact task directory

use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
    launch::{JobLimits, LaunchOptions},
    launch_in_container, supports_lpac,
    token::query_current_process_token,
};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

#[cfg(windows)]
use rappct::launch::{StdioConfig, launch_in_container_with_io};

#[cfg(windows)]
use std::io::{BufRead, BufReader};

type DemoEntry = (&'static str, fn() -> rappct::Result<()>);

#[path = "comprehensive_demo/file_acls.rs"]
mod file_acls;
#[path = "comprehensive_demo/network.rs"]
mod network;
#[path = "comprehensive_demo/web_scraper.rs"]
mod web_scraper;

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

fn cleanup_repo_local_tempdir(scratch: tempfile::TempDir) -> rappct::Result<()> {
    let path = scratch.path().to_path_buf();
    scratch.close().map_err(|error| {
        rappct::AcError::Win32(format!(
            "Failed to clean repository-local task directory {}: {error}",
            path.display()
        ))
    })
}

/// Helper function to pause and wait for user input
fn pause_for_demo(msg: &str) {
    println!("\n{msg}");
    print!("Press Enter to continue...");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

/// Demo 1: Basic Profile Management
/// Shows how to create, query, and delete AppContainer profiles
fn demo_profile_management() -> rappct::Result<()> {
    println!("Expected: Profile ensure/derive/delete should succeed.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║     DEMO 1: AppContainer Profile Management    ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile_name = "rappct.demo.profile";

    // Create or ensure profile exists
    println!("\n→ Creating AppContainer profile: '{profile_name}'");
    let profile = AppContainerProfile::ensure(
        profile_name,
        "Demo Profile",
        Some("Example profile for rappct demonstration"),
    )?;

    println!("✓ Profile created/opened successfully");
    println!("  • Name: {}", profile.name);
    println!("  • SID: {}", profile.sid.as_string());

    // Demonstrate deriving SID from name
    println!("\n→ Deriving SID from profile name...");
    let derived_sid = rappct::profile::derive_sid_from_name(profile_name)?;
    println!("✓ Derived SID: {}", derived_sid.as_string());
    println!(
        "  • Matches original: {}",
        derived_sid.as_string() == profile.sid.as_string()
    );

    // Clean up
    pause_for_demo("Profile will be deleted after viewing");
    profile.delete()?;
    println!("✓ Profile deleted successfully");

    Ok(())
}

/// Demo 2: Token Introspection
/// Shows how to query security token information
fn demo_token_introspection() -> rappct::Result<()> {
    println!("Expected: Shows current token; outside container typically not in AppContainer.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║        DEMO 2: Token Introspection             ║");
    println!("╚════════════════════════════════════════════════╝");

    println!("\n→ Querying current process token...");
    let token_info = query_current_process_token()?;

    println!("✓ Current Process Security Context:");
    println!(
        "  • Running in AppContainer: {}",
        token_info.is_appcontainer
    );
    println!("  • Running in LPAC: {}", token_info.is_lpac);

    if let Some(sid) = &token_info.package_sid {
        println!("  • Package SID: {}", sid.as_string());
    } else {
        println!("  • Package SID: None (not in container)");
    }

    if !token_info.capability_sids.is_empty() {
        println!("  • Capabilities ({}):", token_info.capability_sids.len());
        for cap in &token_info.capability_sids {
            println!("    - {cap}");
        }
    } else {
        println!("  • Capabilities: None");
    }

    Ok(())
}

/// Demo 3: Basic Container Launch
/// Shows minimal AppContainer process launching
fn demo_basic_launch() -> rappct::Result<()> {
    println!("Expected: Isolated cmd.exe runs with no network/file/registry access.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║      DEMO 3: Basic Container Launch            ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile = AppContainerProfile::ensure(
        "rappct.demo.basic",
        "Basic Demo",
        Some("Basic launch demonstration"),
    )?;

    println!("\n→ Building security capabilities (no special permissions)...");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;

    println!("→ Launching isolated cmd.exe in AppContainer...");
    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some("/C echo [BASIC] Hello from isolated AppContainer && echo [BASIC] No network or file access! && timeout /T 3 /NOBREAK >nul".to_string()),
        ..Default::default()
    };

    let child = launch_in_container(&caps, &opts)?;
    println!("✓ Process launched with PID: {}", child.pid);
    println!("  • Running in complete isolation");
    println!("  • No network, file, or registry access");

    thread::sleep(Duration::from_secs(4));
    profile.delete()?;

    Ok(())
}

/// Demo 6: LPAC (Low Privilege AppContainer)
/// Shows LPAC mode with enhanced but still restricted capabilities
fn demo_lpac() -> rappct::Result<()> {
    println!("Expected: Notepad launches under LPAC; limited registry/COM access.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║    DEMO 6: Low Privilege AppContainer (LPAC)   ║");
    println!("╚════════════════════════════════════════════════╝");

    // Check LPAC support
    if supports_lpac().is_err() {
        println!("\n⚠ LPAC not supported on this system");
        println!("  Requires Windows 10 version 1703 or later");
        println!("  💡 CI-only override support requires the private _test_helpers feature");
        return Ok(());
    }

    println!("\n✓ LPAC is supported on this system");

    let profile =
        AppContainerProfile::ensure("rappct.demo.lpac", "LPAC Demo", Some("LPAC demonstration"))?;

    println!("\n→ Building LPAC capabilities...");
    println!("  LPAC provides limited access to:");
    println!("  • Registry (read-only)");
    println!("  • COM objects (lpacCom)");
    println!("  • Some system resources");

    let lpac_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults() // Adds registryRead, lpacCom, etc.
        .build()?;

    println!("\n→ Launching Notepad in LPAC mode...");
    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\notepad.exe"),
        ..Default::default()
    };

    let child = launch_in_container(&lpac_caps, &opts)?;
    println!("✓ LPAC process launched with PID: {}", child.pid);
    println!("  • Has limited registry read access");
    println!("  • Can use certain COM objects");
    println!("  • Still isolated from most system resources");

    pause_for_demo("Close Notepad when ready");
    profile.delete()?;

    Ok(())
}

/// Demo 7: Job Objects and Resource Limits
/// Shows how to apply CPU and memory limits
fn demo_job_limits() -> rappct::Result<()> {
    println!("Expected: Process launches with memory/CPU constraints enforced by job object.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║    DEMO 7: Job Objects & Resource Limits       ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile = AppContainerProfile::ensure(
        "rappct.demo.jobs",
        "Job Demo",
        Some("Resource limits demonstration"),
    )?;

    println!("\n→ Configuring resource limits:");
    println!("  • Memory limit: 50 MB");
    println!("  • CPU limit: 25% (1/4 of one core)");
    println!("  • Kill on job close: Yes");

    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;

    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some("/C echo [LIMITS] Running with resource limits && echo [LIMITS] Memory: max 50MB && echo [LIMITS] CPU: max 25 percent && timeout /T 5 /NOBREAK >nul".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: Some(50 * 1024 * 1024),  // 50 MB
            cpu_rate_percent: Some(25),            // 25% CPU
            kill_on_job_close: true,
        }),
        ..Default::default()
    };

    let child = launch_in_container(&caps, &opts)?;
    println!(
        "✓ Resource-limited process launched with PID: {}",
        child.pid
    );
    println!("  Process is now constrained by job object limits");

    thread::sleep(Duration::from_secs(6));
    profile.delete()?;

    Ok(())
}

/// Demo 8: Process I/O Redirection
/// Shows how to capture process output through pipes
#[cfg(windows)]
fn demo_io_redirection() -> rappct::Result<()> {
    println!("Expected: Captures child stdout/stderr via pipes.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║      DEMO 8: Process I/O Redirection           ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile = AppContainerProfile::ensure(
        "rappct.demo.io",
        "I/O Demo",
        Some("I/O redirection demonstration"),
    )?;

    println!("\n→ Launching process with piped I/O...");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;

    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some("/C echo [PIPE] Line 1 from container && echo [PIPE] Line 2 from container && echo [PIPE] Error line 1>&2".to_string()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    };

    let mut child_io = launch_in_container_with_io(&caps, &opts)?;
    println!("✓ Process launched with PID: {}", child_io.pid);

    println!("\n→ Reading piped output:");

    if let Some(stdout) = child_io.stdout.take() {
        let reader = BufReader::new(stdout);
        println!("  STDOUT:");
        for line in reader.lines().map_while(Result::ok) {
            println!("    > {line}");
        }
    }

    if let Some(stderr) = child_io.stderr.take() {
        let reader = BufReader::new(stderr);
        println!("  STDERR:");
        for line in reader.lines().map_while(Result::ok) {
            println!("    > {line}");
        }
    }

    profile.delete()?;
    Ok(())
}

#[cfg(not(windows))]
fn demo_io_redirection() -> rappct::Result<()> {
    println!("Expected: Captures child stdout/stderr via pipes.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║      DEMO 8: Process I/O Redirection           ║");
    println!("╚════════════════════════════════════════════════╝");
    println!("⚠ Process I/O redirection demo requires Windows");
    Err(rappct::AcError::UnsupportedPlatform)
}

/// Main entry point - runs all demos
fn main() -> rappct::Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║            rappct - Windows AppContainer Toolkit              ║");
    println!("║                  Comprehensive Demo Suite                     ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    println!("\nThis demo suite showcases all major capabilities of rappct:");
    println!("• Profile Management      • Token Introspection");
    println!("• Container Launching     • Network Capabilities");
    println!("• File System ACLs       • LPAC Mode");
    println!("• Resource Limits        • I/O Redirection");
    println!("• Comprehensive Example");

    pause_for_demo("\nReady to start demos?");

    // Run each demo with error handling
    let demos: Vec<DemoEntry> = vec![
        ("Profile Management", demo_profile_management),
        ("Token Introspection", demo_token_introspection),
        ("Basic Container Launch", demo_basic_launch),
        ("Network Capabilities", network::demo_network_capabilities),
        ("File System ACLs", file_acls::demo_file_acls),
        ("LPAC Mode", demo_lpac),
        ("Job Objects & Resource Limits", demo_job_limits),
        ("Process I/O Redirection", demo_io_redirection),
        ("Comprehensive Example", web_scraper::demo_comprehensive),
    ];

    for (name, demo_fn) in demos {
        match demo_fn() {
            Ok(_) => println!("\n✓ {name} completed successfully"),
            Err(e) => {
                println!("\n✗ {name} failed: {e}");
                println!("  Continuing with next demo...");
            }
        }

        if name != "Comprehensive Example" {
            thread::sleep(Duration::from_secs(1));
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    All Demos Complete!                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    println!("\nYou've seen rappct's full capabilities for:");
    println!("✓ Creating secure sandboxes with AppContainer");
    println!("✓ Managing granular permissions and capabilities");
    println!("✓ Enforcing resource limits");
    println!("✓ Controlling file system access");
    println!("✓ Running processes in LPAC mode");
    println!("✓ Capturing process I/O");

    println!("\nFor production use, consider:");
    println!("• Run with administrative privileges for full functionality");
    println!("• Test on Windows 10 1703+ for LPAC support");
    println!("• Review Windows Firewall settings for network features");
    println!("• Use appropriate error handling for all operations");

    Ok(())
}
