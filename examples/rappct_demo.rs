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
            Err(e) => println!("? Firewall exemption cleanup failed: {}", e),
        }
    }
}

use std::path::PathBuf;
use std::net::TcpListener;
use std::io::Write;

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
    println!("- Created profile: {}", profile.name);
    println!("- Profile SID: {}", profile.sid.as_string());
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
        cmdline: Some("/C echo [ISOLATED] Running in AppContainer sandbox && echo [ISOLATED] No special permissions granted".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        ..Default::default()
    })?;
    println!("- Isolated process launched with PID: {}", isolated_child.pid);
    println!("  Watch the output above - the process is completely sandboxed.\n");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // 3. First show normal network access for comparison
    println!("STEP 3A: Normal Network Access (For Comparison)");
    println!("------------------------------------------------");
    println!("First, let's see network access from a normal (non-sandboxed) process:");
    println!("Expected: HTTP should succeed in a normal process (no sandbox).\n");

    // Quick test of normal network access using curl
    use std::process::Command;
    match Command::new("cmd")
        .arg("/C")
        .arg(r#"C:\Windows\System32\curl.exe -s -I -m 5 http://httpbin.org/ip && echo OK || echo FAILED"#)
        .output()
    {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            let result = result.trim();
            if result.contains("OK") {
                println!("- Normal process: HTTP request succeeded");
            } else {
                println!("? Normal process: HTTP request failed ({})", result);
            }
        }
        Err(e) => println!("? Normal process network test error: {}", e),
    }

    println!("\nSTEP 3B: Localhost (loopback) access");
    println!("------------------------------------");
    println!("Stand up a tiny localhost HTTP server, then try from AppContainer:");
    println!("  - Expect failure without loopback exemption");
    println!("  - Expect success after enabling loopback (with --features net)\n");

    // Start a minimal localhost HTTP server on an ephemeral port.
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind localhost");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for _ in 0..4 {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK",
                );
            } else {
                break;
            }
        }
    });

    // Build capabilities with InternetClient; loopback is controlled by firewall exemption.
    let network_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;

    // Attempt localhost without loopback exemption – expect failure.
    let curl_no_loopback = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
        cmdline: Some(format!(" -s -I -m 3 http://127.0.0.1:{}", port)),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        ..Default::default()
    };
    println!("- Trying http://127.0.0.1:{} without loopback exemption...", port);
    let _ = launch_in_container(&network_caps, &curl_no_loopback)?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    #[cfg(feature = "net")]
    let mut firewall_guard: Option<FirewallGuard> = None;

    #[cfg(feature = "net")]
    {
        println!("- Enabling loopback exemption for this AppContainer SID...");
        if let Err(e) = add_loopback_exemption(LoopbackAdd(profile.sid.clone()).confirm_debug_only())
        {
            println!("? Loopback exemption failed: {} (continuing anyway)", e);
        } else {
            firewall_guard = Some(FirewallGuard::new(
                profile.sid.clone(),
                "- Firewall loopback exemption removed",
            ));
            // Try localhost again – expect success (HTTP headers).
            let curl_with_loopback = LaunchOptions {
                exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
                cmdline: Some(format!(" -s -I -m 5 http://127.0.0.1:{}", port)),
                cwd: Some(PathBuf::from("C:\\Windows\\System32")),
                ..Default::default()
            };
            println!("- Trying http://127.0.0.1:{} with loopback exemption...", port);
            let _ = launch_in_container(&network_caps, &curl_with_loopback)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    #[cfg(not(feature = "net"))]
    {
        println!("(Enable --features net to allow localhost by adding a loopback exemption)");
    }

    println!("\nSTEP 3C: Outbound Internet from AppContainer");
    println!("-------------------------------------------");
    println!("Now demonstrate an outbound HTTP request (does not require loopback)");
    let internet_curl = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
        cmdline: Some(" -s -I -m 5 http://httpbin.org/ip".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        ..Default::default()
    };
    let _ = launch_in_container(&network_caps, &internet_curl)?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    let network_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;
    let network_child = launch_in_container(&network_caps, &LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
        cmdline: Some(" -s -I -m 5 http://httpbin.org/ip".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        ..Default::default()
    })?;
    println!("- Network-enabled process launched with PID: {}", network_child.pid);
    println!("  curl will print HTTP headers if successful\n");

    std::thread::sleep(std::time::Duration::from_secs(3));

    // Cleanup
    println!("STEP 4: Cleanup");
    println!("---------------");
    println!("Cleaning up firewall exemptions and deleting the AppContainer profile.");

    #[cfg(feature = "net")]
    {
        let _firewall_guard = firewall_guard; // auto-clean on drop
    }

    let profile_name = profile.name.clone();
    profile.delete()?;
    println!("- Profile '{}' deleted successfully", profile_name);

    println!("\nDemo Complete!");
    println!("=================");
    println!("You've seen how rappct can:");
    println!("- Create secure AppContainer profiles");
    println!("- Launch completely isolated processes");
    println!("- Grant specific capabilities (like network access)");
    println!("- Clean up resources when done");
    println!("Pro tip: Use '--features net' when you need localhost (loopback):");
    println!("  - Adds helpers to grant/remove AppContainer loopback exemptions
  - Usually requires Administrator rights
  - Not required for outbound internet (InternetClient is enough)\n
  Example: cargo run --example rappct_demo --features net");

    Ok(())
}
