use super::{cmd_exe, windows_test_utils::LocalWideString};
use rappct::*;
use std::{ffi::c_void, time::Duration};
use windows::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, HANDLE};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, PSID, TOKEN_GROUPS, TOKEN_QUERY, TokenAppContainerSid, TokenCapabilities,
    TokenIsAppContainer, TokenIsLessPrivilegedAppContainer,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::core::PWSTR;

#[link(name = "Advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(ProcessHandle: HANDLE, DesiredAccess: u32, TokenHandle: *mut HANDLE)
    -> i32;
}

#[repr(C)]
struct TokenAppContainerInformation {
    token_app_container: PSID,
}

struct ProcessToken {
    process: HANDLE,
    token: HANDLE,
}

impl Drop for ProcessToken {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.token);
            let _ = CloseHandle(self.process);
        }
    }
}

#[test]
fn launch_appcontainer_token_matches_profile() {
    let (prof, child, expected_caps) = launch_token_probe("token", false);
    let process_token = open_child_process_token(child.pid);
    assert_is_appcontainer(process_token.token);
    assert_token_sid_matches(process_token.token, &prof);
    assert_token_caps_match(process_token.token, &expected_caps);

    child.wait(Some(Duration::from_secs(5))).expect("wait exit");
    prof.delete().ok();
}

#[test]
fn launch_lpac_token_sets_flag_and_caps() {
    if supports_lpac().is_err() {
        return;
    }
    let (prof, child, expected_caps) = launch_token_probe("lpac.token", true);
    let process_token = open_child_process_token(child.pid);
    assert_is_appcontainer(process_token.token);
    assert_lpac_flag(process_token.token);
    assert_token_sid_matches(process_token.token, &prof);
    assert_token_caps_match(process_token.token, &expected_caps);

    child.wait(Some(Duration::from_secs(5))).expect("wait exit");
    prof.delete().ok();
}

fn launch_token_probe(
    scope: &str,
    lpac: bool,
) -> (AppContainerProfile, rappct::launch::LaunchedIo, Vec<String>) {
    let name = format!("rappct.test.launch.{scope}.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let mut builder =
        SecurityCapabilitiesBuilder::new(&prof.sid).with_known(&[KnownCapability::InternetClient]);
    if lpac {
        builder = builder.with_lpac_defaults().lpac(true);
    }
    let caps = builder.build().expect("build caps");
    let expected_caps = caps.caps.iter().map(|c| c.sid_sddl.clone()).collect();
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C choice /D Y /T 2 > NUL 2>&1".to_string()),
        ..Default::default()
    };
    let child = launch_in_container_with_io(&caps, &opts).expect("launch token probe");
    (prof, child, expected_caps)
}

fn open_child_process_token(pid: u32) -> ProcessToken {
    unsafe {
        let process =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).expect("OpenProcess failed");
        let mut token = HANDLE::default();
        assert_ne!(
            OpenProcessToken(process, TOKEN_QUERY.0, &mut token),
            0,
            "OpenProcessToken failed"
        );
        ProcessToken { process, token }
    }
}

fn assert_is_appcontainer(token: HANDLE) {
    let mut is_ac: u32 = 0;
    let mut retlen: u32 = 0;
    unsafe {
        GetTokenInformation(
            token,
            TokenIsAppContainer,
            Some((&mut is_ac as *mut u32) as *mut c_void),
            std::mem::size_of::<u32>() as u32,
            &mut retlen,
        )
        .expect("TokenIsAppContainer query failed");
    }
    assert_ne!(is_ac, 0, "child token not marked AppContainer");
}

fn assert_lpac_flag(token: HANDLE) {
    let mut retlen = 0;
    let mut is_lpac = windows::core::BOOL(0);
    let result = unsafe {
        GetTokenInformation(
            token,
            TokenIsLessPrivilegedAppContainer,
            Some((&mut is_lpac) as *mut _ as *mut c_void),
            std::mem::size_of::<windows::core::BOOL>() as u32,
            &mut retlen,
        )
    };
    match result {
        Ok(()) => assert!(is_lpac.as_bool(), "child token not marked LPAC"),
        Err(err) if err.code() == windows::core::HRESULT::from_win32(ERROR_INVALID_PARAMETER.0) => {
        }
        Err(err) => panic!("TokenIsLessPrivilegedAppContainer query failed: {err:?}"),
    }
}

fn assert_token_sid_matches(token: HANDLE, prof: &AppContainerProfile) {
    let sid_value = token_appcontainer_sid(token);
    assert_eq!(sid_value, prof.sid.as_string(), "child token SID mismatch");
}

fn token_appcontainer_sid(token: HANDLE) -> String {
    unsafe {
        let mut needed: u32 = 0;
        let _ = GetTokenInformation(token, TokenAppContainerSid, None, 0, &mut needed);
        assert!(
            needed as usize >= std::mem::size_of::<TokenAppContainerInformation>(),
            "TokenAppContainerSid size too small"
        );
        let mut buffer = vec![0u8; needed as usize];
        GetTokenInformation(
            token,
            TokenAppContainerSid,
            Some(buffer.as_mut_ptr() as *mut c_void),
            needed,
            &mut needed,
        )
        .expect("TokenAppContainerSid query failed");
        let info = std::ptr::read_unaligned(buffer.as_ptr() as *const TokenAppContainerInformation);
        assert!(!info.token_app_container.0.is_null(), "token SID was null");
        sid_to_string(info.token_app_container)
    }
}

fn assert_token_caps_match(token: HANDLE, expected_caps: &[String]) {
    let mut actual_caps = token_capability_sids(token);
    let mut expected_caps_sorted = expected_caps.to_vec();
    expected_caps_sorted.sort();
    actual_caps.sort();
    assert_eq!(
        actual_caps, expected_caps_sorted,
        "token capabilities mismatch"
    );
}

fn token_capability_sids(token: HANDLE) -> Vec<String> {
    unsafe {
        let mut needed: u32 = 0;
        let _ = GetTokenInformation(token, TokenCapabilities, None, 0, &mut needed);
        if needed == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u8; needed as usize];
        GetTokenInformation(
            token,
            TokenCapabilities,
            Some(buf.as_mut_ptr() as *mut c_void),
            needed,
            &mut needed,
        )
        .expect("TokenCapabilities query failed");

        let groups = &*(buf.as_ptr() as *const TOKEN_GROUPS);
        let slice = std::slice::from_raw_parts(groups.Groups.as_ptr(), groups.GroupCount as usize);
        slice.iter().map(|entry| sid_to_string(entry.Sid)).collect()
    }
}

fn sid_to_string(sid: PSID) -> String {
    unsafe {
        let mut sid_str = PWSTR::null();
        ConvertSidToStringSidW(sid, &mut sid_str).expect("ConvertSidToStringSidW failed");
        LocalWideString::from_raw(sid_str).to_string_lossy()
    }
}
