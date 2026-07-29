use super::cmd_exe;
use rappct::*;

fn launch_profile(scope: &str) -> AppContainerProfile {
    let name = format!("rappct.test.launch.{scope}.{}", std::process::id());
    AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure")
}

fn internet_caps(profile: &AppContainerProfile) -> SecurityCapabilities {
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps")
}

#[test]
fn launch_with_pipes_and_echo() {
    let prof = launch_profile("pipes");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C echo hello".to_string()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    };
    assert_pipe_diagnostics(&caps, &opts);
    let child = launch_in_container_with_io(&caps, &opts).expect("launch with io");
    let mut s = String::new();
    use std::io::Read;
    child.stdout.unwrap().read_to_string(&mut s).unwrap();
    assert!(s.to_lowercase().contains("hello"));
    prof.delete().ok();
}

#[cfg(feature = "introspection")]
fn assert_pipe_diagnostics(caps: &SecurityCapabilities, opts: &LaunchOptions) {
    let warnings = rappct::diag::validate_configuration(caps, opts);
    assert!(
        warnings.is_empty(),
        "unexpected diagnostics for pipe launch: {warnings:?}"
    );
}

#[cfg(not(feature = "introspection"))]
fn assert_pipe_diagnostics(_caps: &SecurityCapabilities, _opts: &LaunchOptions) {}

#[test]
fn launch_waits_for_exit_code() {
    let prof = launch_profile("wait");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 7".to_string()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    };
    let child = launch_in_container_with_io(&caps, &opts).expect("launch with io");
    let code = child
        .wait(Some(std::time::Duration::from_secs(5)))
        .expect("wait exit");
    assert_eq!(code, 7);
    prof.delete().ok();
}

#[test]
fn launch_with_null_stdio_has_no_parent_streams() {
    let prof = launch_profile("nullstdio");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        stdio: StdioConfig::Null,
        ..Default::default()
    };
    let child = launch_in_container_with_io(&caps, &opts).expect("launch with null stdio");
    assert!(child.stdin.is_none(), "stdin should be detached");
    assert!(child.stdout.is_none(), "stdout should be detached");
    assert!(child.stderr.is_none(), "stderr should be detached");
    let code = child
        .wait(Some(std::time::Duration::from_secs(5)))
        .expect("wait exit");
    assert_eq!(code, 0);
    prof.delete().ok();
}

#[test]
fn launch_with_explicit_handle_list_succeeds() {
    use std::os::windows::io::{AsRawHandle, BorrowedHandle};

    let prof = launch_profile("handles");
    let caps = internet_caps(&prof);
    let fixture = std::fs::File::open(cmd_exe()).expect("open fixture");
    let borrowed = unsafe { BorrowedHandle::borrow_raw(fixture.as_raw_handle()) };
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    }
    .with_handle_list(&[borrowed]);

    let child = launch_in_container_with_io(&caps, &opts).expect("launch with explicit handles");
    let code = child
        .wait(Some(std::time::Duration::from_secs(5)))
        .expect("wait exit");
    assert_eq!(code, 0);
    prof.delete().ok();
}

#[test]
fn launch_with_stdio_inherit_overrides_succeeds() {
    use std::os::windows::io::{AsRawHandle, BorrowedHandle};

    let prof = launch_profile("inherit");
    let caps = internet_caps(&prof);
    let fixture = std::fs::File::open(cmd_exe()).expect("open fixture");
    let borrowed = unsafe { BorrowedHandle::borrow_raw(fixture.as_raw_handle()) };
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    }
    .with_stdio_inherit(Some(borrowed), Some(borrowed), Some(borrowed));

    let child = launch_in_container_with_io(&caps, &opts).expect("launch with stdio overrides");
    let code = child
        .wait(Some(std::time::Duration::from_secs(5)))
        .expect("wait exit");
    assert_eq!(code, 0);
    prof.delete().ok();
}
