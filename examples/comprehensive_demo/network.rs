use super::{cleanup_repo_local_tempdir, repo_local_tempdir};
use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilitiesBuilder,
    acl::{AccessMask, ResourcePath, grant_to_package},
    launch::LaunchOptions,
    launch_in_container,
};
use std::{path::PathBuf, thread, time::Duration};

/// Demo 4: Network Capabilities
/// Shows how to grant network access
pub(crate) fn demo_network_capabilities() -> rappct::Result<()> {
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
    let scratch = repo_local_tempdir("network")?;
    let temp_dir = scratch.path().to_path_buf();
    grant_to_package(
        ResourcePath::Directory(temp_dir.clone()),
        &profile.sid,
        AccessMask::GENERIC_ALL,
    )?;

    run_internet_client_example(&profile, &temp_dir)?;
    run_client_server_example(&profile, &temp_dir)?;
    run_private_network_example(&profile, &temp_dir)?;

    profile.delete()?;
    cleanup_repo_local_tempdir(scratch)
}

fn run_internet_client_example(
    profile: &AppContainerProfile,
    temp_dir: &std::path::Path,
) -> rappct::Result<()> {
    println!("\n→ Example 1: Internet Client capability");
    println!("  Allows: Outbound internet connections");
    println!("  Denies: Server operations, LAN access");

    let output_file = temp_dir.join(format!("rappct_http_client_{}.txt", std::process::id()));
    let cmdline = format!(
        "/C echo [NET-CLIENT] Testing Internet Client && powershell -Command \"$urls=@('http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; foreach($u in $urls){{ try {{ $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5; if($r.StatusCode){{ $code=$r.StatusCode; break }} }} catch {{}} }}; if($code){{ $code | Out-File -FilePath '{}' -Encoding ASCII }} else {{ 'HTTP failed' | Out-File -FilePath '{}' -Encoding ASCII }}\" && type \"{}\" 2>nul || echo HTTP failed && del \"{}\" 2>nul && ping -n 2 8.8.8.8 && timeout /T 2 /NOBREAK >nul",
        output_file.display(),
        output_file.display(),
        output_file.display(),
        output_file.display()
    );
    launch_network_probe(profile, KnownCapability::InternetClient, cmdline, 5)
}

fn run_client_server_example(
    profile: &AppContainerProfile,
    temp_dir: &std::path::Path,
) -> rappct::Result<()> {
    println!("\n→ Example 2: Internet Client/Server capability");
    println!("  Allows: Internet connections + listening on ports");

    let output_file = temp_dir.join(format!("rappct_http_server_{}.txt", std::process::id()));
    let cmdline = format!(
        "/C echo [NET-SERVER] Can act as both client and server && powershell -Command \"$urls=@('http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; $proxy=$env:HTTPS_PROXY; if(-not $proxy){{ $proxy=$env:HTTP_PROXY }}; foreach($u in $urls){{ try {{ if($proxy){{ $r=Invoke-WebRequest -Uri $u -Proxy $proxy -UseBasicParsing -TimeoutSec 5 }} else {{ $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5 }}; if($r.StatusCode){{ $code=$r.StatusCode; break }} }} catch {{}} }}; if($code){{ $code | Out-File -FilePath '{}' -Encoding ASCII }} else {{ 'HTTP failed' | Out-File -FilePath '{}' -Encoding ASCII }}\" && type \"{}\" 2>nul || echo HTTP failed && del \"{}\" 2>nul && netstat -an | findstr LISTENING && timeout /T 2 /NOBREAK >nul",
        output_file.display(),
        output_file.display(),
        output_file.display(),
        output_file.display()
    );
    launch_network_probe(profile, KnownCapability::InternetClientServer, cmdline, 3)
}

fn run_private_network_example(
    profile: &AppContainerProfile,
    temp_dir: &std::path::Path,
) -> rappct::Result<()> {
    println!("\n→ Example 3: Private Network Client/Server");
    println!("  Allows: LAN/domain network access");

    let output_file = temp_dir.join(format!("rappct_http_private_{}.txt", std::process::id()));
    let cmdline = format!(
        "/C echo [NET-PRIVATE] Access to private networks && powershell -Command \"$urls=@('http://example.com','http://www.msftconnecttest.com/connecttest.txt'); $code=''; foreach($u in $urls){{ try {{ $r=Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5; if($r.StatusCode){{ $code=$r.StatusCode; break }} }} catch {{}} }}; if($code){{ $code | Out-File -FilePath '{}' -Encoding ASCII }} else {{ 'HTTP failed' | Out-File -FilePath '{}' -Encoding ASCII }}\" && type \"{}\" 2>nul || echo HTTP failed && del \"{}\" 2>nul && timeout /T 2 /NOBREAK >nul",
        output_file.display(),
        output_file.display(),
        output_file.display(),
        output_file.display()
    );
    launch_network_probe(
        profile,
        KnownCapability::PrivateNetworkClientServer,
        cmdline,
        3,
    )
}

fn launch_network_probe(
    profile: &AppContainerProfile,
    capability: KnownCapability,
    cmdline: String,
    sleep_seconds: u64,
) -> rappct::Result<()> {
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[capability])
        .build()?;
    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some(cmdline),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts)?;
    println!("✓ Launched with PID: {}", child.pid);
    thread::sleep(Duration::from_secs(sleep_seconds));
    Ok(())
}
