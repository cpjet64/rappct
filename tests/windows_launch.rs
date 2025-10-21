#[cfg(windows)]
use rappct::*;

#[cfg(all(windows, feature = "introspection"))]
use rappct::diag::{validate_configuration, ConfigWarning};

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::Security::PSID;

#[cfg(windows)]
#[link(name = "Advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(ProcessHandle: HANDLE, DesiredAccess: u32, TokenHandle: *mut HANDLE)
        -> i32;
}

#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(windows)]
#[repr(C)]
struct TokenAppContainerInformation {
    token_app_container: PSID,
}

#[cfg(windows)]
fn cmd_exe() -> std::path::PathBuf {
    std::path::PathBuf::from("C:/Windows/System32/cmd.exe")
}

#[cfg(windows)]
#[test]
fn launch_ac_cmd_exits() {
    let name = format!("rappct.test.launch.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts).expect("launch ac");
    assert!(child.pid > 0);
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_lpac_cmd_exits_if_supported() {
    if supports_lpac().is_err() {
        return;
    }
    let name = format!("rappct.test.launch.lpac.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults()
        .lpac(true)
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts).expect("launch lpac");
    assert!(child.pid > 0);
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_appcontainer_token_matches_profile() {
    use std::ffi::c_void;
    use std::time::Duration;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::{
        GetTokenInformation, TokenAppContainerSid, TokenIsAppContainer, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    let name = format!("rappct.test.launch.token.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps");
    let expected_caps: Vec<String> = caps.caps.iter().map(|c| c.sid_sddl.clone()).collect();
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C choice /D Y /T 2 > NUL 2>&1".to_string()),
        ..Default::default()
    };
    let child = launch_in_container_with_io(&caps, &opts).expect("launch with token");

    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, child.pid)
            .expect("OpenProcess failed");
        let mut token = HANDLE::default();
        assert_ne!(
            OpenProcessToken(process, TOKEN_QUERY.0, &mut token),
            0,
            "OpenProcessToken failed"
        );

        let mut is_ac: u32 = 0;
        let mut retlen: u32 = 0;
        GetTokenInformation(
            token,
            TokenIsAppContainer,
            Some((&mut is_ac as *mut u32) as *mut c_void),
            std::mem::size_of::<u32>() as u32,
            &mut retlen,
        )
        .expect("TokenIsAppContainer query failed");
        assert_ne!(is_ac, 0, "child token not marked AppContainer");

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
        let token_sid = info.token_app_container;
        assert!(!token_sid.0.is_null(), "token AppContainer SID was null");
        let mut sid_str = PWSTR::null();
        ConvertSidToStringSidW(token_sid, &mut sid_str).expect("ConvertSidToStringSidW failed");
        let sid_value = {
            let mut len = 0usize;
            while *sid_str.0.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(sid_str.0, len);
            String::from_utf16_lossy(slice)
        };
        LocalFree(sid_str.0 as isize);
        assert_eq!(sid_value, prof.sid.as_string(), "child token SID mismatch");
        let mut actual_caps = token_capability_sids(token);
        let mut expected_caps_sorted = expected_caps.clone();
        expected_caps_sorted.sort();
        actual_caps.sort();
        assert_eq!(
            actual_caps, expected_caps_sorted,
            "token capabilities mismatch"
        );
        CloseHandle(token).expect("CloseHandle token");
        CloseHandle(process).expect("CloseHandle process");
    }

    child.wait(Some(Duration::from_secs(5))).expect("wait exit");
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_lpac_token_sets_flag_and_caps() {
    if supports_lpac().is_err() {
        return;
    }
    use std::ffi::c_void;
    use std::time::Duration;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::{
        GetTokenInformation, TokenAppContainerSid, TokenIsAppContainer,
        TokenIsLessPrivilegedAppContainer, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    let name = format!("rappct.test.launch.lpac.token.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .unwrap()
        .with_lpac_defaults()
        .unwrap()
        .lpac(true)
        .build()
        .expect("build caps");
    let expected_caps: Vec<String> = caps.caps.iter().map(|c| c.sid_sddl.clone()).collect();
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C choice /D Y /T 2 > NUL 2>&1".to_string()),
        ..Default::default()
    };
    let child = launch_in_container_with_io(&caps, &opts).expect("launch lpac token");

    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, child.pid)
            .expect("OpenProcess failed");
        let mut token = HANDLE::default();
        assert_ne!(
            OpenProcessToken(process, TOKEN_QUERY.0, &mut token),
            0,
            "OpenProcessToken failed"
        );

        let mut is_ac: u32 = 0;
        let mut retlen: u32 = 0;
        GetTokenInformation(
            token,
            TokenIsAppContainer,
            Some((&mut is_ac as *mut u32) as *mut c_void),
            std::mem::size_of::<u32>() as u32,
            &mut retlen,
        )
        .expect("TokenIsAppContainer query failed");
        assert_ne!(is_ac, 0, "child token not marked AppContainer");

        retlen = 0;
        let mut is_lpac = windows::core::BOOL(0);
        let lpac_result = GetTokenInformation(
            token,
            TokenIsLessPrivilegedAppContainer,
            Some((&mut is_lpac) as *mut _ as *mut c_void),
            std::mem::size_of::<windows::core::BOOL>() as u32,
            &mut retlen,
        );
        use windows::Win32::Foundation::ERROR_INVALID_PARAMETER;
        match lpac_result {
            Ok(()) => assert!(is_lpac.as_bool(), "child token not marked LPAC"),
            Err(err)
                if err.code()
                    == windows::core::HRESULT::from_win32(ERROR_INVALID_PARAMETER.0) =>
            {
                // Windows builds before TokenIsLessPrivilegedAppContainer support return E_INVALIDARG.
            }
            Err(err) => panic!("TokenIsLessPrivilegedAppContainer query failed: {:?}", err),
        }

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
        let token_sid = info.token_app_container;
        assert!(!token_sid.0.is_null(), "token AppContainer SID was null");
        let mut sid_str = PWSTR::null();
        ConvertSidToStringSidW(token_sid, &mut sid_str).expect("ConvertSidToStringSidW failed");
        let sid_value = {
            let mut len = 0usize;
            while *sid_str.0.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(sid_str.0, len);
            String::from_utf16_lossy(slice)
        };
        LocalFree(sid_str.0 as isize);
        assert_eq!(sid_value, prof.sid.as_string(), "child token SID mismatch");

        let mut actual_caps = token_capability_sids(token);
        let mut expected_caps_sorted = expected_caps.clone();
        expected_caps_sorted.sort();
        actual_caps.sort();
        assert_eq!(
            actual_caps, expected_caps_sorted,
            "token capabilities mismatch"
        );

        CloseHandle(token).expect("CloseHandle token");
        CloseHandle(process).expect("CloseHandle process");
    }

    child.wait(Some(Duration::from_secs(5))).expect("wait exit");
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_ac_with_job_limits() {
    let name = format!("rappct.test.launch.job.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: Some(32 * 1024 * 1024),
            cpu_rate_percent: Some(50),
            kill_on_job_close: false,
        }),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts).expect("launch with job limits");
    assert!(child.pid > 0);
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_job_limits_reported_by_query() {
    use std::ffi::c_void;
    use std::time::Duration;
    use windows::Win32::Foundation::STILL_ACTIVE;
    use windows::Win32::System::JobObjects::{
        JobObjectCpuRateControlInformation, JobObjectExtendedLimitInformation,
        QueryInformationJobObject, JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
        JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };

    let name = format!("rappct.test.launch.jobinfo.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps");
    let memory_limit: usize = 8 * 1024 * 1024;
    let cpu_percent: u32 = 25;
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C timeout /T 30 /NOBREAK > NUL".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: Some(memory_limit),
            cpu_rate_percent: Some(cpu_percent),
            kill_on_job_close: true,
        }),
        ..Default::default()
    };
    let mut child = launch_in_container_with_io(&caps, &opts).expect("launch with job limits");
    let job_handle = child
        .job_guard
        .as_ref()
        .expect("job guard missing")
        .as_handle();

    unsafe {
        let mut ext: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        QueryInformationJobObject(
            Some(job_handle),
            JobObjectExtendedLimitInformation,
            &mut ext as *mut _ as *mut c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            None,
        )
        .expect("QueryInformationJobObject(ext) failed");
        assert_ne!(
            (ext.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_PROCESS_MEMORY).0,
            0,
            "process memory limit flag not set",
        );
        assert_ne!(
            (ext.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE).0,
            0,
            "kill-on-close flag not set",
        );
        assert_eq!(
            ext.ProcessMemoryLimit, memory_limit,
            "memory limit mismatch"
        );
    }

    unsafe {
        let mut cpu: JOBOBJECT_CPU_RATE_CONTROL_INFORMATION = std::mem::zeroed();
        QueryInformationJobObject(
            Some(job_handle),
            JobObjectCpuRateControlInformation,
            &mut cpu as *mut _ as *mut c_void,
            std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
            None,
        )
        .expect("QueryInformationJobObject(cpu) failed");
        assert_ne!(
            (cpu.ControlFlags & JOB_OBJECT_CPU_RATE_CONTROL_ENABLE).0,
            0,
            "cpu control enable not set",
        );
        assert_ne!(
            (cpu.ControlFlags & JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP).0,
            0,
            "cpu hard cap not set",
        );
        let cpu_rate = cpu.Anonymous.CpuRate;
        assert_eq!(cpu_rate, cpu_percent * 100, "cpu rate mismatch");
    }

    let guard = child.job_guard.take().expect("job guard missing");
    drop(guard);
    let exit = child
        .wait(Some(Duration::from_secs(5)))
        .expect("wait after dropping guard");
    assert_ne!(
        exit, STILL_ACTIVE.0 as u32,
        "child still active after guard drop"
    );
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_job_guard_drop_terminates_process() {
    use std::time::{Duration, Instant};
    use windows::Win32::Foundation::STILL_ACTIVE;

    let name = format!("rappct.test.launch.jobkill.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .unwrap()
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C timeout /T 30 /NOBREAK > NUL".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: None,
            cpu_rate_percent: None,
            kill_on_job_close: true,
        }),
        ..Default::default()
    };
    let mut child = launch_in_container_with_io(&caps, &opts).expect("launch with kill-on-close");
    let guard = child.job_guard.take().expect("job guard missing");
    drop(guard);
    let start = Instant::now();
    let exit = child
        .wait(Some(Duration::from_secs(5)))
        .expect("wait after guard drop");
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "job guard drop did not terminate in time"
    );
    assert_ne!(exit, STILL_ACTIVE.0 as u32, "child remained active");
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_with_pipes_and_echo() {
    let name = format!("rappct.test.launch.pipes.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .unwrap()
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C echo hello".to_string()),
        stdio: StdioConfig::Pipe,
        ..Default::default()
    };
    #[cfg(feature = "introspection")]
    {
        let warnings = validate_configuration(&caps, &opts);
        assert!(
            warnings.is_empty(),
            "unexpected diagnostics for pipe launch: {:?}",
            warnings
        );
    }
    let child = launch_in_container_with_io(&caps, &opts).expect("launch with io");
    use std::io::Read;
    let mut s = String::new();
    child.stdout.unwrap().read_to_string(&mut s).unwrap();
    assert!(s.to_lowercase().contains("hello"));
    prof.delete().ok();
}

#[cfg(windows)]
#[test]
fn launch_waits_for_exit_code() {
    let name = format!("rappct.test.launch.wait.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");
    let caps = SecurityCapabilitiesBuilder::new(&prof.sid)
        .with_known(&[KnownCapability::InternetClient])
        .unwrap()
        .build()
        .expect("build caps");
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

#[cfg(all(windows, feature = "introspection"))]
#[test]
fn diagnostics_reports_missing_caps() {
    let name = format!("rappct.test.launch.diag.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure");

    let caps_no_network = SecurityCapabilitiesBuilder::new(&prof.sid)
        .build()
        .expect("build caps");
    let opts = LaunchOptions {
        exe: cmd_exe(),
        ..Default::default()
    };
    let warnings_no_net = validate_configuration(&caps_no_network, &opts);
    assert!(warnings_no_net.contains(&ConfigWarning::NoNetworkCaps));

    let caps_lpac_missing = SecurityCapabilitiesBuilder::new(&prof.sid)
        .lpac(true)
        .build()
        .expect("build caps");
    let warnings_lpac = validate_configuration(&caps_lpac_missing, &opts);
    assert!(warnings_lpac.contains(&ConfigWarning::LpacWithoutCommonCaps));

    prof.delete().ok();
}

#[cfg(windows)]
fn token_capability_sids(token: HANDLE) -> Vec<String> {
    use std::ffi::c_void;
    use windows::core::PWSTR;
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::{GetTokenInformation, TokenCapabilities, TOKEN_GROUPS};

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
        let count = groups.GroupCount as usize;
        let slice = std::slice::from_raw_parts(groups.Groups.as_ptr(), count);
        let mut caps = Vec::with_capacity(count);
        for entry in slice {
            let mut sid_str = PWSTR::null();
            ConvertSidToStringSidW(entry.Sid, &mut sid_str).expect("ConvertSidToStringSidW failed");
            let sid_value = {
                let mut len = 0usize;
                while *sid_str.0.add(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(sid_str.0, len);
                String::from_utf16_lossy(slice)
            };
            LocalFree(sid_str.0 as isize);
            caps.push(sid_value);
        }
        caps
    }
}
