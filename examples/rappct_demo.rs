//! Simple rappct demonstration program
//!
//! This example shows the essential features of rappct:
//! - Creating AppContainer profiles
//! - Launching sandboxed processes
//! - Granting specific capabilities
//! - Automatic network configuration (with 'net' feature)

#[cfg(windows)]
use rappct::launch::launch_in_container_with_io;
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

use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;

fn main() -> rappct::Result<()> {
    println!("rappct - Windows AppContainer Demo");
    println!("===================================\n");

    println!("This demo shows how to sandbox processes using Windows AppContainer technology.");
    println!("AppContainers enforce security at the OS level - similar to Linux containers.\n");

    // 1. Create a profile
    println!("STEP 1: Creating AppContainer Profile");
    println!("--------------------------------------");
    println!("A profile defines a security boundary. Windows assigns each one a unique SID");
    println!("(Security Identifier) that controls what sandboxed processes can access.\n");

    let profile =
        AppContainerProfile::ensure("demo.app", "Demo Application", Some("rappct demonstration"))?;
    println!("✓ Created profile: {}", profile.name);
    println!("  SID: {}", profile.sid.as_string());
    println!("  (This SID identifies our sandbox and governs all access checks)\n");

    // 2. Launch isolated process (no capabilities)
    println!("STEP 2: Launching Process with Zero Capabilities");
    println!("-------------------------------------------------");
    println!("Launching with NO capabilities = maximum isolation.");
    println!("The process can run but has virtually no access to files, network, or registry.\n");

    let isolated_caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;
    let isolated_child = launch_in_container(&isolated_caps, &LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some("/C echo [ISOLATED] Running in AppContainer sandbox && echo [ISOLATED] No special permissions granted".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        ..Default::default()
    })?;
    println!("✓ Sandboxed process launched (PID: {})", isolated_child.pid);
    println!("  If it printed messages, the sandbox is working!\n");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // 3. Test outbound HTTP without network capability (should fail)
    println!("STEP 3: Outbound HTTP Without Network Capability");
    println!("-------------------------------------------------");
    println!("Testing outbound HTTP from AppContainer with NO network capabilities.");
    println!("Expected: HTTP should be blocked, demonstrating network isolation.\n");

    #[cfg(windows)]
    {
        let no_net_caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;
        let no_net_curl = LaunchOptions {
            exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
            cmdline: Some(" -s -I -f -m 5 http://example.com".to_string()),
            cwd: Some(PathBuf::from("C:\\Windows\\System32")),
            stdio: rappct::launch::StdioConfig::Pipe,
            ..Default::default()
        };
        println!("→ Trying HTTP request without InternetClient capability...");
        let mut child = launch_in_container_with_io(&no_net_caps, &no_net_curl)?;
        let mut out = String::new();
        if let Some(mut s) = child.stdout.take() {
            let _ = s.read_to_string(&mut out);
        }
        let code = child.wait(Some(std::time::Duration::from_secs(6)))?;
        if code == 0 {
            println!(
                "✗ Unexpected! HTTP succeeded without network capability. Output:\n{}",
                out
            );
        } else {
            println!(
                "✓ Blocked as expected (exit {}). Network isolation is working.",
                code
            );
        }
    }
    #[cfg(not(windows))]
    {
        println!("⚠ Skipped on non-Windows platform");
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    // 4. First show normal network access for comparison
    println!("\nSTEP 4A: Baseline - Network Access Without Sandbox");
    println!("---------------------------------------------------");
    println!("First, let's test HTTP from the host (unsandboxed) as a baseline.\n");

    // Quick test of normal network access using curl
    use std::process::Command;
    match Command::new(r"C:\Windows\System32\curl.exe")
        .args(["-s", "-I", "-f", "-m", "5", "http://example.com"])
        .output()
    {
        Ok(output) => {
            let code = output.status.code().unwrap_or(0);
            if code == 0 {
                println!(
                    "✓ Host network access succeeded (exit 0). Headers:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                );
            } else if code == 22 {
                // curl -f returns 22 for HTTP errors (4xx, 5xx), but connection worked
                println!(
                    "⚠ HTTP error from server (exit 22), but network is working. Headers:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                );
            } else {
                println!(
                    "✗ Connection failed (exit {}). Output:\n{}",
                    code,
                    String::from_utf8_lossy(&output.stdout)
                );
            }
        }
        Err(e) => println!("✗ Network test error: {}", e),
    }

    println!("\nSTEP 4B: Sandboxed Localhost Access");
    println!("------------------------------------");
    println!("Starting a local HTTP server, then trying to reach it from inside the sandbox.");
    println!("By default, AppContainers BLOCK loopback (127.0.0.1) even with InternetClient.");
    println!("The 'net' feature allows adding a firewall exemption to permit loopback.\n");

    // Start a minimal localhost HTTP server on an ephemeral port.
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind localhost");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for _ in 0..4 {
            if let Ok((mut stream, _)) = listener.accept() {
                // Read request headers to ensure curl has sent them
                let mut buf = [0u8; 512];
                let _ = stream.read(&mut buf);
                // Send response
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK",
                );
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
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
        stdio: rappct::launch::StdioConfig::Pipe,
        ..Default::default()
    };
    println!(
        "→ Testing http://127.0.0.1:{} WITHOUT loopback exemption...",
        port
    );
    #[cfg(windows)]
    {
        let mut child = launch_in_container_with_io(&network_caps, &curl_no_loopback)?;
        let mut out = String::new();
        if let Some(mut s) = child.stdout.take() {
            let _ = s.read_to_string(&mut out);
        }
        let code = child.wait(Some(std::time::Duration::from_secs(4)))?;
        if code == 0 {
            println!(
                "✗ Unexpected! Localhost succeeded without exemption. Output:\n{}",
                out
            );
        } else {
            println!(
                "✓ Blocked as expected (exit {}). AppContainers deny loopback by default.",
                code
            );
        }
    }
    #[cfg(not(windows))]
    {
        let _ = launch_in_container(&network_caps, &curl_no_loopback)?;
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    #[cfg(feature = "net")]
    let mut firewall_guard: Option<FirewallGuard> = None;

    #[cfg(feature = "net")]
    {
        println!("\n→ Adding firewall exemption to allow loopback for this container...");
        if let Err(e) =
            add_loopback_exemption(LoopbackAdd(profile.sid.clone()).confirm_debug_only())
        {
            println!("✗ Exemption failed: {} (continuing anyway)", e);
        } else {
            firewall_guard = Some(FirewallGuard::new(
                profile.sid.clone(),
                "✓ Firewall loopback exemption removed",
            ));
            // Give the firewall exemption a moment to propagate
            std::thread::sleep(std::time::Duration::from_millis(1000));
            // Try localhost again – expect success (HTTP headers).
            let curl_with_loopback = LaunchOptions {
                exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
                cmdline: Some(format!(" -s -I -m 5 http://127.0.0.1:{}", port)),
                cwd: Some(PathBuf::from("C:\\Windows\\System32")),
                stdio: rappct::launch::StdioConfig::Pipe,
                ..Default::default()
            };
            println!(
                "→ Testing http://127.0.0.1:{} WITH loopback exemption...",
                port
            );
            let mut child = launch_in_container_with_io(&network_caps, &curl_with_loopback)?;
            let mut out = String::new();
            if let Some(mut s) = child.stdout.take() {
                let _ = s.read_to_string(&mut out);
            }
            let code = child.wait(Some(std::time::Duration::from_secs(5)))?;
            if code == 0 {
                println!("✓ Success! (exit 0). Headers:\n{}", out);
            } else {
                println!("✗ Still failed (exit {}). Output:\n{}", code, out);
            }
        }
    }

    #[cfg(not(feature = "net"))]
    {
        println!("\n  (Run with --features net to test loopback exemption)");
    }

    println!("\nSTEP 4C: Sandboxed Outbound Internet Access");
    println!("--------------------------------------------");
    println!("Now testing outbound HTTP with InternetClient. No firewall exemption needed.");
    let internet_curl = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
        cmdline: Some(" -s -I -m 5 http://example.com".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        stdio: rappct::launch::StdioConfig::Pipe,
        ..Default::default()
    };
    #[cfg(windows)]
    {
        let mut child = launch_in_container_with_io(&network_caps, &internet_curl)?;
        let mut out = String::new();
        if let Some(mut s) = child.stdout.take() {
            let _ = s.read_to_string(&mut out);
        }
        let code = child.wait(Some(std::time::Duration::from_secs(6)))?;
        if code == 0 {
            println!(
                "✓ Outbound HTTP succeeded from sandbox (exit 0). Headers:\n{}",
                out
            );
        } else {
            println!(
                "✗ Outbound request failed (exit {}). Output:\n{}",
                code, out
            );
        }
    }
    #[cfg(not(windows))]
    {
        let _ = launch_in_container(&network_caps, &internet_curl)?;
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // Cleanup
    println!("\nSTEP 5: Cleanup");
    println!("---------------");
    println!("Removing firewall exemptions and deleting the AppContainer profile.");

    #[cfg(feature = "net")]
    {
        let _firewall_guard = firewall_guard; // auto-clean on drop
    }

    let profile_name = profile.name.clone();
    profile.delete()?;
    println!("✓ Profile '{}' deleted successfully", profile_name);

    println!("\n════════════════════════════════════");
    println!("Demo Complete!");
    println!("════════════════════════════════════");
    println!("\nWhat you've seen:");
    println!("  • Create isolated AppContainer profiles");
    println!("  • Launch processes in maximum-security sandbox");
    println!("  • Grant specific capabilities (InternetClient)");
    println!("  • Control loopback access via firewall exemptions");
    println!("  • Clean up all resources automatically");
    println!("\nKey takeaways:");
    println!("  • Loopback (localhost) is BLOCKED by default in AppContainers");
    println!("  • The 'net' feature adds loopback exemption helpers (needs admin)");
    println!("  • Outbound internet only needs InternetClient - no exemption required");
    println!("\nTry it: cargo run --example rappct_demo --features net");

    Ok(())
}
