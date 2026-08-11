use std::path::Path;

use crate::ffi::mem::LocalAllocGuard;
use crate::{AcError, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::Authorization::{
    ConvertStringSidToSidW, EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW,
    GetSecurityInfo, SE_FILE_OBJECT, SE_REGISTRY_KEY, SetEntriesInAclW, SetNamedSecurityInfoW,
    SetSecurityInfo, TRUSTEE_FORM, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_TYPE,
    TRUSTEE_W,
};
use windows::Win32::Security::{
    ACE_FLAGS, ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, REG_SAM_FLAGS, RegCloseKey, RegOpenKeyExW,
};
use windows::core::{PCWSTR, PWSTR};

use super::{AceInheritance, ResourcePath};

pub(super) fn grant_sid_access(target: ResourcePath, sid_sddl: &str, access: u32) -> Result<()> {
    validate_existing_resource(&target)?;
    let trustee = SidTrustee::from_sddl(sid_sddl)?;
    match target {
        ResourcePath::File(path) => {
            grant_file_system_path(&path, AceInheritance::NONE, access, trustee.as_trustee())
        }
        ResourcePath::Directory(path) => grant_file_system_path(
            &path,
            AceInheritance::SUB_CONTAINERS_AND_OBJECTS,
            access,
            trustee.as_trustee(),
        ),
        ResourcePath::DirectoryCustom(path, inheritance) => {
            grant_file_system_path(&path, inheritance, access, trustee.as_trustee())
        }
        ResourcePath::RegistryKey(spec) => grant_registry_key(&spec, access, trustee.as_trustee()),
    }
}

fn validate_existing_resource(target: &ResourcePath) -> Result<()> {
    match target {
        ResourcePath::File(path) if !path.is_file() => Err(file_not_found(path)),
        ResourcePath::Directory(path) | ResourcePath::DirectoryCustom(path, _)
            if !path.is_dir() =>
        {
            Err(AcError::ResourceNotFound {
                path: path.display().to_string(),
                hint: "create the directory before calling grant_to_package()",
            })
        }
        _ => Ok(()),
    }
}

fn file_not_found(path: &Path) -> AcError {
    let hint = if path.exists() {
        "expected a file path; use ResourcePath::Directory for directories"
    } else {
        "create the file before calling grant_to_package()"
    };
    AcError::ResourceNotFound {
        path: path.display().to_string(),
        hint,
    }
}

struct SidTrustee {
    _sid: LocalAllocGuard<std::ffi::c_void>,
    trustee: TRUSTEE_W,
}

impl SidTrustee {
    fn from_sddl(sid_sddl: &str) -> Result<Self> {
        let wide = crate::ffi::wstr::to_utf16(sid_sddl);
        let mut psid = PSID(std::ptr::null_mut());
        // SAFETY: `wide` is valid NUL-terminated UTF-16, and `psid` receives a LocalAlloc SID.
        unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid) }
            .map_err(|_| AcError::Win32("ConvertStringSidToSidW failed".into()))?;
        // SAFETY: ConvertStringSidToSidW allocates the SID with LocalAlloc.
        let sid = unsafe { LocalAllocGuard::from_raw(psid.0) };
        Ok(Self {
            trustee: sid_trustee(sid.as_ptr()),
            _sid: sid,
        })
    }

    fn as_trustee(&self) -> TRUSTEE_W {
        self.trustee
    }
}

fn sid_trustee(psid: *mut std::ffi::c_void) -> TRUSTEE_W {
    let mut trustee: TRUSTEE_W = unsafe_zeroed();
    trustee.TrusteeForm = TRUSTEE_FORM(TRUSTEE_IS_SID.0);
    trustee.TrusteeType = TRUSTEE_TYPE(TRUSTEE_IS_WELL_KNOWN_GROUP.0);
    trustee.ptstrName = PWSTR(psid as *mut _);
    trustee
}

fn grant_file_system_path(
    path: &Path,
    inheritance: AceInheritance,
    access: u32,
    trustee: TRUSTEE_W,
) -> Result<()> {
    let path_w = crate::ffi::wstr::to_utf16_os(path.as_os_str());
    let dacl = read_named_dacl(&path_w)?;
    if dacl.old_dacl.is_null() {
        return Ok(());
    }
    let new_dacl = merge_dacl(
        dacl.old_dacl,
        access,
        inheritance,
        trustee,
        "SetEntriesInAclW",
    )?;
    apply_named_dacl(&path_w, new_dacl.as_ptr())
}

fn read_named_dacl(path_w: &[u16]) -> Result<SecurityDescriptorDacl> {
    let mut p_sd = PSECURITY_DESCRIPTOR(std::ptr::null_mut());
    let mut p_dacl = std::ptr::null_mut();
    // SAFETY: Win32 receives a NUL-terminated path and returns DACL/descriptor pointers.
    let status = unsafe {
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
    if status.0 != 0 {
        return Err(AcError::Win32(format!(
            "GetNamedSecurityInfoW failed: {status:?}"
        )));
    }
    Ok(SecurityDescriptorDacl::new(p_sd, p_dacl))
}

fn apply_named_dacl(path_w: &[u16], new_dacl: *mut ACL) -> Result<()> {
    // SAFETY: The path and DACL pointer are valid for the duration of the call.
    let status = unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(path_w.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_dacl as *const ACL),
            None,
        )
    };
    win32_status(status.0, "SetNamedSecurityInfoW")
}

fn grant_registry_key(spec: &str, access: u32, trustee: TRUSTEE_W) -> Result<()> {
    let (root, subkey_w) = parse_registry_root(spec)?;
    let key = RegistryKeyHandle::open(root, &subkey_w)?;
    let dacl = read_registry_dacl(key.raw())?;
    if dacl.old_dacl.is_null() {
        return Ok(());
    }
    let new_dacl = merge_dacl(
        dacl.old_dacl,
        access,
        AceInheritance::NONE,
        trustee,
        "SetEntriesInAclW(reg)",
    )?;
    apply_registry_dacl(key.raw(), new_dacl.as_ptr())
}

fn parse_registry_root(spec: &str) -> Result<(HKEY, Vec<u16>)> {
    for (prefix, root) in registry_roots() {
        if spec
            .get(..prefix.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
        {
            return Ok((root, crate::ffi::wstr::to_utf16(&spec[prefix.len()..])));
        }
    }
    Err(AcError::Win32(
        "Unsupported registry root (use HKCU or HKLM)".into(),
    ))
}

fn registry_roots() -> [(&'static str, HKEY); 4] {
    [
        ("HKCU\\", HKEY_CURRENT_USER),
        ("HKEY_CURRENT_USER\\", HKEY_CURRENT_USER),
        ("HKLM\\", HKEY_LOCAL_MACHINE),
        ("HKEY_LOCAL_MACHINE\\", HKEY_LOCAL_MACHINE),
    ]
}

struct RegistryKeyHandle(HKEY);

impl RegistryKeyHandle {
    fn open(root: HKEY, subkey_w: &[u16]) -> Result<Self> {
        let mut hkey = HKEY(std::ptr::null_mut());
        // SAFETY: Open the key with only the rights required to read and replace its DACL.
        let status = unsafe {
            RegOpenKeyExW(
                root,
                PCWSTR(subkey_w.as_ptr()),
                Some(0),
                REG_SAM_FLAGS(0x0002_0000 | 0x0004_0000),
                &mut hkey,
            )
        };
        if status.0 != 0 {
            return Err(AcError::Win32(format!("RegOpenKeyExW failed: {status:?}")));
        }
        Ok(Self(hkey))
    }

    fn raw(&self) -> HKEY {
        self.0
    }
}

impl Drop for RegistryKeyHandle {
    fn drop(&mut self) {
        // SAFETY: This handle is owned by the guard and closed exactly once.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

fn read_registry_dacl(hkey: HKEY) -> Result<SecurityDescriptorDacl> {
    let mut p_sd = PSECURITY_DESCRIPTOR(std::ptr::null_mut());
    let mut p_dacl = std::ptr::null_mut();
    // SAFETY: Query security info for an open registry key handle.
    let status = unsafe {
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
    if status.0 != 0 {
        return Err(AcError::Win32(format!(
            "GetSecurityInfo(reg) failed: {status:?}"
        )));
    }
    Ok(SecurityDescriptorDacl::new(p_sd, p_dacl))
}

fn apply_registry_dacl(hkey: HKEY, new_dacl: *mut ACL) -> Result<()> {
    // SAFETY: Apply new DACL to an open registry key.
    let status = unsafe {
        SetSecurityInfo(
            HANDLE(hkey.0),
            SE_REGISTRY_KEY,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_dacl as *const ACL),
            None,
        )
    };
    win32_status(status.0, "SetSecurityInfo(reg)")
}

struct SecurityDescriptorDacl {
    _sd: LocalAllocGuard<std::ffi::c_void>,
    old_dacl: *mut ACL,
}

impl SecurityDescriptorDacl {
    fn new(sd: PSECURITY_DESCRIPTOR, old_dacl: *mut ACL) -> Self {
        // SAFETY: Security descriptors returned by Get*SecurityInfo are LocalAlloc-managed.
        let sd = unsafe { LocalAllocGuard::from_raw(sd.0) };
        Self { _sd: sd, old_dacl }
    }
}

fn merge_dacl(
    old_dacl: *mut ACL,
    access: u32,
    inheritance: AceInheritance,
    trustee: TRUSTEE_W,
    context: &'static str,
) -> Result<LocalAllocGuard<ACL>> {
    let entry = explicit_access(access, inheritance, trustee);
    let mut new_dacl = std::ptr::null_mut();
    // SAFETY: SetEntriesInAclW consumes a valid entry slice and returns a LocalAlloc ACL.
    let status =
        unsafe { SetEntriesInAclW(Some(&[entry]), Some(old_dacl as *const ACL), &mut new_dacl) };
    win32_status(status.0, context)?;
    // SAFETY: The API returned a LocalAlloc-managed ACL on success.
    Ok(unsafe { LocalAllocGuard::from_raw(new_dacl) })
}

fn explicit_access(
    access: u32,
    inheritance: AceInheritance,
    trustee: TRUSTEE_W,
) -> EXPLICIT_ACCESS_W {
    let mut entry: EXPLICIT_ACCESS_W = unsafe_zeroed();
    entry.grfAccessPermissions = access;
    entry.grfAccessMode = GRANT_ACCESS;
    entry.grfInheritance = ACE_FLAGS(inheritance.0);
    entry.Trustee = trustee;
    entry
}

fn win32_status(status: u32, context: &str) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(AcError::Win32(format!("{context} failed: {status:?}")))
    }
}

fn unsafe_zeroed<T>() -> T {
    // SAFETY: Win32 record types used here are plain old data initialized before use.
    unsafe { std::mem::zeroed() }
}
