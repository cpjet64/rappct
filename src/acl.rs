//! ACL helpers for files/directories and registry keys (DACL grant).
#![allow(clippy::undocumented_unsafe_blocks)]

#[cfg(windows)]
use crate::ffi::mem::LocalAllocGuard;
use crate::sid::AppContainerSid;
use crate::{AcError, Result};

/// ACE inheritance flags for directory ACL grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AceInheritance(pub u32);

impl AceInheritance {
    /// Inherited by child containers and objects (default for directories).
    /// Equivalent to `CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE`.
    pub const SUB_CONTAINERS_AND_OBJECTS: Self = Self(0x3);
    /// Inherited by child containers only (`CONTAINER_INHERIT_ACE`).
    pub const SUB_CONTAINERS_ONLY: Self = Self(0x2);
    /// Inherited by child objects only (`OBJECT_INHERIT_ACE`).
    pub const OBJECTS_ONLY: Self = Self(0x1);
    /// No inheritance — ACE applies only to the directory itself.
    pub const NONE: Self = Self(0x0);
}

/// Target resource for granting AppContainer or capability access.
///
/// Notes:
/// - `RegistryKey` supports only `HKCU` and `HKLM` roots (case-insensitive shorthands
///   `HKCU\\`/`HKLM\\` and full names `HKEY_CURRENT_USER\\`/`HKEY_LOCAL_MACHINE\\`).
/// - `Directory` uses [`AceInheritance::SUB_CONTAINERS_AND_OBJECTS`] by default.
///   Use `DirectoryCustom` to override the inheritance flags.
#[derive(Clone, Debug)]
pub enum ResourcePath {
    File(std::path::PathBuf),
    Directory(std::path::PathBuf),
    /// Directory with custom ACE inheritance flags.
    DirectoryCustom(std::path::PathBuf, AceInheritance),
    RegistryKey(String),
}

#[derive(Clone, Copy, Debug)]
pub struct AccessMask(pub u32);

impl AccessMask {
    /// Full (generic) access commonly used in examples/tests.
    pub const GENERIC_ALL: Self = Self(0x001F_01FF);

    /// FILE_GENERIC_READ access mask.
    #[cfg(windows)]
    pub const FILE_GENERIC_READ: Self =
        Self(windows::Win32::Storage::FileSystem::FILE_GENERIC_READ.0);
    /// FILE_GENERIC_WRITE access mask.
    #[cfg(windows)]
    pub const FILE_GENERIC_WRITE: Self =
        Self(windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE.0);

    /// FILE_GENERIC_READ (non-Windows fallback value)
    #[cfg(not(windows))]
    pub const FILE_GENERIC_READ: Self = Self(0x0001_20089);
    /// FILE_GENERIC_WRITE (non-Windows fallback value)
    #[cfg(not(windows))]
    pub const FILE_GENERIC_WRITE: Self = Self(0x0001_20116);
}

#[cfg_attr(not(windows), allow(unused_variables))]
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

#[cfg_attr(not(windows), allow(unused_variables))]
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
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn grant_sid_access(target: ResourcePath, sid_sddl: &str, access: u32) -> Result<()> {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::Authorization::{
        ConvertStringSidToSidW, EXPLICIT_ACCESS_W, GetNamedSecurityInfoW, GetSecurityInfo,
        SE_FILE_OBJECT, SE_REGISTRY_KEY, SetEntriesInAclW, SetNamedSecurityInfoW, SetSecurityInfo,
        TRUSTEE_FORM, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_TYPE, TRUSTEE_W,
    };
    use windows::Win32::Security::{ACE_FLAGS, ACL, DACL_SECURITY_INFORMATION};
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, RegCloseKey,
        RegOpenKeyExW,
    };
    use windows::core::{PCWSTR, PWSTR};

    // Convert SDDL to PSID
    let wide: Vec<u16> = crate::util::to_utf16(sid_sddl);
    let mut psid = windows::Win32::Security::PSID(std::ptr::null_mut());
    if unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid) }.is_err() {
        return Err(AcError::Win32("ConvertStringSidToSidW failed".into()));
    }
    let psid_guard = unsafe { LocalAllocGuard::from_raw(psid.0) };
    let trustee_psid = windows::Win32::Security::PSID(psid_guard.as_ptr());

    // Build trustee and explicit access
    let mut trustee: TRUSTEE_W = std::mem::zeroed();
    trustee.TrusteeForm = TRUSTEE_FORM(TRUSTEE_IS_SID.0);
    trustee.TrusteeType = TRUSTEE_TYPE(TRUSTEE_IS_WELL_KNOWN_GROUP.0);
    trustee.ptstrName = PWSTR(trustee_psid.0 as *mut _);

    let mut ea: EXPLICIT_ACCESS_W = std::mem::zeroed();
    ea.grfAccessPermissions = access;
    ea.grfAccessMode = windows::Win32::Security::Authorization::GRANT_ACCESS;
    ea.Trustee = trustee;

    match target {
        ResourcePath::File(path) => {
            ea.grfInheritance = ACE_FLAGS(0);
            let path_w: Vec<u16> = crate::util::to_utf16_os(path.as_os_str());
            let mut p_sd = windows::Win32::Security::PSECURITY_DESCRIPTOR(std::ptr::null_mut());
            let mut p_dacl: *mut ACL = std::ptr::null_mut();
            let st = unsafe {
                GetNamedSecurityInfoW(
                    PCWSTR(path_w.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(&mut p_dacl),
                    None,
                    &mut p_sd,
                )
            };
            if st.0 != 0 {
                return Err(AcError::Win32(format!(
                    "GetNamedSecurityInfoW failed: {:?}",
                    st
                )));
            }
            let _sd_guard = unsafe { LocalAllocGuard::from_raw(p_sd.0) };
            let mut new_dacl: *mut ACL = std::ptr::null_mut();
            let entries = [ea];
            let st2 = unsafe {
                SetEntriesInAclW(Some(&entries), Some(p_dacl as *const ACL), &mut new_dacl)
            };
            if st2.0 != 0 {
                return Err(AcError::Win32(format!(
                    "SetEntriesInAclW failed: {:?}",
                    st2
                )));
            }
            let new_dacl_guard = unsafe { LocalAllocGuard::from_raw(new_dacl) };
            let st3 = unsafe {
                SetNamedSecurityInfoW(
                    PCWSTR(path_w.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(new_dacl_guard.as_ptr() as *const ACL),
                    None,
                )
            };
            if st3.0 != 0 {
                return Err(AcError::Win32(format!(
                    "SetNamedSecurityInfoW failed: {:?}",
                    st3
                )));
            }
            Ok(())
        }
        ResourcePath::Directory(ref path) | ResourcePath::DirectoryCustom(ref path, _) => {
            let inheritance = match target {
                ResourcePath::DirectoryCustom(_, flags) => flags.0,
                _ => AceInheritance::SUB_CONTAINERS_AND_OBJECTS.0,
            };
            ea.grfInheritance = ACE_FLAGS(inheritance);
            let path_w: Vec<u16> = crate::util::to_utf16_os(path.as_os_str());
            let mut p_sd = windows::Win32::Security::PSECURITY_DESCRIPTOR(std::ptr::null_mut());
            let mut p_dacl: *mut ACL = std::ptr::null_mut();
            let st = unsafe {
                GetNamedSecurityInfoW(
                    PCWSTR(path_w.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(&mut p_dacl),
                    None,
                    &mut p_sd,
                )
            };
            if st.0 != 0 {
                return Err(AcError::Win32(format!(
                    "GetNamedSecurityInfoW failed: {:?}",
                    st
                )));
            }
            let _sd_guard = unsafe { LocalAllocGuard::from_raw(p_sd.0) };
            let mut new_dacl: *mut ACL = std::ptr::null_mut();
            let entries = [ea];
            let st2 = unsafe {
                SetEntriesInAclW(Some(&entries), Some(p_dacl as *const ACL), &mut new_dacl)
            };
            if st2.0 != 0 {
                return Err(AcError::Win32(format!(
                    "SetEntriesInAclW failed: {:?}",
                    st2
                )));
            }
            let new_dacl_guard = unsafe { LocalAllocGuard::from_raw(new_dacl) };
            let st3 = unsafe {
                SetNamedSecurityInfoW(
                    PCWSTR(path_w.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(new_dacl_guard.as_ptr() as *const ACL),
                    None,
                )
            };
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
                let (root, rest) = if up.strip_prefix("HKCU\\").is_some() {
                    (HKEY_CURRENT_USER, &spec[5..])
                } else if up.strip_prefix("HKEY_CURRENT_USER\\").is_some() {
                    (HKEY_CURRENT_USER, &spec[18..])
                } else if up.strip_prefix("HKLM\\").is_some() {
                    (HKEY_LOCAL_MACHINE, &spec[5..])
                } else if up.strip_prefix("HKEY_LOCAL_MACHINE\\").is_some() {
                    (HKEY_LOCAL_MACHINE, &spec[19..])
                } else {
                    return None;
                };
                let w: Vec<u16> = crate::util::to_utf16(rest);
                Some((root, w))
            }
            let Some((root, subkey_w)) = parse_root(&spec) else {
                return Err(AcError::Win32(
                    "Unsupported registry root (use HKCU or HKLM)".into(),
                ));
            };
            let mut hkey = HKEY(std::ptr::null_mut());
            let st = unsafe {
                RegOpenKeyExW(
                    root,
                    PCWSTR(subkey_w.as_ptr()),
                    Some(0),
                    KEY_READ | KEY_WRITE,
                    &mut hkey,
                )
            };
            if st.0 != 0 {
                return Err(AcError::Win32(format!("RegOpenKeyExW failed: {:?}", st)));
            }

            let mut p_sd = windows::Win32::Security::PSECURITY_DESCRIPTOR(std::ptr::null_mut());
            let mut p_dacl: *mut ACL = std::ptr::null_mut();
            let st2 = unsafe {
                GetSecurityInfo(
                    HANDLE(hkey.0),
                    SE_REGISTRY_KEY,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(&mut p_dacl),
                    None,
                    Some(&mut p_sd),
                )
            };
            if st2.0 != 0 {
                let _ = RegCloseKey(hkey);
                return Err(AcError::Win32(format!(
                    "GetSecurityInfo(reg) failed: {:?}",
                    st2
                )));
            }
            let _sd_guard = unsafe { crate::ffi::mem::LocalAllocGuard::from_raw(p_sd.0) };
            let mut new_dacl: *mut ACL = std::ptr::null_mut();
            let entries = [ea];
            let st3 = unsafe {
                SetEntriesInAclW(Some(&entries), Some(p_dacl as *const ACL), &mut new_dacl)
            };
            if st3.0 != 0 {
                let _ = RegCloseKey(hkey);
                return Err(AcError::Win32(format!(
                    "SetEntriesInAclW(reg) failed: {:?}",
                    st3
                )));
            }
            let new_dacl_guard = unsafe { crate::ffi::mem::LocalAllocGuard::from_raw(new_dacl) };
            let st4 = unsafe {
                SetSecurityInfo(
                    HANDLE(hkey.0),
                    SE_REGISTRY_KEY,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(new_dacl_guard.as_ptr() as *const ACL),
                    None,
                )
            };
            let _ = unsafe { RegCloseKey(hkey) };
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

#[cfg(test)]
mod tests {
    use super::{AccessMask, AceInheritance};

    #[test]
    fn constants_are_consistent() {
        assert_eq!(AccessMask::GENERIC_ALL.0, 0x001F_01FF);
        #[cfg(windows)]
        {
            use windows::Win32::Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE};
            assert_eq!(AccessMask::FILE_GENERIC_READ.0, FILE_GENERIC_READ.0);
            assert_eq!(AccessMask::FILE_GENERIC_WRITE.0, FILE_GENERIC_WRITE.0);
        }
    }

    #[test]
    fn ace_inheritance_constants_match_win32_values() {
        // OBJECT_INHERIT_ACE = 0x1, CONTAINER_INHERIT_ACE = 0x2
        assert_eq!(AceInheritance::NONE.0, 0x0);
        assert_eq!(AceInheritance::OBJECTS_ONLY.0, 0x1);
        assert_eq!(AceInheritance::SUB_CONTAINERS_ONLY.0, 0x2);
        assert_eq!(AceInheritance::SUB_CONTAINERS_AND_OBJECTS.0, 0x3);
    }
}
