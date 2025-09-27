//! ACL helpers for files/directories and registry keys (DACL grant).

use crate::sid::AppContainerSid;
use crate::{AcError, Result};

#[derive(Clone, Debug)]
pub enum ResourcePath {
    File(std::path::PathBuf),
    Directory(std::path::PathBuf),
    RegistryKey(String),
}

#[derive(Clone, Copy, Debug)]
pub struct AccessMask(pub u32);

pub fn grant_to_package(
    target: ResourcePath,
    sid: &AppContainerSid,
    access: AccessMask,
) -> Result<()> {
    #[cfg(windows)]
    unsafe {
        grant_sid_access(target, sid.as_string(), access.0)
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

pub fn grant_to_capability(
    target: ResourcePath,
    cap_sid_sddl: &str,
    access: AccessMask,
) -> Result<()> {
    #[cfg(windows)]
    unsafe {
        grant_sid_access(target, cap_sid_sddl, access.0)
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
unsafe fn grant_sid_access(target: ResourcePath, sid_sddl: &str, access: u32) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::Authorization::{
        ConvertStringSidToSidW, GetNamedSecurityInfoW, GetSecurityInfo, SetEntriesInAclW,
        SetNamedSecurityInfoW, SetSecurityInfo, EXPLICIT_ACCESS_W, SE_FILE_OBJECT, SE_REGISTRY_KEY,
        TRUSTEE_FORM, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_TYPE, TRUSTEE_W,
    };
    use windows::Win32::Security::{ACE_FLAGS, ACL, DACL_SECURITY_INFORMATION};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ,
        KEY_WRITE,
    };

    #[link(name = "Kernel32")]
    extern "system" {
        fn LocalFree(h: isize) -> isize;
    }

    // Convert SDDL to PSID
    let wide: Vec<u16> = std::ffi::OsStr::new(sid_sddl)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut psid = windows::Win32::Security::PSID(std::ptr::null_mut());
    if ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid).is_err() {
        return Err(AcError::Win32("ConvertStringSidToSidW failed".into()));
    }

    // Build trustee and explicit access
    let mut trustee: TRUSTEE_W = std::mem::zeroed();
    trustee.TrusteeForm = TRUSTEE_FORM(TRUSTEE_IS_SID.0);
    trustee.TrusteeType = TRUSTEE_TYPE(TRUSTEE_IS_WELL_KNOWN_GROUP.0);
    trustee.ptstrName = PWSTR(psid.0 as *mut _);

    let mut ea: EXPLICIT_ACCESS_W = std::mem::zeroed();
    ea.grfAccessPermissions = access;
    ea.grfAccessMode = windows::Win32::Security::Authorization::GRANT_ACCESS;
    ea.Trustee = trustee;

    match target {
        ResourcePath::File(path) => {
            ea.grfInheritance = ACE_FLAGS(0);
            let path_w: Vec<u16> = std::os::windows::ffi::OsStrExt::encode_wide(path.as_os_str())
                .chain(std::iter::once(0))
                .collect();
            let mut p_sd = windows::Win32::Security::PSECURITY_DESCRIPTOR(std::ptr::null_mut());
            let mut p_dacl: *mut ACL = std::ptr::null_mut();
            let st = GetNamedSecurityInfoW(
                PCWSTR(path_w.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(&mut p_dacl),
                None,
                &mut p_sd,
            );
            if st.0 != 0 {
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!(
                    "GetNamedSecurityInfoW failed: {:?}",
                    st
                )));
            }
            let mut new_dacl: *mut ACL = std::ptr::null_mut();
            let entries = [ea];
            let st2 = SetEntriesInAclW(Some(&entries), Some(p_dacl as *const ACL), &mut new_dacl);
            if st2.0 != 0 {
                if !p_sd.0.is_null() {
                    let _ = LocalFree(p_sd.0 as isize);
                }
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!(
                    "SetEntriesInAclW failed: {:?}",
                    st2
                )));
            }
            let st3 = SetNamedSecurityInfoW(
                PCWSTR(path_w.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(new_dacl as *const ACL),
                None,
            );
            if !p_sd.0.is_null() {
                let _ = LocalFree(p_sd.0 as isize);
            }
            if !new_dacl.is_null() {
                let _ = LocalFree(new_dacl as isize);
            }
            let _ = LocalFree(psid.0 as isize);
            if st3.0 != 0 {
                return Err(AcError::Win32(format!(
                    "SetNamedSecurityInfoW failed: {:?}",
                    st3
                )));
            }
            Ok(())
        }
        ResourcePath::Directory(path) => {
            ea.grfInheritance = ACE_FLAGS(0x3u32); // SUB_CONTAINERS_AND_OBJECTS_INHERIT
            let path_w: Vec<u16> = std::os::windows::ffi::OsStrExt::encode_wide(path.as_os_str())
                .chain(std::iter::once(0))
                .collect();
            let mut p_sd = windows::Win32::Security::PSECURITY_DESCRIPTOR(std::ptr::null_mut());
            let mut p_dacl: *mut ACL = std::ptr::null_mut();
            let st = GetNamedSecurityInfoW(
                PCWSTR(path_w.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(&mut p_dacl),
                None,
                &mut p_sd,
            );
            if st.0 != 0 {
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!(
                    "GetNamedSecurityInfoW failed: {:?}",
                    st
                )));
            }
            let mut new_dacl: *mut ACL = std::ptr::null_mut();
            let entries = [ea];
            let st2 = SetEntriesInAclW(Some(&entries), Some(p_dacl as *const ACL), &mut new_dacl);
            if st2.0 != 0 {
                if !p_sd.0.is_null() {
                    let _ = LocalFree(p_sd.0 as isize);
                }
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!(
                    "SetEntriesInAclW failed: {:?}",
                    st2
                )));
            }
            let st3 = SetNamedSecurityInfoW(
                PCWSTR(path_w.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(new_dacl as *const ACL),
                None,
            );
            if !p_sd.0.is_null() {
                let _ = LocalFree(p_sd.0 as isize);
            }
            if !new_dacl.is_null() {
                let _ = LocalFree(new_dacl as isize);
            }
            let _ = LocalFree(psid.0 as isize);
            if st3.0 != 0 {
                return Err(AcError::Win32(format!(
                    "SetNamedSecurityInfoW failed: {:?}",
                    st3
                )));
            }
            Ok(())
        }
        ResourcePath::RegistryKey(spec) => {
            // Parse root and subkey
            fn parse_root(spec: &str) -> Option<(HKEY, Vec<u16>)> {
                let up = spec.to_ascii_uppercase();
                let (root, rest) = if let Some(_) = up.strip_prefix("HKCU\\") {
                    (HKEY_CURRENT_USER, &spec[5..])
                } else if let Some(_) = up.strip_prefix("HKEY_CURRENT_USER\\") {
                    (HKEY_CURRENT_USER, &spec[18..])
                } else if let Some(_) = up.strip_prefix("HKLM\\") {
                    (HKEY_LOCAL_MACHINE, &spec[5..])
                } else if let Some(_) = up.strip_prefix("HKEY_LOCAL_MACHINE\\") {
                    (HKEY_LOCAL_MACHINE, &spec[19..])
                } else {
                    return None;
                };
                let w: Vec<u16> = std::ffi::OsStr::new(rest)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                Some((root, w))
            }
            let Some((root, subkey_w)) = parse_root(&spec) else {
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(
                    "Unsupported registry root (use HKCU or HKLM)".into(),
                ));
            };
            let mut hkey = HKEY(std::ptr::null_mut());
            let st = RegOpenKeyExW(
                root,
                PCWSTR(subkey_w.as_ptr()),
                Some(0),
                KEY_READ | KEY_WRITE,
                &mut hkey,
            );
            if st.0 != 0 {
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!("RegOpenKeyExW failed: {:?}", st)));
            }

            let mut p_sd = windows::Win32::Security::PSECURITY_DESCRIPTOR(std::ptr::null_mut());
            let mut p_dacl: *mut ACL = std::ptr::null_mut();
            let st2 = GetSecurityInfo(
                HANDLE(hkey.0),
                SE_REGISTRY_KEY,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(&mut p_dacl),
                None,
                Some(&mut p_sd),
            );
            if st2.0 != 0 {
                let _ = RegCloseKey(hkey);
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!(
                    "GetSecurityInfo(reg) failed: {:?}",
                    st2
                )));
            }
            let mut new_dacl: *mut ACL = std::ptr::null_mut();
            let entries = [ea];
            let st3 = SetEntriesInAclW(Some(&entries), Some(p_dacl as *const ACL), &mut new_dacl);
            if st3.0 != 0 {
                if !p_sd.0.is_null() {
                    let _ = LocalFree(p_sd.0 as isize);
                }
                let _ = RegCloseKey(hkey);
                let _ = LocalFree(psid.0 as isize);
                return Err(AcError::Win32(format!(
                    "SetEntriesInAclW(reg) failed: {:?}",
                    st3
                )));
            }
            let st4 = SetSecurityInfo(
                HANDLE(hkey.0),
                SE_REGISTRY_KEY,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(new_dacl as *const ACL),
                None,
            );
            if !p_sd.0.is_null() {
                let _ = LocalFree(p_sd.0 as isize);
            }
            if !new_dacl.is_null() {
                let _ = LocalFree(new_dacl as isize);
            }
            let _ = RegCloseKey(hkey);
            let _ = LocalFree(psid.0 as isize);
            if st4.0 != 0 {
                return Err(AcError::Win32(format!(
                    "SetSecurityInfo(reg) failed: {:?}",
                    st4
                )));
            }
            Ok(())
        }
    }
}
