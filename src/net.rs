//! Network isolation helpers (skeleton). Feature: `net`

use crate::sid::AppContainerSid;
use crate::{AcError, Result};

#[cfg(all(windows, feature = "net"))]
use std::collections::HashSet;

#[cfg(all(windows, feature = "net"))]
use windows::core::PWSTR;

#[cfg(all(windows, feature = "net"))]
use windows::Win32::Security::PSID;

#[cfg(all(windows, feature = "net"))]
#[link(name = "Kernel32")]
extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(all(windows, feature = "net"))]
unsafe fn pwstr_to_string(ptr: PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while *ptr.0.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len))
}

#[cfg(all(windows, feature = "net"))]
unsafe fn psid_to_string(psid: PSID) -> Result<String> {
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    let mut raw = PWSTR::null();
    ConvertSidToStringSidW(psid, &mut raw)
        .map_err(|e| AcError::Win32(format!("ConvertSidToStringSidW failed: {e}")))?;
    let value = pwstr_to_string(raw);
    LocalFree(raw.0 as isize);
    Ok(value)
}

pub fn list_appcontainers() -> Result<Vec<(AppContainerSid, String)>> {
    #[cfg(all(windows, feature = "net"))]
    unsafe {
        use windows::Win32::NetworkManagement::WindowsFirewall::{
            NetworkIsolationEnumAppContainers, NetworkIsolationFreeAppContainers,
            NetworkIsolationGetAppContainerConfig, INET_FIREWALL_APP_CONTAINER,
            NETISO_FLAG_FORCE_COMPUTE_BINARIES,
        };
        use windows::Win32::Security::SID_AND_ATTRIBUTES;

        let mut count: u32 = 0;
        let mut arr: *mut INET_FIREWALL_APP_CONTAINER = std::ptr::null_mut();
        let err = NetworkIsolationEnumAppContainers(
            NETISO_FLAG_FORCE_COMPUTE_BINARIES.0 as u32,
            &mut count,
            &mut arr,
        );
        if err != 0 {
            return Err(AcError::Win32(format!(
                "NetworkIsolationEnumAppContainers failed: {err}"
            )));
        }

        let slice = if arr.is_null() {
            &[][..]
        } else {
            std::slice::from_raw_parts(arr, count as usize)
        };
        let mut out = Vec::with_capacity(slice.len());
        let mut sid_set: HashSet<String> = HashSet::with_capacity(slice.len());
        for item in slice {
            let sid_str = psid_to_string(PSID(item.appContainerSid as *mut _))?;
            let display = pwstr_to_string(item.displayName);
            sid_set.insert(sid_str.clone());
            out.push((AppContainerSid::from_sddl(sid_str), display));
        }
        if !arr.is_null() {
            NetworkIsolationFreeAppContainers(arr);
        }

        let mut cfg_count: u32 = 0;
        let mut cfg_arr: *mut SID_AND_ATTRIBUTES = std::ptr::null_mut();
        let cfg_err = NetworkIsolationGetAppContainerConfig(&mut cfg_count, &mut cfg_arr);
        if cfg_err != 0 {
            return Err(AcError::Win32(format!(
                "NetworkIsolationGetAppContainerConfig failed: {cfg_err}"
            )));
        }
        if !cfg_arr.is_null() {
            let cfg_slice = std::slice::from_raw_parts(cfg_arr, cfg_count as usize);
            for sa in cfg_slice {
                let sid_str = psid_to_string(sa.Sid)?;
                if !sid_set.contains(&sid_str) {
                    LocalFree(cfg_arr as isize);
                    return Err(AcError::Win32(format!(
                        "Firewall config SID {sid_str} missing from enumeration"
                    )));
                }
            }
            LocalFree(cfg_arr as isize);
        }

        Ok(out)
    }
    #[cfg(all(windows, not(feature = "net")))]
    {
        Err(AcError::Unimplemented("net feature not enabled"))
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

/// Safety latch: force explicit acknowledgement before applying loopback exemptions.
pub struct LoopbackAdd(pub AppContainerSid);

pub fn add_loopback_exemption(req: LoopbackAdd) -> Result<()> {
    let _ = &req;
    #[cfg(all(windows, feature = "net"))]
    unsafe {
        // Safety latch: require explicit confirm prior to call
        if !CONFIRM_NEXT.swap(false, std::sync::atomic::Ordering::SeqCst) {
            return Err(AcError::AccessDenied {
                context: "loopback exemption requires confirm_debug_only()".into(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "confirmation missing",
                )),
            });
        }
        set_loopback(true, &req.0)
    }
    #[cfg(all(windows, not(feature = "net")))]
    {
        Err(AcError::Unimplemented("net feature not enabled"))
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

pub fn remove_loopback_exemption(sid: &AppContainerSid) -> Result<()> {
    let _ = sid;
    #[cfg(all(windows, feature = "net"))]
    unsafe {
        set_loopback(false, sid)
    }
    #[cfg(all(windows, not(feature = "net")))]
    {
        Err(AcError::Unimplemented("net feature not enabled"))
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(all(windows, feature = "net"))]
static CONFIRM_NEXT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl LoopbackAdd {
    pub fn confirm_debug_only(self) -> Self {
        #[cfg(all(windows, feature = "net"))]
        {
            CONFIRM_NEXT.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        self
    }
}

#[cfg(all(windows, feature = "net"))]
unsafe fn set_loopback(allow: bool, sid: &AppContainerSid) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::NetworkManagement::WindowsFirewall::{
        NetworkIsolationGetAppContainerConfig, NetworkIsolationSetAppContainerConfig,
    };
    use windows::Win32::Security::Authorization::ConvertStringSidToSidW;
    use windows::Win32::Security::{EqualSid, SID_AND_ATTRIBUTES};

    let mut cur_count: u32 = 0;
    let mut cur_arr: *mut SID_AND_ATTRIBUTES = std::ptr::null_mut();
    let err = NetworkIsolationGetAppContainerConfig(&mut cur_count, &mut cur_arr);
    if err != 0 {
        return Err(AcError::Win32(format!(
            "NetworkIsolationGetAppContainerConfig failed: {err}"
        )));
    }
    let mut vec: Vec<SID_AND_ATTRIBUTES> = if !cur_arr.is_null() {
        std::slice::from_raw_parts(cur_arr, cur_count as usize).to_vec()
    } else {
        Vec::new()
    };

    let sddl_w: Vec<u16> = std::ffi::OsStr::new(sid.as_string())
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut psid_raw = PSID::default();
    ConvertStringSidToSidW(PCWSTR(sddl_w.as_ptr()), &mut psid_raw)
        .map_err(|e| AcError::Win32(format!("ConvertStringSidToSidW failed: {e}")))?;
    let target = psid_raw;

    if allow {
        let mut exists = false;
        for sa in &vec {
            if EqualSid(sa.Sid, target).is_ok() {
                exists = true;
                break;
            }
        }
        if !exists {
            vec.push(SID_AND_ATTRIBUTES {
                Sid: target,
                Attributes: 0,
            });
        }
    } else {
        vec.retain(|sa| EqualSid(sa.Sid, target).is_err());
    }

    let err2 = NetworkIsolationSetAppContainerConfig(&vec);
    if !cur_arr.is_null() {
        LocalFree(cur_arr as isize);
    }
    LocalFree(psid_raw.0 as isize);
    if err2 != 0 {
        return Err(AcError::Win32(format!(
            "NetworkIsolationSetAppContainerConfig failed: {err2}"
        )));
    }
    Ok(())
}

#[cfg(all(windows, not(feature = "net")))]
unsafe fn set_loopback(_allow: bool, _sid: &AppContainerSid) -> Result<()> {
    Err(AcError::Unimplemented("net feature not enabled"))
}

#[cfg(not(windows))]
unsafe fn set_loopback(_allow: bool, _sid: &AppContainerSid) -> Result<()> {
    Err(AcError::UnsupportedPlatform)
}
