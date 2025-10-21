//! Network capability demonstration with automatic firewall configuration
//!
//! This example demonstrates rappct's built-in firewall loopback exemption
//! functionality for proper AppContainer network access.

use rappct::{supports_lpac, AppContainerProfile, KnownCapability};

#[cfg(windows)]
use rappct::SecurityCapabilitiesBuilder;

#[cfg(windows)]
use rappct::launch::{launch_in_container_with_io, LaunchOptions, StdioConfig};

#[cfg(feature = "net")]
use rappct::net::{add_loopback_exemption, remove_loopback_exemption, LoopbackAdd};

#[cfg(feature = "net")]
struct FirewallGuard {
    sid: rappct::sid::AppContainerSid,
    intro: Option<&'static str>,
    success: &'static str,
}

#[cfg(feature = "net")]
impl FirewallGuard {
    fn new(
        sid: rappct::sid::AppContainerSid,
        intro: Option<&'static str>,
        success: &'static str,
    ) -> Self {
        Self {
            sid,
            intro,
            success,
        }
    }
}

#[cfg(feature = "net")]
impl Drop for FirewallGuard {
    fn drop(&mut self) {
        if let Some(message) = self.intro {
            println!("{}", message);
        }
        match remove_loopback_exemption(&self.sid) {
            Ok(_) => println!("{}", self.success),
            Err(e) => println!("⚠ Firewall exemption cleanup failed: {}", e),
        }
    }
}

#[cfg(windows)]
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    time::Duration,
};

fn main() -> rappct::Result<()> {
    println!("rappct Network Capability Demo");
    println!("==============================\n");

    #[cfg(not(feature = "net"))]
    {
        println!("❌ ERROR: This example requires the 'net' feature to be enabled!");
        println!("Run with: cargo run --example network_demo --features net");
        println!("\nThe 'net' feature provides automatic firewall loopback exemption");
        println!("which is essential for AppContainer network functionality.");
        Ok(())
    }

    #[cfg(feature = "net")]
    {
        println!(
            "This demo shows rappct's automatic firewall configuration for AppContainer network access."
        );
        println!("rappct will handle Windows Firewall loopback exemptions automatically.\n");

        println!("⚠️ PREREQUISITES:");
        println!("• Run as Administrator (required for firewall modifications)");
        println!("• 'net' feature enabled (✓ enabled)");
        println!("• Internet connectivity for testing\n");

        let profile = AppContainerProfile::ensure(
            "network.test",
            "Network Test",
            Some("Network capability testing"),
        )?;

        println!("✓ Created test profile: {}", profile.sid.as_string());

        // Add firewall loopback exemption for localhost access
        println!("→ Adding firewall loopback exemption for network access...");
        let firewall_guard =
            match add_loopback_exemption(LoopbackAdd(profile.sid.clone()).confirm_debug_only()) {
                Ok(_) => {
                    println!("✓ Firewall loopback exemption added");
                    Some(FirewallGuard::new(
                        profile.sid.clone(),
                        Some("\n→ Removing firewall loopback exemption..."),
                        "✓ Firewall exemption removed",
                    ))
                }
                Err(e) => {
                    println!("⚠ Firewall exemption failed: {}", e);
                    println!("  Network tests may have limited functionality");
                    None
                }
            };

        run_network_tests(&profile)?;

        // FirewallGuard will auto-cleanup on drop
        let _firewall_guard = firewall_guard;

        // Cleanup profile
        let profile_name = profile.name.clone();
        profile.delete()?;
        println!("✓ Cleaned up profile: {}", profile_name);

        println!("\n🎉 Network Demo Complete!");
        println!("========================");
        println!("Key takeaways:");
        println!("• rappct automatically handles Windows Firewall configuration");
        println!("• Use LoopbackAdd::confirm_debug_only() for development/testing");
        println!("• Always clean up firewall exemptions when done");
        println!("• Network capabilities work much better with proper firewall config");
        Ok(())
    }
}

#[cfg(feature = "net")]
fn run_network_tests(profile: &AppContainerProfile) -> rappct::Result<()> {
    println!(
        "Expected: Baseline (normal process) DNS and HTTP should succeed; AppContainer results vary by capability set and Windows version.\n"
    );

    // First, demonstrate normal (non-AppContainer) network access
    println!("\n=== BASELINE: Normal Network Access (No AppContainer) ===");
    println!("→ Running network tests outside AppContainer to show normal behavior");

    use std::process::Command;
    println!("\n→ Testing DNS resolution (normal process):");
    match Command::new("nslookup").arg("google.com").output() {
        Ok(output) => {
            if output.status.success() {
                println!("✓ DNS: SUCCESS (normal process can resolve domains)");
            } else {
                println!("⚠ DNS: FAILED (may be network/policy issue)");
            }
        }
        Err(e) => println!("⚠ DNS test error: {}", e),
    }

    println!("\n→ Testing HTTP connectivity (normal process):");
    match Command::new("powershell")
        .arg("-Command")
        .arg("try { (Invoke-WebRequest -Uri 'http://httpbin.org/ip' -UseBasicParsing -TimeoutSec 3).StatusCode } catch { 'Failed' }")
        .output() {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            let result = result.trim();
            if result.contains("200") {
                println!("✓ HTTP: SUCCESS (normal process can access internet)");
            } else {
                println!("⚠ HTTP: {} (may be network/policy issue)", result);
            }
        }
        Err(e) => println!("⚠ HTTP test error: {}", e),
    }

    println!("\n{}", "=".repeat(60));
    println!("Now comparing with AppContainer isolation:");
    println!("{}", "=".repeat(60));

    // Test 1: No network capability (should fail)
    println!("\n=== TEST 1: AppContainer with No Network Capability ===");
    println!(
        "Expected: Network commands will fail or be severely restricted (demonstrating isolation)"
    );

    test_network_access(profile, &[], "NO-NET", false).run_with_timeout(6)?;

    // Test 2: Internet Client capability
    println!("\n=== TEST 2: AppContainer with Internet Client Capability ===");
    println!(
        "Expected: HTTP should work; DNS may succeed or fail depending on cache and Windows version"
    );

    test_network_access(
        profile,
        &[KnownCapability::InternetClient],
        "INET-CLIENT",
        false,
    )
    .run_with_timeout(6)?;

    // Test 3: All network capabilities
    println!("\n=== TEST 3: AppContainer with All Network Capabilities ===");
    println!(
        "Expected: Broadest AppContainer network access, though local policies may still limit specific calls"
    );

    test_network_access(
        profile,
        &[
            KnownCapability::InternetClient,
            KnownCapability::InternetClientServer,
            KnownCapability::PrivateNetworkClientServer,
        ],
        "ALL-NET",
        false,
    )
    .run_with_timeout(6)?;

    // Test 4: LPAC with network
    if supports_lpac().is_ok() {
        println!("\n=== TEST 4: LPAC + Network ===");
        println!("Expected: Limited network; HTTP checks may be restricted by LPAC policy.");

        test_lpac_network(profile).run_with_timeout(6)?;
    }

    println!("\n🔍 Key Insight: Compare the baseline results with AppContainer results");
    println!("   • Normal processes: Full network access");
    println!("   • AppContainer: Restricted access demonstrating security isolation");

    Ok(())
}

#[cfg(feature = "net")]
fn test_network_access(
    profile: &AppContainerProfile,
    capabilities: &[KnownCapability],
    prefix: &str,
    enable_lpac: bool,
) -> NetworkTest {
    NetworkTest {
        profile_sid: profile.sid.clone(),
        capabilities: capabilities.to_vec(),
        prefix: prefix.to_string(),
        enable_lpac,
    }
}

#[cfg(feature = "net")]
fn test_lpac_network(profile: &AppContainerProfile) -> NetworkTest {
    test_network_access(profile, &[KnownCapability::InternetClient], "LPAC", true)
}

#[cfg_attr(not(windows), allow(dead_code))]
#[cfg(feature = "net")]
struct NetworkTest {
    profile_sid: rappct::sid::AppContainerSid,
    capabilities: Vec<KnownCapability>,
    prefix: String,
    enable_lpac: bool,
}

#[cfg(all(feature = "net", windows))]
impl NetworkTest {
    fn run_with_timeout(self, timeout_secs: u64) -> rappct::Result<()> {
        let mut caps_builder = SecurityCapabilitiesBuilder::new(&self.profile_sid);

        if !self.capabilities.is_empty() {
            caps_builder = caps_builder.with_known(&self.capabilities);
        }

        if self.enable_lpac {
            caps_builder = caps_builder.with_lpac_defaults();
        }

        let caps = caps_builder.build()?;

        // Create a comprehensive network test script with better HTTP testing
        let mut test_script = format!(
            r#"/C echo [{prefix}] Starting network tests... && echo [{prefix}] Test 1: DNS resolution && nslookup google.com 1>nul 2>nul && echo [{prefix}] DNS: SUCCESS || echo [{prefix}] DNS: FAILED && echo [{prefix}] Test 2: HTTP connectivity && powershell -Command "try {{ $response = Invoke-WebRequest -Uri 'http://httpbin.org/ip' -UseBasicParsing -TimeoutSec 5; 'HTTP: SUCCESS (Status: ' + $response.StatusCode + ')' }} catch {{ 'HTTP: FAILED - ' + $_.Exception.Message }}" && echo [{prefix}] Test 3: Localhost test && powershell -Command "try {{ $response = Invoke-WebRequest -Uri 'http://127.0.0.1:1' -UseBasicParsing -TimeoutSec 2 }} catch {{ if ($_.Exception.Message -like '*ConnectFailure*') {{ 'LOCALHOST: ACCESSIBLE (connection refused = good)' }} else {{ 'LOCALHOST: BLOCKED - ' + $_.Exception.Message }} }}" && echo [{prefix}] Network tests completed"#,
            prefix = self.prefix
        );

        // In LPAC, PowerShell may fail due to ETW/COM init restrictions; use curl for HTTP
        if self.enable_lpac {
            // Skip HTTP/localhost tests under LPAC to avoid environment-specific failures.
            test_script = format!(
                r#"/C echo [{prefix}] Starting network tests... && echo [{prefix}] Test 1: DNS resolution && nslookup google.com 1>nul 2>nul && echo [{prefix}] DNS: SUCCESS || echo [{prefix}] DNS: FAILED && echo [{prefix}] Test 2: HTTP connectivity (skipped under LPAC) && echo [{prefix}] Test 3: Localhost test (skipped under LPAC) && echo [{prefix}] Network tests completed"#,
                prefix = self.prefix
            );
        }

        let opts = LaunchOptions {
            exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            cmdline: Some(test_script),
            stdio: StdioConfig::Pipe,
            ..Default::default()
        };

        println!(
            "→ Launching test with capabilities: {:?}",
            self.capabilities
        );

        let mut child = launch_in_container_with_io(&caps, &opts)?;
        println!("✓ Process PID: {}", child.pid);

        // Read output in real-time
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                println!("  {}", line);
            }
        }

        child.wait(Some(Duration::from_secs(timeout_secs)))?;
        Ok(())
    }
}

#[cfg(all(feature = "net", not(windows)))]
impl NetworkTest {
    fn run_with_timeout(self, _timeout_secs: u64) -> rappct::Result<()> {
        Err(rappct::AcError::UnsupportedPlatform)
    }
}
