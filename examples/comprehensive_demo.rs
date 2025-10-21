//! Comprehensive rappct demonstration with individual capability examples
//!
//! This example provides clear, isolated demonstrations of each rappct capability
//! followed by a combined example showing how to use multiple features together.
//! Designed for easy developer adoption and understanding.

use rappct::{
    acl::{grant_to_package, AccessMask, ResourcePath},
    launch::{JobLimits, LaunchOptions, StdioConfig},
    launch_in_container, supports_lpac,
    token::query_current_process_token,
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
};
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

#[cfg(windows)]
use rappct::launch::launch_in_container_with_io;

#[cfg(windows)]
use std::io::{BufRead, BufReader};

type DemoEntry = (&'static str, fn() -> rappct::Result<()>);

/// Helper function to pause and wait for user input
fn pause_for_demo(msg: &str) {
    println!("\n{}", msg);
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
    println!("\n→ Creating AppContainer profile: '{}'", profile_name);
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
            println!("    - {}", cap);
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

/// Demo 4: Network Capabilities
/// Shows how to grant network access
fn demo_network_capabilities() -> rappct::Result<()> {
    println!(
        "Expected: InternetClient => HTTP works, DNS may fail; Client/Server => can listen; PrivateNetwork => LAN allowed."
    );
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║      DEMO 4: Network Capabilities              ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile = AppContainerProfile::ensure(
        "rappct.demo.network",
        "Network Demo",
        Some("Network capability demonstration"),
    )?;

    // Example 1: Internet Client only
    println!("\n→ Example 1: Internet Client capability");
    println!("  Allows: Outbound internet connections");
    println!("  Denies: Server operations, LAN access");

    let client_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;

    let client_opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        // Try multiple endpoints to reduce false negatives
        cmdline: Some("/C echo [NET-CLIENT] Testing Internet Client && powershell -Command \"$urls=@('http://httpbin.org/ip','http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; foreach($u in $urls){ try { $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5; if($r.StatusCode){ $code=$r.StatusCode; break } } catch {} }; if($code){ $code } else { 'HTTP failed' }\" && ping -n 2 8.8.8.8 && timeout /T 2 /NOBREAK >nul".to_string()),
        ..Default::default()
    };

    let child1 = launch_in_container(&client_caps, &client_opts)?;
    println!("✓ Launched with PID: {}", child1.pid);
    thread::sleep(Duration::from_secs(5));

    // Example 2: Client/Server capabilities
    println!("\n→ Example 2: Internet Client/Server capability");
    println!("  Allows: Internet connections + listening on ports");

    let server_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClientServer])
        .build()?;

    let server_opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        // Multi-endpoint HTTP with optional proxy support from env (HTTPS_PROXY/HTTP_PROXY)
        cmdline: Some("/C echo [NET-SERVER] Can act as both client and server && powershell -Command \"$urls=@('http://httpbin.org/ip','http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; $proxy=$env:HTTPS_PROXY; if(-not $proxy){ $proxy=$env:HTTP_PROXY }; foreach($u in $urls){ try { if($proxy){ $r=Invoke-WebRequest -Uri $u -Proxy $proxy -UseBasicParsing -TimeoutSec 5 } else { $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5 }; if($r.StatusCode){ $code=$r.StatusCode; break } } catch {} }; if($code){ $code } else { 'HTTP failed' }\" && netstat -an | findstr LISTENING && timeout /T 2 /NOBREAK >nul".to_string()),
        ..Default::default()
    };

    let child2 = launch_in_container(&server_caps, &server_opts)?;
    println!("✓ Launched with PID: {}", child2.pid);
    thread::sleep(Duration::from_secs(3));

    // Example 3: Private network access
    println!("\n→ Example 3: Private Network Client/Server");
    println!("  Allows: LAN/domain network access");

    let private_caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::PrivateNetworkClientServer])
        .build()?;

    let private_opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        // Use multi-endpoint HTTP to demonstrate network access reliably
        cmdline: Some("/C echo [NET-PRIVATE] Access to private networks && powershell -Command \"$urls=@('http://httpbin.org/ip','http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; foreach($u in $urls){ try { $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5; if($r.StatusCode){ $code=$r.StatusCode; break } } catch {} }; if($code){ $code } else { 'HTTP failed' }\" && timeout /T 2 /NOBREAK >nul".to_string()),
        ..Default::default()
    };

    let child3 = launch_in_container(&private_caps, &private_opts)?;
    println!("✓ Launched with PID: {}", child3.pid);
    thread::sleep(Duration::from_secs(3));

    profile.delete()?;
    Ok(())
}

/// Demo 5: File System ACLs
/// Shows how to grant file/directory access to containers
fn demo_file_acls() -> rappct::Result<()> {
    println!("Expected: Allowed file readable; denied file fails from AppContainer.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║        DEMO 5: File System ACLs                ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile = AppContainerProfile::ensure(
        "rappct.demo.acl",
        "ACL Demo",
        Some("File system ACL demonstration"),
    )?;

    // Create test directory structure
    let demo_dir = env::temp_dir().join("rappct_acl_demo");
    let allowed_dir = demo_dir.join("allowed");
    let denied_dir = demo_dir.join("denied");

    fs::create_dir_all(&allowed_dir).map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to create dir {}: {}",
            allowed_dir.display(),
            e
        ))
    })?;
    fs::create_dir_all(&denied_dir).map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to create dir {}: {}",
            denied_dir.display(),
            e
        ))
    })?;

    let allowed_file = allowed_dir.join("readable.txt");
    let denied_file = denied_dir.join("secret.txt");

    fs::write(&allowed_file, "This file is accessible from AppContainer!").map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to write test file {}: {}",
            allowed_file.display(),
            e
        ))
    })?;
    fs::write(
        &denied_file,
        "This file is NOT accessible from AppContainer!",
    )
    .map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to write test file {}: {}",
            denied_file.display(),
            e
        ))
    })?;

    println!("\n→ Created test structure:");
    println!("  • {}", demo_dir.display());
    println!("    ├── allowed/");
    println!("    │   └── readable.txt (will grant access)");
    println!("    └── denied/");
    println!("        └── secret.txt (no access)");

    println!("\n→ First, testing normal (non-AppContainer) file access:");
    use std::process::Command;
    let test_cmd = format!(
        "type \"{}\" && type \"{}\"",
        allowed_file.display(),
        denied_file.display()
    );
    match Command::new("cmd").arg("/C").arg(&test_cmd).output() {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            if result.contains("This file is accessible") {
                println!("✓ Normal process: Can read files (no restrictions)");
            } else {
                println!("⚠ Normal process: Unexpected file access behavior");
            }
        }
        Err(e) => println!("⚠ Normal process file test error: {}", e),
    }
    println!("\n→ Now comparing with AppContainer restrictions:");

    // Grant access to specific file and directory
    println!("\n→ Granting AppContainer access to allowed directory...");
    println!("  This modifies Windows ACLs to allow the AppContainer SID to access specific files");

    // Grant full access to the allowed directory and its contents
    grant_to_package(
        ResourcePath::Directory(allowed_dir.clone()),
        &profile.sid,
        AccessMask(0x001F01FF), // GENERIC_ALL - full access
    )?;

    grant_to_package(
        ResourcePath::File(allowed_file.clone()),
        &profile.sid,
        AccessMask(0x001F01FF), // GENERIC_ALL - full access
    )?;
    println!("✓ ACLs applied - AppContainer can now access the allowed directory");

    // Launch process to test access
    println!("\n→ Testing file access from AppContainer...");
    println!("  Expected: Can read allowed file, cannot read denied file");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;

    let test_script = format!(
        r#"/C echo [ACL-TEST] Testing file access... && echo [ACL-TEST] Reading allowed file: && type "{}" && echo. && echo [ACL-TEST] Trying denied file (should fail): && type "{}" 2>nul || echo [ACL-TEST] Access denied as expected && timeout /T 3 /NOBREAK >nul"#,
        allowed_file.display(),
        denied_file.display()
    );

    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some(test_script),
        ..Default::default()
    };

    let child = launch_in_container(&caps, &opts)?;
    println!("✓ Test process PID: {}", child.pid);

    thread::sleep(Duration::from_secs(4));

    // Cleanup
    fs::remove_dir_all(&demo_dir).ok();
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
        println!("  💡 You can test LPAC features by setting RAPPCT_TEST_LPAC_STATUS=ok");
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
            println!("    > {}", line);
        }
    }

    if let Some(stderr) = child_io.stderr.take() {
        let reader = BufReader::new(stderr);
        println!("  STDERR:");
        for line in reader.lines().map_while(Result::ok) {
            println!("    > {}", line);
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

/// Demo 9: Comprehensive Example
/// Combines multiple features in a realistic scenario
fn demo_comprehensive() -> rappct::Result<()> {
    println!(
        "Expected: Sandboxed PowerShell can fetch HTTP, write file within granted directory, respect job limits."
    );
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║     DEMO 9: Comprehensive Example              ║");
    println!("║     (Secure Web Scraper Sandbox)               ║");
    println!("╚════════════════════════════════════════════════╝");

    println!("\nScenario: Sandboxed PowerShell script that:");
    println!("  • Downloads content from the internet");
    println!("  • Saves to a specific allowed directory");
    println!("  • Has memory and CPU limits");
    println!("  • Runs in LPAC mode for enhanced but limited access");

    // Setup profile
    let profile = AppContainerProfile::ensure(
        "rappct.demo.webscraper",
        "Web Scraper Sandbox",
        Some("Secure web scraper with limited permissions"),
    )?;

    // Setup allowed directory
    let work_dir = env::temp_dir().join("rappct_scraper_sandbox");
    fs::create_dir_all(&work_dir).map_err(|e| rappct::AcError::Win32(e.to_string()))?;

    println!("\n→ Setting up sandbox environment...");
    println!("  • Work directory: {}", work_dir.display());

    // Grant ACL permissions
    grant_to_package(
        ResourcePath::Directory(work_dir.clone()),
        &profile.sid,
        AccessMask(0x001F01FF), // GENERIC_ALL for the work directory
    )?;
    println!("  ✓ File system ACLs configured");

    // Build capabilities
    let mut caps_builder = SecurityCapabilitiesBuilder::new(&profile.sid).with_known(&[
        KnownCapability::InternetClient, // Can download from internet
    ]);

    // Add LPAC if supported
    if supports_lpac().is_ok() {
        caps_builder = caps_builder.with_lpac_defaults();
        println!("  ✓ LPAC mode enabled");
    }

    let caps = caps_builder.build()?;
    println!("  ✓ Capabilities configured");

    // Create PowerShell script
    let script = r#"param($WorkDir)

Write-Host 'Sandboxed Web Scraper Started' -ForegroundColor Green
Write-Host "Working directory: $WorkDir"
Write-Host ''

try {
    Write-Host 'Downloading example content...'
    $url = 'http://example.com'
    $response = Invoke-WebRequest -Uri $url -UseBasicParsing

    $outputFile = Join-Path $WorkDir 'downloaded_content.html'
    $response.Content | Out-File -FilePath $outputFile

    Write-Host "Content saved to: $outputFile" -ForegroundColor Green
    Write-Host "File size: $((Get-Item $outputFile).Length) bytes"
} catch {
    Write-Host "Download failed: $_" -ForegroundColor Red
}

Write-Host ''
Write-Host 'Sandbox restrictions in effect:' -ForegroundColor Yellow
Write-Host '  - Network: Internet client only'
Write-Host '  - File access: Limited to work directory'
Write-Host '  - Memory: Max 100MB'
Write-Host '  - CPU: Max 50%'
"#;

    let script_file = work_dir.join("scraper.ps1");
    fs::write(&script_file, script).map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to write PowerShell script {}: {}",
            script_file.display(),
            e
        ))
    })?;

    // Launch sandboxed PowerShell
    println!("\n→ Launching sandboxed PowerShell scraper...");
    println!("  Resource limits:");
    println!("    • Memory: 100 MB max");
    println!("    • CPU: 50% max");

    let work_dir_arg = format!("{}", work_dir.display()).replace('\'', "''");

    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
        cmdline: Some(format!(
            "-NoProfile -ExecutionPolicy Bypass -File \"{}\" -WorkDir '{}'",
            script_file.display(),
            work_dir_arg
        )),
        cwd: Some(work_dir.clone()),
        stdio: StdioConfig::Inherit,
        join_job: Some(JobLimits {
            memory_bytes: Some(100 * 1024 * 1024), // 100 MB
            cpu_rate_percent: Some(50),            // 50% CPU
            kill_on_job_close: true,
        }),
        ..Default::default()
    };

    let child = launch_in_container(&caps, &opts)?;
    println!("✓ Sandboxed process launched with PID: {}", child.pid);

    println!("\n→ Waiting for completion...");
    thread::sleep(Duration::from_secs(8));

    // Check results
    let output_file = work_dir.join("downloaded_content.html");
    if output_file.exists() {
        let content =
            fs::read_to_string(&output_file).map_err(|e| rappct::AcError::Win32(e.to_string()))?;
        println!("\n✓ Successfully downloaded content");
        println!("  File size: {} bytes", content.len());
        println!(
            "  First 100 chars: {}...",
            &content[..content.len().min(100)]
        );
    }

    // Cleanup
    println!("\n→ Cleaning up...");
    fs::remove_dir_all(&work_dir).ok();
    profile.delete()?;
    println!("✓ Sandbox environment cleaned");

    Ok(())
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
        ("Network Capabilities", demo_network_capabilities),
        ("File System ACLs", demo_file_acls),
        ("LPAC Mode", demo_lpac),
        ("Job Objects & Resource Limits", demo_job_limits),
        ("Process I/O Redirection", demo_io_redirection),
        ("Comprehensive Example", demo_comprehensive),
    ];

    for (name, demo_fn) in demos {
        match demo_fn() {
            Ok(_) => println!("\n✓ {} completed successfully", name),
            Err(e) => {
                println!("\n✗ {} failed: {}", name, e);
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
