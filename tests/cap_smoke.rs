#![cfg(windows)]

use rappct::Result;
use rappct::capability::CapabilityName;
use rappct::profile::AppContainerProfile;
use rappct::test_support::{AppSid, AttributeList, CatalogCaps};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    CreateProcessW, EXTENDED_STARTUPINFO_PRESENT, GetExitCodeProcess, PROCESS_INFORMATION,
    STARTUPINFOEXW, STARTUPINFOW, WaitForSingleObject,
};
use windows::core::{PCWSTR, PWSTR};

const CMD_EXE: &str = "C:\\Windows\\System32\\cmd.exe";
const CMD_ARGS: &str = " /C ver";

#[test]
fn lpac_launch_with_known_caps() -> Result<()> {
    if std::env::var("RAPPCT_ITESTS").ok().as_deref() != Some("1") {
        println!("skipped cap_smoke::lpac_launch_with_known_caps (set RAPPCT_ITESTS=1 to run)");
        return Ok(());
    }

    let mut profile = Some(AppContainerProfile::ensure(
        "rappct.cap_smoke",
        "rappct",
        Some("capability smoke test"),
    )?);
    let profile_sid = profile.as_ref().map(|p| p.sid.clone()).unwrap();

    let app_sid = AppSid::from_app_container(&profile_sid)?;
    let caps = CatalogCaps::from_catalog(
        app_sid,
        &[
            CapabilityName::InternetClient,
            CapabilityName::PrivateNetworkClientServer,
        ],
    )?;
    let mut attr_list = AttributeList::with_capacity(1)?;
    attr_list.set_security_capabilities(&caps)?;

    let exe_w = rappct::util::win::to_utf16(CMD_EXE);
    let mut args_w = rappct::util::win::to_utf16(CMD_ARGS);

    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.lpAttributeList = attr_list.as_mut_ptr();

    let mut proc_info = PROCESS_INFORMATION::default();
    unsafe {
        CreateProcessW(
            PCWSTR(exe_w.as_ptr()),
            Some(PWSTR(args_w.as_mut_ptr())),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            PCWSTR::null(),
            &mut startup as *mut STARTUPINFOEXW as *mut STARTUPINFOW,
            &mut proc_info,
        )
        .map_err(|e| rappct::AcError::Win32(format!("CreateProcessW failed: {e}")))?;

        let wait_result = WaitForSingleObject(proc_info.hProcess, 30_000);
        assert_eq!(
            wait_result, WAIT_OBJECT_0,
            "process did not exit within timeout"
        );

        let mut exit_code = 1u32;
        GetExitCodeProcess(proc_info.hProcess, &mut exit_code)
            .map_err(|e| rappct::AcError::Win32(format!("GetExitCodeProcess failed: {e}")))?;
        assert_eq!(exit_code, 0, "cmd /C ver should exit with code 0");

        let _ = CloseHandle(proc_info.hThread);
        let _ = CloseHandle(proc_info.hProcess);
    }

    if let Some(profile) = profile.take() {
        profile.delete()?;
    }
    Ok(())
}
