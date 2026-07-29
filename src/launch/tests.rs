use super::{JobObjectDropGuard, LaunchOptions, LaunchedIo, make_cmd_args, merge_parent_env};
use crate::capability::CapabilityName;
use crate::ffi::sec_caps::OwnedSecurityCapabilities;
use crate::ffi::sid::OwnedSid;
use std::ffi::OsString;
use std::os::windows::io::{AsRawHandle, BorrowedHandle};
use std::rc::Rc;
use std::time::Duration;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::CreateEventW;

#[test]
fn merge_parent_env_includes_essentials_if_present() {
    let out = merge_parent_env(vec![(OsString::from("RAPPCT_X"), OsString::from("1"))]);
    assert!(out.iter().any(|(k, v)| k == "RAPPCT_X" && v == "1"));
    if std::env::var_os("SystemRoot").is_some() {
        assert!(out.iter().any(|(k, _)| k == "SystemRoot"));
    }
}

#[test]
fn merge_parent_env_does_not_duplicate_existing_keys() {
    let out = merge_parent_env(vec![(
        OsString::from("systemroot"),
        OsString::from("X:\\Custom"),
    )]);
    let matches = out
        .iter()
        .filter(|(k, _)| k.to_string_lossy().eq_ignore_ascii_case("SystemRoot"))
        .count();
    assert_eq!(matches, 1, "SystemRoot should not be duplicated");
    assert!(
        out.iter().any(
            |(k, v)| k.to_string_lossy().eq_ignore_ascii_case("systemroot") && v == "X:\\Custom"
        )
    );
}

#[test]
fn merge_parent_env_keeps_case_insensitive_last_writer_wins() {
    let out = merge_parent_env(vec![
        (OsString::from("RAPPCT_X"), OsString::from("1")),
        (OsString::from("systemroot"), OsString::from("X:\\Lower")),
        (OsString::from("SystemRoot"), OsString::from("Y:\\Upper")),
    ]);
    let matches = out
        .iter()
        .filter(|(k, _)| k.to_string_lossy().eq_ignore_ascii_case("SystemRoot"))
        .count();
    assert_eq!(
        matches, 1,
        "case-insensitive duplicates must collapse to one key"
    );
    assert!(
        out.iter().any(
            |(k, v)| k.to_string_lossy().eq_ignore_ascii_case("SystemRoot") && v == "Y:\\Upper"
        )
    );
}

#[test]
fn make_cmd_args_handles_none_and_some() {
    assert!(make_cmd_args(&None).is_none());

    let args = make_cmd_args(&Some(" /C exit 0".to_string())).expect("expected args");
    assert_eq!(args.last().copied(), Some(0));
    assert_eq!(
        String::from_utf16_lossy(&args[..args.len() - 1]),
        " /C exit 0"
    );
}

#[test]
fn with_env_merge_accumulates_existing_and_added_values() {
    let opts = LaunchOptions {
        env: Some(vec![(OsString::from("A"), OsString::from("1"))]),
        ..Default::default()
    }
    .with_env_merge(&[(OsString::from("B"), OsString::from("2"))]);

    let env = opts.env.expect("env should be populated");
    assert!(env.iter().any(|(k, v)| k == "A" && v == "1"));
    assert!(env.iter().any(|(k, v)| k == "B" && v == "2"));
}

#[test]
fn with_handle_list_and_stdio_inherit_duplicate_handles() {
    let file = std::fs::File::open("C:\\Windows\\System32\\cmd.exe").expect("open fixture");
    // SAFETY: Borrowed handle is valid while `file` remains alive.
    let borrowed = unsafe { BorrowedHandle::borrow_raw(file.as_raw_handle()) };
    let opts = LaunchOptions::default()
        .try_with_handle_list(&[borrowed])
        .expect("duplicate handle list")
        .try_with_stdio_inherit(Some(borrowed), Some(borrowed), Some(borrowed))
        .expect("duplicate stdio");

    assert_eq!(opts.extra.handle_list.len(), 1);
    assert_ne!(opts.extra.handle_list[0].as_win32().0, std::ptr::null_mut());
    assert_ne!(opts.extra.handle_list[0].as_win32().0, file.as_raw_handle());
    assert!(opts.extra.stdio.stdin.is_some());
    assert!(opts.extra.stdio.stdout.is_some());
    assert!(opts.extra.stdio.stderr.is_some());
}

#[test]
fn handle_list_survives_source_handle_drop() {
    let opts = {
        let file = std::fs::File::open("C:\\Windows\\System32\\cmd.exe").expect("open fixture");
        // SAFETY: Borrowed handle is live while `file` remains in this block.
        let borrowed = unsafe { BorrowedHandle::borrow_raw(file.as_raw_handle()) };
        LaunchOptions::default()
            .try_with_handle_list(&[borrowed])
            .expect("duplicate handle list")
    };

    let borrowed_duplicate = opts.extra.handle_list[0].as_borrowed();
    let round_trip = crate::ffi::handles::duplicate_handle(borrowed_duplicate, false)
        .expect("duplicate stored handle");
    assert_ne!(round_trip.as_win32().0, std::ptr::null_mut());
}

#[test]
fn launch_options_default_matches_expected_windows_shape() {
    let opts = LaunchOptions::default();
    assert_eq!(
        opts.exe,
        std::path::PathBuf::from("C:\\Windows\\System32\\notepad.exe")
    );
    assert_eq!(
        opts.cwd,
        Some(std::path::PathBuf::from("C:\\Windows\\System32"))
    );
    assert!(opts.cmdline.is_none());
    assert!(opts.env.is_none());
    assert!(matches!(opts.stdio, super::StdioConfig::Inherit));
    assert!(!opts.suspended);
    assert!(opts.join_job.is_none());
    assert!(opts.startup_timeout.is_none());
}

#[test]
fn job_object_drop_guard_disable_flips_state_and_invalid_assign_fails() {
    let mut guard = JobObjectDropGuard::new().expect("create guard");
    assert!(guard.kill_on_drop_enabled());
    assert_ne!(guard.as_handle().0, std::ptr::null_mut());
    guard.disable_kill_on_drop().expect("disable kill on drop");
    assert!(!guard.kill_on_drop_enabled());

    let err = guard
        .assign_process_handle(HANDLE::default())
        .expect_err("assigning null process handle should fail");
    assert!(err.to_string().contains("AssignProcessToJobObject"));
}

#[test]
fn inherit_list_tracks_pushed_handles_and_raw_slice() {
    let mut list = super::attributes::InheritList::default();
    assert!(list.is_empty());

    // SAFETY: CreateEventW returns an owned event handle on success.
    let event = unsafe { CreateEventW(None, true, false, None).expect("create event") };
    let owned = crate::ffi::handles::from_win32(event).expect("wrap event handle");
    let raw = owned.as_win32();

    list.push(owned);
    assert!(!list.is_empty());
    assert_eq!(list.slice(), &[raw]);
}

#[test]
fn launched_io_wait_returns_timeout_for_unsignaled_waitable_handle() {
    // SAFETY: CreateEventW returns an owned event handle on success.
    let event = unsafe { CreateEventW(None, true, false, None).expect("create event") };
    let process = crate::ffi::handles::from_win32(event).expect("wrap event handle");
    let io = LaunchedIo {
        pid: 42,
        stdin: None,
        stdout: None,
        stderr: None,
        job_guard: None,
        process,
    };

    let err = io
        .wait(Some(Duration::from_millis(1)))
        .expect_err("unsignaled handle should time out");
    match err {
        crate::AcError::LaunchFailed { stage, hint, .. } => {
            assert_eq!(stage, "wait");
            assert_eq!(hint, "timeout");
        }
        other => panic!("expected timeout LaunchFailed, got: {other:?}"),
    }
}

#[test]
fn with_security_capabilities_sets_internal_override() {
    let sid = OwnedSid::from_sddl("S-1-15-2-1").expect("owned sid");
    let caps = OwnedSecurityCapabilities::from_catalog(sid, &[CapabilityName::InternetClient])
        .expect("security caps");
    let opts = LaunchOptions::default().with_security_capabilities(caps);
    assert!(opts.extra.security_caps.is_some());
}

#[test]
fn effective_startup_timeout_skips_suspended_launches() {
    let suspended = LaunchOptions {
        suspended: true,
        startup_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };
    let active = LaunchOptions {
        suspended: false,
        startup_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };

    assert!(super::startup::effective_startup_timeout(&suspended).is_none());
    assert_eq!(
        super::startup::effective_startup_timeout(&active),
        Some(Duration::from_secs(5))
    );
}

#[test]
fn inflate_security_caps_prefers_override_when_provided() {
    let sec = crate::capability::SecurityCapabilities {
        package: crate::sid::AppContainerSid::from_sddl("S-1-15-2-1"),
        caps: Vec::new(),
        lpac: false,
    };
    let sid = OwnedSid::from_sddl("S-1-15-2-1").expect("owned sid");
    let override_caps = Rc::new(
        OwnedSecurityCapabilities::from_catalog(sid, &[CapabilityName::InternetClient])
            .expect("override caps"),
    );
    let got = super::attributes::inflate_security_caps(&sec, Some(override_caps.clone()))
        .expect("inflate with override");
    assert!(Rc::ptr_eq(&got, &override_caps));
}

#[test]
fn duplicate_additional_handles_preserves_shared_handles() {
    let mut list = super::attributes::InheritList::default();
    let file = std::fs::File::open("C:\\Windows\\System32\\cmd.exe").expect("open fixture");
    // SAFETY: Borrowed handle is valid while `file` remains alive.
    let borrowed = unsafe { BorrowedHandle::borrow_raw(file.as_raw_handle()) };
    let opts = LaunchOptions::default()
        .try_with_handle_list(&[borrowed])
        .expect("duplicate handle list");

    super::attributes::duplicate_additional_handles(&opts.extra.handle_list, &mut list)
        .expect("register handles");
    assert!(!list.is_empty());
    assert_eq!(list.slice(), &[opts.extra.handle_list[0].as_win32()]);
}
