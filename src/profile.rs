//! AppContainer profile management (skeleton).
//! - Create/open/delete
//! - Resolve folder and named-object paths
#![allow(clippy::undocumented_unsafe_blocks)]

#[cfg(windows)]
use crate::ffi::{mem::LocalAllocGuard, sid::OwnedSid, wstr::WideString};
use crate::sid::AppContainerSid;
use crate::{AcError, Result};

#[derive(Clone, Debug)]
pub struct AppContainerProfile {
    pub name: String,
    pub sid: AppContainerSid,
}

impl AppContainerProfile {
    /// Create or open profile idempotently.
    pub fn ensure(_name: &str, _display: &str, _description: Option<&str>) -> Result<Self> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_INVALID_PARAMETER};
            use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
            use windows::core::{HRESULT, PCWSTR, PWSTR};

            #[link(name = "Userenv")]
            unsafe extern "system" {
                fn CreateAppContainerProfile(
                    pszAppContainerName: PCWSTR,
                    pszDisplayName: PCWSTR,
                    pszDescription: PCWSTR,
                    pCapabilities: *mut core::ffi::c_void,
                    dwCapabilityCount: u32,
                    ppSidAppContainerSid: *mut *mut core::ffi::c_void,
                ) -> HRESULT;
                fn DeriveAppContainerSidFromAppContainerName(
                    pszAppContainerName: PCWSTR,
                    ppSidAppContainerSid: *mut *mut core::ffi::c_void,
                ) -> HRESULT;
            }

            unsafe {
                // Prepare wide strings using stable WideString buffers
                let name_w = WideString::from_str(_name);
                let display_w = WideString::from_str(_display);
                let desc_w = _description.map(WideString::from_str);
                let desc_ptr = desc_w
                    .as_ref()
                    .map(|w| w.as_pcwstr())
                    .unwrap_or(PCWSTR::null());

                let mut sid_ptr = std::ptr::null_mut();
                // Attempt to create
                let hr: HRESULT = CreateAppContainerProfile(
                    name_w.as_pcwstr(),
                    display_w.as_pcwstr(),
                    desc_ptr,
                    std::ptr::null_mut(),
                    0,
                    &mut sid_ptr,
                );

                let already_exists = HRESULT::from_win32(ERROR_ALREADY_EXISTS.0);
                let invalid_parameter = HRESULT::from_win32(ERROR_INVALID_PARAMETER.0);
                let sid_owned = if hr.is_ok() {
                    OwnedSid::from_freesid_psid(sid_ptr)
                } else if hr == already_exists || hr == invalid_parameter {
                    // Fallback to derive SID when profile already exists or metadata mismatches
                    let mut sid2 = std::ptr::null_mut();
                    let hr2 =
                        DeriveAppContainerSidFromAppContainerName(name_w.as_pcwstr(), &mut sid2);
                    if !hr2.is_ok() {
                        return Err(AcError::Win32(format!(
                            "DeriveAppContainerSidFromAppContainerName failed: 0x{:08X}",
                            hr2.0
                        )));
                    }
                    OwnedSid::from_freesid_psid(sid2)
                } else {
                    return Err(AcError::Win32(format!(
                        "CreateAppContainerProfile failed: 0x{:08X}",
                        hr.0
                    )));
                };

                // Convert to SDDL
                let mut sddl_ptr = PWSTR::null();
                ConvertSidToStringSidW(sid_owned.as_psid(), &mut sddl_ptr)
                    .map_err(|e| AcError::Win32(format!("ConvertSidToStringSidW failed: {}", e)))?;
                let sddl_guard = LocalAllocGuard::<u16>::from_raw(sddl_ptr.0);
                let sddl = sddl_guard.to_string_lossy();

                Ok(Self {
                    name: _name.to_string(),
                    sid: AppContainerSid::from_sddl(sddl),
                })
            }
        }
        #[cfg(not(windows))]
        {
            Err(AcError::UnsupportedPlatform)
        }
    }
    pub fn delete(self) -> Result<()> {
        #[cfg(windows)]
        {
            // No local imports needed here
            #[link(name = "Userenv")]
            unsafe extern "system" {
                fn DeleteAppContainerProfile(
                    pszAppContainerName: windows::core::PCWSTR,
                ) -> windows::core::HRESULT;
            }
            let name_w = WideString::from_str(&self.name);
            unsafe {
                let hr = DeleteAppContainerProfile(name_w.as_pcwstr());
                if !hr.is_ok() {
                    return Err(AcError::Win32(format!(
                        "DeleteAppContainerProfile failed: 0x{:08X}",
                        hr.0
                    )));
                }
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Err(AcError::UnsupportedPlatform)
        }
    }
    pub fn folder_path(&self) -> Result<std::path::PathBuf> {
        #[cfg(windows)]
        {
            use std::path::PathBuf;
            #[link(name = "Userenv")]
            unsafe extern "system" {
                fn GetAppContainerFolderPath(
                    pszAppContainerSid: windows::Win32::Security::PSID,
                    ppszPath: *mut windows::core::PWSTR,
                ) -> windows::core::HRESULT;
            }
            #[link(name = "Userenv")]
            unsafe extern "system" {
                fn DeriveAppContainerSidFromAppContainerName(
                    name: windows::core::PCWSTR,
                    sid: *mut windows::Win32::Security::PSID,
                ) -> windows::core::HRESULT;
            }
            use windows::core::PWSTR;
            unsafe {
                // Derive package PSID from name for folder query
                let name_w = WideString::from_str(&self.name);
                let mut psid = windows::Win32::Security::PSID(std::ptr::null_mut());
                let hr_sid =
                    DeriveAppContainerSidFromAppContainerName(name_w.as_pcwstr(), &mut psid);
                if !hr_sid.is_ok() {
                    return Err(AcError::Win32(format!(
                        "DeriveAppContainerSidFromAppContainerName failed: 0x{:08X}",
                        hr_sid.0
                    )));
                }
                let psid_owned = OwnedSid::from_freesid_psid(psid.0);
                let mut out: PWSTR = PWSTR::null();
                let hr = GetAppContainerFolderPath(psid_owned.as_psid(), &mut out);
                if hr.is_ok() {
                    // Convert returned PWSTR to PathBuf
                    let path = {
                        let guard = crate::ffi::mem::CoTaskMem::<u16>::from_raw(out.0);
                        let mut len = 0usize;
                        while *guard.as_ptr().add(len) != 0 {
                            len += 1;
                        }
                        let slice = std::slice::from_raw_parts(guard.as_ptr(), len);
                        let s = String::from_utf16_lossy(slice);
                        PathBuf::from(s)
                    };
                    Ok(path)
                } else {
                    // Fallback: synthesize under LocalAppData\Packages\{SID}
                    let sid_s = self.sid.as_string().to_string();
                    match std::env::var_os("LOCALAPPDATA") {
                        Some(base) => Ok(PathBuf::from(base).join("Packages").join(sid_s)),
                        None => Err(AcError::Win32(format!(
                            "GetAppContainerFolderPath failed: 0x{:08X}",
                            hr.0
                        ))),
                    }
                }
            }
        }
        #[cfg(not(windows))]
        {
            Err(AcError::UnsupportedPlatform)
        }
    }
    pub fn named_object_path(&self) -> Result<String> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::Security::Authorization::ConvertStringSidToSidW;
            use windows::core::PWSTR;
            #[link(name = "Userenv")]
            unsafe extern "system" {
                fn GetAppContainerNamedObjectPath(
                    token: windows::Win32::Foundation::HANDLE,
                    appcontainersid: *mut core::ffi::c_void,
                    length: u32,
                    path: windows::core::PWSTR,
                    returnlength: *mut u32,
                ) -> i32;
            }
            unsafe {
                // Convert SDDL to PSID
                let sddl_w = WideString::from_str(self.sid.as_string());
                let mut psid = windows::Win32::Security::PSID(std::ptr::null_mut());
                if ConvertStringSidToSidW(sddl_w.as_pcwstr(), &mut psid).is_err() {
                    return Err(AcError::Win32("ConvertStringSidToSidW failed".into()));
                }
                let psid_guard = LocalAllocGuard::<std::ffi::c_void>::from_raw(psid.0);
                let mut needed: u32 = 0;
                // First call to get required length (chars including NUL)
                let _ = GetAppContainerNamedObjectPath(
                    HANDLE::default(),
                    psid_guard.as_ptr(),
                    0,
                    PWSTR::null(),
                    &mut needed,
                );
                if needed == 0 {
                    return Err(AcError::Win32(
                        "GetAppContainerNamedObjectPath size query failed".into(),
                    ));
                }
                let mut buf: Vec<u16> = vec![0u16; needed as usize];
                let mut retlen: u32 = 0;
                let ok = GetAppContainerNamedObjectPath(
                    HANDLE::default(),
                    psid_guard.as_ptr(),
                    needed,
                    PWSTR(buf.as_mut_ptr()),
                    &mut retlen,
                );
                if ok == 0 {
                    return Err(AcError::Win32(
                        "GetAppContainerNamedObjectPath failed".into(),
                    ));
                }
                // Convert to String without trailing NUL
                let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                let s = String::from_utf16_lossy(&buf[..len]);
                Ok(s)
            }
        }
        #[cfg(not(windows))]
        {
            Err(AcError::UnsupportedPlatform)
        }
    }
}

pub fn derive_sid_from_name(name: &str) -> Result<AppContainerSid> {
    let _ = name;
    #[cfg(windows)]
    {
        #[link(name = "Userenv")]
        unsafe extern "system" {
            fn DeriveAppContainerSidFromAppContainerName(
                name: windows::core::PCWSTR,
                sid: *mut *mut core::ffi::c_void,
            ) -> windows::core::HRESULT;
        }
        use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
        use windows::core::{PCWSTR, PWSTR};
        unsafe {
            let name_w = WideString::from_str(name);
            let mut sid_ptr = std::ptr::null_mut();
            let hr = DeriveAppContainerSidFromAppContainerName(name_w.as_pcwstr(), &mut sid_ptr);
            if !hr.is_ok() {
                return Err(AcError::Win32(format!(
                    "DeriveAppContainerSidFromAppContainerName failed: 0x{:08X}",
                    hr.0
                )));
            }
            let sid_owned = OwnedSid::from_freesid_psid(sid_ptr);
            let mut sddl_ptr = PWSTR::null();
            if ConvertSidToStringSidW(sid_owned.as_psid(), &mut sddl_ptr).is_err() {
                return Err(AcError::Win32("ConvertSidToStringSidW failed".into()));
            }
            let sddl_guard = LocalAllocGuard::<u16>::from_raw(sddl_ptr.0);
            let sddl = sddl_guard.to_string_lossy();
            Ok(AppContainerSid::from_sddl(sddl))
        }
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}
