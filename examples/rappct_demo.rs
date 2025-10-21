//! Simple rappct demonstration program
//!
//! This example shows the essential features of rappct:
//! - Creating AppContainer profiles
//! - Launching sandboxed processes
//! - Granting specific capabilities
//! - Automatic network configuration (with 'net' feature)

use rappct::{
    launch::LaunchOptions, launch_in_container, AppContainerProfile, KnownCapability,
    SecurityCapabilitiesBuilder,
};

#[cfg(feature = "net")]
use rappct::net::{add_loopback_exemption, remove_loopback_exemption, LoopbackAdd};

#[cfg(feature = "net")]
struct FirewallGuard {
    sid: rappct::sid::AppContainerSid,
    success: &'static str,
}

#[cfg(feature = "net")]
impl FirewallGuard {
    fn new(sid: rappct::sid::AppContainerSid, success: &'static str) -> Self {
        Self { sid, success }
    }
}

#[cfg(feature = "net")]
impl Drop for FirewallGuard {
    fn drop(&mut self) {
        match remove_loopback_exemption(&self.sid) {
            Ok(_) => println!("{}", self.success),
            Err(e) => println!("⚠ Firewall exemption cleanup failed: {}", e),
        }
    }
}

use std::path::PathBuf;

fn main() -> rappct::Result<()> {
    println!("rappct - Windows AppContainer Demo");
    println!("===================================\n");

    println!(
        "This demo shows how rappct creates secure sandboxes using Windows AppContainer technology."
    );
    println!(
        "AppContainers provide process-level isolation similar to containers on other platforms.\n"
    );

    // 1. Create a profile
    println!("STEP 1: Creating AppContainer Profile");
    println!("--------------------------------------");
    println!("AppContainer profiles define the security boundary for sandboxed processes.");
    println!(
        "Each profile gets a unique SID (Security Identifier) that Windows uses for access control.\n"
    );

    let profile =
        AppContainerProfile::ensure("demo.app", "Demo Application", Some("rappct demonstration"))?;
    println!("✓ Created profile: {}", profile.name);
    println!("✓ Profile SID: {}", profile.sid.as_string());
    println!("  This SID uniquely identifies our sandbox and controls what it can access.\n");

    // 2. Launch isolated process (no capabilities)
    println!("STEP 2: Launching Completely Isolated Process");
    println!("----------------------------------------------");
    println!("Creating a process with NO capabilities - maximum isolation.");
    println!("This process will have extremely limited access to system resources.");
    println!("Expected: Process runs but has no network, file system, or registry access.\n");

    let isolated_caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;
    let isolated_child = launch_in_container(&isolated_caps, &LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some("/C echo [ISOLATED] Running in AppContainer sandbox && echo [ISOLATED] No special permissions granted && timeout /T 2 /NOBREAK >nul".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")), // Set valid working directory
        ..Default::default()
    })?;
    println!(
        "✓ Isolated process launched with PID: {}",
        isolated_child.pid
    );
    println!("  Watch the output above - the process is completely sandboxed.\n");

    // Wait for first process
    std::thread::sleep(std::time::Duration::from_secs(3));

    // 3. First show normal network access for comparison
    println!("STEP 3A: Normal Network Access (For Comparison)");
    println!("------------------------------------------------");
    println!("First, let's see network access from a normal (non-sandboxed) process:");
    println!("Expected: HTTP should succeed in a normal process (no sandbox).\n");

    // Quick test of normal network access
    use std::process::Command;
    match Command::new("powershell")
        .arg("-Command")
        .arg("try { (Invoke-WebRequest -Uri 'http://httpbin.org/ip' -UseBasicParsing -TimeoutSec 3).StatusCode } catch { 'Failed' }")
        .output() {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            let result = result.trim();
            if result.contains("200") {
                println!("✓ Normal process: HTTP request succeeded ({})", result);
            } else {
                println!("⚠ Normal process: HTTP request failed ({})", result);
            }
        }
        Err(e) => println!("⚠ Normal process network test error: {}", e),
    }

    println!("\nSTEP 3B: AppContainer with Network Access");
    println!("------------------------------------------");
    println!("Now granting 'InternetClient' capability to allow outbound network connections.");
    println!("Compare this result with the normal process above:");
    println!("Expected: With InternetClient, HTTP should work inside the AppContainer.\n");

    #[cfg(feature = "net")]
    let mut firewall_guard: Option<FirewallGuard> = None;

    #[cfg(feature = "net")]
    {
        println!("Setting up automatic firewall loopback exemption for better network access...");
        if let Err(e) =
            add_loopback_exemption(LoopbackAdd(profile.sid.clone()).confirm_debug_only())
        {
            println!("⚠ Firewall exemption failed: {} (continuing anyway)", e);
        } else {
            println!("✓ Firewall loopback exemption configured");
            firewall_guard = Some(FirewallGuard::new(
                profile.sid.clone(),
                "✓ Firewall loopback exemption removed",
            ));
        }
    }

    #[cfg(not(feature = "net"))]
    {
        println!("Note: Run with '--features net' for automatic firewall configuration");
    }

    println!("Note: ICMP ping may still fail on some systems even when HTTP works.\n");

    let network_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;
    let network_child = launch_in_container(&network_caps, &LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some(r#"/C echo [NETWORK] Process with internet access && powershell -Command "$urls=@('http://httpbin.org/ip','http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; foreach($u in $urls){ try { $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5; if($r.StatusCode){ $code=$r.StatusCode; break } } catch {} }; if($code){ $code } else { 'HTTP failed' }" && echo [NETWORK] Test completed"#.to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")), // Set valid working directory
        ..Default::default()
    })?;
    println!(
        "✓ Network-enabled process launched with PID: {}",
        network_child.pid
    );
    println!("  Watch above - HTTP request should work with proper network capability\n");

    // Wait for processes to complete
    std::thread::sleep(std::time::Duration::from_secs(6));

    // Cleanup
    println!("STEP 4: Cleanup");
    println!("---------------");
    println!("Cleaning up firewall exemptions and deleting the AppContainer profile.");

    #[cfg(feature = "net")]
    {
        // FirewallGuard will auto-cleanup on drop
        let _firewall_guard = firewall_guard; // keep scope explicit
    }

    let profile_name = profile.name.clone();
    profile.delete()?;
    println!("✓ Profile '{}' deleted successfully", profile_name);

    println!("\n🎉 Demo Complete!");
    println!("=================");
    println!("You've seen how rappct can:");
    println!("• Create secure AppContainer profiles");
    println!("• Launch completely isolated processes");
    println!("• Grant specific capabilities (like network access)");
    println!("• Clean up resources when done");
    println!("\nThe key insight: Windows AppContainer provides strong isolation");
    println!("while allowing you to grant exactly the permissions needed.");
    println!("\n💡 Pro tip: Use '--features net' for automatic firewall configuration:");
    println!("   cargo run --example rappct_demo --features net");

    Ok(())
}
