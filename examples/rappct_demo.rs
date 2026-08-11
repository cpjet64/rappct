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
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder, launch::LaunchOptions,
    launch_in_container,
};

#[cfg(all(windows, feature = "net"))]
use rappct::net::{LoopbackAdd, add_loopback_exemption, remove_loopback_exemption};

#[cfg(all(windows, feature = "net"))]
struct FirewallGuard {
    sid: rappct::sid::AppContainerSid,
    success: &'static str,
}

#[cfg(all(windows, feature = "net"))]
impl FirewallGuard {
    fn new(sid: rappct::sid::AppContainerSid, success: &'static str) -> Self {
        Self { sid, success }
    }
}

#[cfg(all(windows, feature = "net"))]
impl Drop for FirewallGuard {
    fn drop(&mut self) {
        match remove_loopback_exemption(&self.sid) {
            Ok(_) => println!("{}", self.success),
            Err(e) => println!("? Firewall exemption cleanup failed: {e}"),
        }
    }
}

use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> rappct::Result<()> {
    print_intro();
    let profile = create_demo_profile()?;
    demo_zero_capability_launch(&profile)?;
    demo_http_without_network(&profile)?;
    demo_host_network_baseline();
    let port = start_local_http_server()?;
    let network_caps = build_network_caps(&profile)?;
    demo_loopback_without_exemption(&network_caps, port)?;

    #[cfg(all(windows, feature = "net"))]
    let firewall_guard = demo_loopback_with_exemption(&profile, &network_caps, port)?;
    #[cfg(not(all(windows, feature = "net")))]
    demo_loopback_with_exemption(&profile, &network_caps, port)?;

    demo_outbound_internet(&network_caps)?;
    print_cleanup_header();

    #[cfg(all(windows, feature = "net"))]
    cleanup_firewall_guard(firewall_guard);

    cleanup_profile_and_print_summary(profile)
}

fn print_intro() {
    println!("rappct - Windows AppContainer Demo");
    println!("===================================\n");

    println!("This demo shows how to sandbox processes using Windows AppContainer technology.");
    println!("AppContainers enforce security at the OS level - similar to Linux containers.\n");
}

fn create_demo_profile() -> rappct::Result<AppContainerProfile> {
    println!("STEP 1: Creating AppContainer Profile");
    println!("--------------------------------------");
    println!("A profile defines a security boundary. Windows assigns each one a unique SID");
    println!("(Security Identifier) that controls what sandboxed processes can access.\n");

    let profile =
        AppContainerProfile::ensure("demo.app", "Demo Application", Some("rappct demonstration"))?;
    println!("✓ Created profile: {}", profile.name);
    println!("  SID: {}", profile.sid.as_string());
    println!("  (This SID identifies our sandbox and governs all access checks)\n");
    Ok(profile)
}

fn demo_zero_capability_launch(profile: &AppContainerProfile) -> rappct::Result<()> {
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

    std::thread::sleep(Duration::from_secs(1));
    Ok(())
}

fn demo_http_without_network(_profile: &AppContainerProfile) -> rappct::Result<()> {
    println!("STEP 3: Outbound HTTP Without Network Capability");
    println!("-------------------------------------------------");
    println!("Testing outbound HTTP from AppContainer with NO network capabilities.");
    println!("Expected: HTTP should be blocked, demonstrating network isolation.\n");

    #[cfg(windows)]
    {
        let no_net_caps = SecurityCapabilitiesBuilder::new(&_profile.sid).build()?;
        let no_net_curl = LaunchOptions {
            exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
            cmdline: Some(" -s -I -f -m 5 http://example.com".to_string()),
            cwd: Some(PathBuf::from("C:\\Windows\\System32")),
            stdio: rappct::launch::StdioConfig::Pipe,
            ..Default::default()
        };
        println!("→ Trying HTTP request without InternetClient capability...");
        let (code, out) = launch_capture(&no_net_caps, &no_net_curl, Duration::from_secs(6))?;
        if code == 0 {
            println!("✗ Unexpected! HTTP succeeded without network capability. Output:\n{out}");
        } else {
            println!("✓ Blocked as expected (exit {code}). Network isolation is working.");
        }
    }
    #[cfg(not(windows))]
    {
        println!("⚠ Skipped on non-Windows platform");
    }

    std::thread::sleep(Duration::from_secs(1));
    Ok(())
}

fn demo_host_network_baseline() {
    println!("\nSTEP 4A: Baseline - Network Access Without Sandbox");
    println!("---------------------------------------------------");
    println!("First, let's test HTTP from the host (unsandboxed) as a baseline.\n");

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
        Err(e) => println!("✗ Network test error: {e}"),
    }
}

fn start_local_http_server() -> rappct::Result<u16> {
    println!("\nSTEP 4B: Sandboxed Localhost Access");
    println!("------------------------------------");
    println!("Starting a local HTTP server, then trying to reach it from inside the sandbox.");
    println!("By default, AppContainers BLOCK loopback (127.0.0.1) even with InternetClient.");
    println!("The 'net' feature allows adding a firewall exemption to permit loopback.\n");

    // Start a minimal localhost HTTP server on an ephemeral port.
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| rappct::AcError::Win32(format!("failed to bind localhost listener: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| {
            rappct::AcError::Win32(format!("failed to query localhost listener address: {e}"))
        })?
        .port();
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
    Ok(port)
}

fn build_network_caps(
    profile: &AppContainerProfile,
) -> rappct::Result<rappct::SecurityCapabilities> {
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
}

fn demo_loopback_without_exemption(
    network_caps: &rappct::SecurityCapabilities,
    port: u16,
) -> rappct::Result<()> {
    let curl_no_loopback = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
        cmdline: Some(format!(" -s -I -m 3 http://127.0.0.1:{port}")),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        stdio: rappct::launch::StdioConfig::Pipe,
        ..Default::default()
    };
    println!("→ Testing http://127.0.0.1:{port} WITHOUT loopback exemption...");
    #[cfg(windows)]
    {
        let (code, out) = launch_capture(network_caps, &curl_no_loopback, Duration::from_secs(4))?;
        if code == 0 {
            println!("✗ Unexpected! Localhost succeeded without exemption. Output:\n{out}");
        } else {
            println!(
                "✓ Blocked as expected (exit {code}). AppContainers deny loopback by default."
            );
        }
    }
    #[cfg(not(windows))]
    {
        let _ = launch_in_container(network_caps, &curl_no_loopback)?;
        std::thread::sleep(Duration::from_secs(2));
    }
    Ok(())
}

#[cfg(all(windows, feature = "net"))]
fn demo_loopback_with_exemption(
    profile: &AppContainerProfile,
    network_caps: &rappct::SecurityCapabilities,
    port: u16,
) -> rappct::Result<Option<FirewallGuard>> {
    println!("\n→ Adding firewall exemption to allow loopback for this container...");
    if let Err(e) =
        add_loopback_exemption(LoopbackAdd::new(profile.sid.clone()).confirm_debug_only())
    {
        println!("✗ Exemption failed: {e} (continuing anyway)");
        return Ok(None);
    }

    let guard = FirewallGuard::new(profile.sid.clone(), "✓ Firewall loopback exemption removed");
    std::thread::sleep(Duration::from_millis(1000));
    try_loopback_with_exemption(network_caps, port)?;
    Ok(Some(guard))
}

#[cfg(all(windows, feature = "net"))]
fn try_loopback_with_exemption(
    network_caps: &rappct::SecurityCapabilities,
    port: u16,
) -> rappct::Result<()> {
    let curl_with_loopback = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\curl.exe"),
        cmdline: Some(format!(" -s -I -m 5 http://127.0.0.1:{port}")),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        stdio: rappct::launch::StdioConfig::Pipe,
        ..Default::default()
    };
    println!("→ Testing http://127.0.0.1:{port} WITH loopback exemption...");
    let (code, out) = launch_capture(network_caps, &curl_with_loopback, Duration::from_secs(5))?;
    if code == 0 {
        println!("✓ Success! (exit 0). Headers:\n{out}");
    } else {
        println!("✗ Still failed (exit {code}). Output:\n{out}");
    }
    Ok(())
}

#[cfg(not(all(windows, feature = "net")))]
fn demo_loopback_with_exemption(
    _profile: &AppContainerProfile,
    _network_caps: &rappct::SecurityCapabilities,
    _port: u16,
) -> rappct::Result<()> {
    println!("\n  (Run with --features net to test loopback exemption)");
    Ok(())
}

fn demo_outbound_internet(network_caps: &rappct::SecurityCapabilities) -> rappct::Result<()> {
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
        let (code, out) = launch_capture(network_caps, &internet_curl, Duration::from_secs(6))?;
        if code == 0 {
            println!("✓ Outbound HTTP succeeded from sandbox (exit 0). Headers:\n{out}");
        } else {
            println!("✗ Outbound request failed (exit {code}). Output:\n{out}");
        }
    }
    #[cfg(not(windows))]
    {
        let _ = launch_in_container(network_caps, &internet_curl)?;
        std::thread::sleep(Duration::from_secs(2));
    }
    Ok(())
}

#[cfg(windows)]
fn launch_capture(
    caps: &rappct::SecurityCapabilities,
    opts: &LaunchOptions,
    timeout: Duration,
) -> rappct::Result<(u32, String)> {
    let mut child = launch_in_container_with_io(caps, opts)?;
    let capture = child.capture_output();
    let code = child.wait(Some(timeout))?;
    let output = capture.finish()?;
    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok((code, out))
}

#[cfg(all(windows, feature = "net"))]
fn cleanup_firewall_guard(guard: Option<FirewallGuard>) {
    let _firewall_guard = guard;
}

fn print_cleanup_header() {
    println!("\nSTEP 5: Cleanup");
    println!("---------------");
    println!("Removing firewall exemptions and deleting the AppContainer profile.");
}

fn cleanup_profile_and_print_summary(profile: AppContainerProfile) -> rappct::Result<()> {
    let profile_name = profile.name.clone();
    profile.delete()?;
    println!("✓ Profile '{profile_name}' deleted successfully");

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
