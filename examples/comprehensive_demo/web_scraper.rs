use super::{cleanup_repo_local_tempdir, repo_local_tempdir};
use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilities, SecurityCapabilitiesBuilder,
    acl::{AccessMask, ResourcePath, grant_to_package},
    launch::{JobLimits, LaunchOptions, StdioConfig},
    launch_in_container, supports_lpac,
};
use std::{fs, path::PathBuf, thread, time::Duration};

const WEB_SCRAPER_SCRIPT: &str = r#"param($WorkDir)

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

struct ScraperWorkspace {
    profile: AppContainerProfile,
    scratch: tempfile::TempDir,
    work_dir: PathBuf,
}

/// Demo 9: Comprehensive Example
/// Combines multiple features in a realistic scenario
pub(crate) fn demo_comprehensive() -> rappct::Result<()> {
    print_comprehensive_header();
    let workspace = prepare_scraper_workspace()?;
    let caps = build_scraper_capabilities(&workspace.profile)?;
    let script_file = write_scraper_script(&workspace.work_dir)?;
    let opts = scraper_launch_options(&workspace.work_dir, &script_file);

    let child = launch_in_container(&caps, &opts)?;
    println!("✓ Sandboxed process launched with PID: {}", child.pid);
    println!("\n→ Waiting for completion...");
    thread::sleep(Duration::from_secs(8));
    report_scraper_output(&workspace.work_dir)?;
    cleanup_scraper_workspace(workspace)
}

fn print_comprehensive_header() {
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
}

fn prepare_scraper_workspace() -> rappct::Result<ScraperWorkspace> {
    let profile = AppContainerProfile::ensure(
        "rappct.demo.webscraper",
        "Web Scraper Sandbox",
        Some("Secure web scraper with limited permissions"),
    )?;
    let scratch = repo_local_tempdir("web-scraper")?;
    let work_dir = scratch.path().to_path_buf();

    println!("\n→ Setting up sandbox environment...");
    println!("  • Work directory: {}", work_dir.display());
    grant_to_package(
        ResourcePath::Directory(work_dir.clone()),
        &profile.sid,
        AccessMask::GENERIC_ALL,
    )?;
    println!("  ✓ File system ACLs configured");
    Ok(ScraperWorkspace {
        profile,
        scratch,
        work_dir,
    })
}

fn build_scraper_capabilities(
    profile: &AppContainerProfile,
) -> rappct::Result<SecurityCapabilities> {
    let mut caps_builder = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient]);
    if supports_lpac().is_ok() {
        caps_builder = caps_builder.with_lpac_defaults();
        println!("  ✓ LPAC mode enabled");
    }

    let caps = caps_builder.build()?;
    println!("  ✓ Capabilities configured");
    Ok(caps)
}

fn write_scraper_script(work_dir: &std::path::Path) -> rappct::Result<PathBuf> {
    let script_file = work_dir.join("scraper.ps1");
    fs::write(&script_file, WEB_SCRAPER_SCRIPT).map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to write PowerShell script {}: {}",
            script_file.display(),
            e
        ))
    })?;
    Ok(script_file)
}

fn scraper_launch_options(
    work_dir: &std::path::Path,
    script_file: &std::path::Path,
) -> LaunchOptions {
    println!("\n→ Launching sandboxed PowerShell scraper...");
    println!("  Resource limits:");
    println!("    • Memory: 100 MB max");
    println!("    • CPU: 50% max");
    let work_dir_arg = format!("{}", work_dir.display()).replace('\'', "''");

    LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
        cmdline: Some(format!(
            "-NoProfile -ExecutionPolicy Bypass -File \"{}\" -WorkDir '{}'",
            script_file.display(),
            work_dir_arg
        )),
        cwd: Some(work_dir.to_path_buf()),
        stdio: StdioConfig::Inherit,
        join_job: Some(JobLimits {
            memory_bytes: Some(100 * 1024 * 1024),
            cpu_rate_percent: Some(50),
            kill_on_job_close: true,
        }),
        ..Default::default()
    }
}

fn report_scraper_output(work_dir: &std::path::Path) -> rappct::Result<()> {
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
    Ok(())
}

fn cleanup_scraper_workspace(workspace: ScraperWorkspace) -> rappct::Result<()> {
    println!("\n→ Cleaning up...");
    workspace.profile.delete()?;
    cleanup_repo_local_tempdir(workspace.scratch)?;
    println!("✓ Sandbox environment cleaned");
    Ok(())
}
