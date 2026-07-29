use super::{cleanup_repo_local_tempdir, repo_local_tempdir};
use rappct::{
    AppContainerProfile, SecurityCapabilitiesBuilder,
    acl::{AccessMask, ResourcePath, grant_to_package},
    launch::LaunchOptions,
    launch_in_container,
};
use std::{fs, path::PathBuf, process::Command, thread, time::Duration};

struct AclFixture {
    demo_dir: PathBuf,
    allowed_dir: PathBuf,
    allowed_file: PathBuf,
    denied_file: PathBuf,
}

/// Demo 5: File System ACLs
/// Shows how to grant file/directory access to containers
pub(crate) fn demo_file_acls() -> rappct::Result<()> {
    println!("Expected: Allowed file readable; denied file fails from AppContainer.");
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║        DEMO 5: File System ACLs                ║");
    println!("╚════════════════════════════════════════════════╝");

    let profile = AppContainerProfile::ensure(
        "rappct.demo.acl",
        "ACL Demo",
        Some("File system ACL demonstration"),
    )?;
    let scratch = repo_local_tempdir("file-acls")?;
    let fixture = create_acl_fixture(scratch.path())?;
    print_acl_fixture(&fixture);
    run_host_file_probe(&fixture);
    grant_allowed_access(&profile, &fixture)?;
    launch_acl_probe(&profile, &fixture)?;

    profile.delete()?;
    cleanup_repo_local_tempdir(scratch)
}

fn create_acl_fixture(root: &std::path::Path) -> rappct::Result<AclFixture> {
    let allowed_dir = root.join("allowed");
    let denied_dir = root.join("denied");
    create_dir(&allowed_dir)?;
    create_dir(&denied_dir)?;

    let allowed_file = allowed_dir.join("readable.txt");
    let denied_file = denied_dir.join("secret.txt");
    write_file(&allowed_file, "This file is accessible from AppContainer!")?;
    write_file(
        &denied_file,
        "This file is NOT accessible from AppContainer!",
    )?;

    Ok(AclFixture {
        demo_dir: root.to_path_buf(),
        allowed_dir,
        allowed_file,
        denied_file,
    })
}

fn create_dir(path: &std::path::Path) -> rappct::Result<()> {
    fs::create_dir_all(path).map_err(|e| {
        rappct::AcError::Win32(format!("Failed to create dir {}: {}", path.display(), e))
    })
}

fn write_file(path: &std::path::Path, contents: &str) -> rappct::Result<()> {
    fs::write(path, contents).map_err(|e| {
        rappct::AcError::Win32(format!(
            "Failed to write test file {}: {}",
            path.display(),
            e
        ))
    })
}

fn print_acl_fixture(fixture: &AclFixture) {
    println!("\n→ Created test structure:");
    println!("  • {}", fixture.demo_dir.display());
    println!("    ├── allowed/");
    println!("    │   └── readable.txt (will grant access)");
    println!("    └── denied/");
    println!("        └── secret.txt (no access)");
}

fn run_host_file_probe(fixture: &AclFixture) {
    println!("\n→ First, testing normal (non-AppContainer) file access:");
    let test_cmd = format!(
        "type \"{}\" && type \"{}\"",
        fixture.allowed_file.display(),
        fixture.denied_file.display()
    );
    match Command::new("cmd").arg("/C").arg(&test_cmd).output() {
        Ok(output) => report_host_file_probe(&output.stdout),
        Err(e) => println!("⚠ Normal process file test error: {e}"),
    }
    println!("\n→ Now comparing with AppContainer restrictions:");
}

fn report_host_file_probe(stdout: &[u8]) {
    let result = String::from_utf8_lossy(stdout);
    if result.contains("This file is accessible") {
        println!("✓ Normal process: Can read files (no restrictions)");
    } else {
        println!("⚠ Normal process: Unexpected file access behavior");
    }
}

fn grant_allowed_access(profile: &AppContainerProfile, fixture: &AclFixture) -> rappct::Result<()> {
    println!("\n→ Granting AppContainer access to allowed directory...");
    println!("  This modifies Windows ACLs to allow the AppContainer SID to access specific files");
    grant_to_package(
        ResourcePath::Directory(fixture.allowed_dir.clone()),
        &profile.sid,
        AccessMask::GENERIC_ALL,
    )?;
    grant_to_package(
        ResourcePath::File(fixture.allowed_file.clone()),
        &profile.sid,
        AccessMask::GENERIC_ALL,
    )?;
    println!("✓ ACLs applied - AppContainer can now access the allowed directory");
    Ok(())
}

fn launch_acl_probe(profile: &AppContainerProfile, fixture: &AclFixture) -> rappct::Result<()> {
    println!("\n→ Testing file access from AppContainer...");
    println!("  Expected: Can read allowed file, cannot read denied file");
    let caps = SecurityCapabilitiesBuilder::new(&profile.sid).build()?;
    let opts = LaunchOptions {
        exe: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
        cmdline: Some(acl_probe_script(fixture)),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts)?;
    println!("✓ Test process PID: {}", child.pid);
    thread::sleep(Duration::from_secs(4));
    Ok(())
}

fn acl_probe_script(fixture: &AclFixture) -> String {
    format!(
        r#"/C echo [ACL-TEST] Testing file access... && echo [ACL-TEST] Reading allowed file: && type "{}" && echo. && echo [ACL-TEST] Trying denied file (should fail): && type "{}" 2>nul || echo [ACL-TEST] Access denied as expected && timeout /T 3 /NOBREAK >nul"#,
        fixture.allowed_file.display(),
        fixture.denied_file.display()
    )
}
