use super::{repo_local_tempdir, resolve_cmd_exe};
use rappct::{
    AppContainerProfile, KnownCapability, SecurityCapabilities, SecurityCapabilitiesBuilder,
    acl::{AccessMask, ResourcePath, grant_to_package},
    launch::{JobLimits, LaunchOptions},
    launch_in_container,
};
use std::{ffi::OsString, path::PathBuf, process::Command, thread, time::Duration};

#[cfg(windows)]
use rappct::launch::{StdioConfig, launch_in_container_with_io};
#[cfg(windows)]
use std::io::{BufRead, BufReader};

/// Demo 5: Advanced Launch Options
pub(crate) fn demo_advanced_launch() -> rappct::Result<()> {
    println!("=== DEMO 5: Advanced Launch Options ===");
    println!("Demonstrating suspended launch, custom environment, and timeouts");
    run_normal_environment_probe();
    println!("\n→ Now comparing with AppContainer restrictions:");

    let profile = AppContainerProfile::ensure("rappct.advanced.launch", "Advanced Launch", None)?;
    let scratch = repo_local_tempdir("advanced-launch")?;
    let task_temp = scratch.path().as_os_str().to_os_string();
    grant_to_package(
        ResourcePath::Directory(scratch.path().to_path_buf()),
        &profile.sid,
        AccessMask::GENERIC_ALL,
    )?;

    let custom_env = custom_launch_environment(task_temp);
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()?;
    let opts = advanced_launch_options(custom_env);

    run_advanced_launch(&caps, &opts)?;
    cleanup_advanced_launch(profile, scratch)
}

fn run_normal_environment_probe() {
    println!("\n→ Baseline: Normal process with custom environment");
    let output = Command::new("cmd")
        .arg("/C")
        .arg("echo Normal process: RAPPCT_DEMO=%RAPPCT_DEMO% && echo Normal process: PATH accessible")
        .env("RAPPCT_DEMO", "normal")
        .output();
    match output {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            println!("✓ Normal process: Custom environment and PATH work normally");
            if result.contains("normal") {
                println!("  • Environment variable: SUCCESS");
            }
        }
        Err(e) => println!("⚠ Normal process test error: {e}"),
    }
}

fn custom_launch_environment(task_temp: OsString) -> Vec<(OsString, OsString)> {
    vec![
        (OsString::from("RAPPCT_DEMO"), OsString::from("advanced")),
        (
            OsString::from("ISOLATION_LEVEL"),
            OsString::from("appcontainer"),
        ),
        (
            OsString::from("PATH"),
            OsString::from("C:\\Windows\\System32"),
        ),
        (OsString::from("TEMP"), task_temp.clone()),
        (OsString::from("TMP"), task_temp.clone()),
        (OsString::from("TMPDIR"), task_temp),
    ]
}

fn advanced_launch_options(custom_env: Vec<(OsString, OsString)>) -> LaunchOptions {
    println!("→ Launching with custom environment and timeout...");
    println!(
        "  Environment has {} variables (system essentials + custom)",
        custom_env.len()
    );
    LaunchOptions {
        exe: resolve_cmd_exe(),
        cmdline: Some("/C echo RAPPCT_DEMO=%RAPPCT_DEMO% && echo ISOLATION_LEVEL=%ISOLATION_LEVEL% && echo SystemRoot=%SystemRoot% && echo Advanced launch completed".to_string()),
        cwd: Some(PathBuf::from("C:\\Windows\\System32")),
        env: Some(custom_env),
        suspended: false,
        startup_timeout: Some(Duration::from_secs(10)),
        join_job: Some(JobLimits {
            memory_bytes: Some(64 * 1024 * 1024),
            cpu_rate_percent: Some(25),
            kill_on_job_close: true,
        }),
        ..Default::default()
    }
}

fn run_advanced_launch(caps: &SecurityCapabilities, opts: &LaunchOptions) -> rappct::Result<()> {
    match launch_in_container(caps, opts) {
        Ok(child) => {
            println!("✓ Advanced launch successful, PID: {}", child.pid);
            println!("  Process has custom environment and resource limits");
            thread::sleep(Duration::from_secs(3));
        }
        Err(e) => {
            println!("⚠ Advanced launch failed: {e}");
            println!("  This is normal in restricted AppContainer environments");
            println!("  The advanced APIs still work for profile/SID management");
        }
    }
    Ok(())
}

fn cleanup_advanced_launch(
    profile: AppContainerProfile,
    scratch: tempfile::TempDir,
) -> rappct::Result<()> {
    profile.delete()?;
    scratch.close().map_err(|error| {
        rappct::AcError::Win32(format!(
            "Failed to clean repository-local advanced launch scratch: {error}"
        ))
    })?;
    println!("✓ Profile cleaned up\n");
    Ok(())
}

/// Demo 6: Enhanced I/O with Error Handling
#[cfg(windows)]
pub(crate) fn demo_enhanced_io() -> rappct::Result<()> {
    println!("=== DEMO 6: Enhanced I/O with Error Handling ===");
    println!("Using launch_in_container_with_io for full process interaction");

    let profile = AppContainerProfile::ensure("rappct.io.demo", "I/O Demo", None)?;
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;
    let opts = enhanced_io_options();

    let mut child_io = launch_in_container_with_io(&caps, &opts)?;
    println!("✓ Process launched with PID: {}", child_io.pid);
    read_process_pipe("stdout", child_io.stdout.take());
    read_process_pipe("stderr", child_io.stderr.take());
    child_io.wait(Some(Duration::from_secs(5)))?;

    profile.delete()?;
    println!("✓ Profile cleaned up\n");
    Ok(())
}

#[cfg(windows)]
fn enhanced_io_options() -> LaunchOptions {
    println!("→ Launching process with full I/O redirection...");
    LaunchOptions {
        exe: resolve_cmd_exe(),
        cmdline: Some("/C echo [STDOUT] Hello from AppContainer && echo [STDERR] This is an error message 1>&2 && echo [STDOUT] Process completed".to_string()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    }
}

#[cfg(windows)]
fn read_process_pipe(label: &str, pipe: Option<std::fs::File>) {
    if let Some(pipe) = pipe {
        println!("\n→ Reading {label}:");
        let reader = BufReader::new(pipe);
        for line in reader.lines() {
            match line {
                Ok(content) => println!("  {}: {content}", label.to_uppercase()),
                Err(e) => println!("  {} read error: {e}", label.to_uppercase()),
            }
        }
    }
}

#[cfg(not(windows))]
pub(crate) fn demo_enhanced_io() -> rappct::Result<()> {
    println!("=== DEMO 6: Enhanced I/O with Error Handling ===");
    println!("Using launch_in_container_with_io for full process interaction");
    println!("⚠ Enhanced I/O demo requires Windows");
    Err(rappct::AcError::UnsupportedPlatform)
}
