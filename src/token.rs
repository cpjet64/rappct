//! Token introspection (skeleton).
#![allow(clippy::undocumented_unsafe_blocks)]

use crate::sid::AppContainerSid;
#[cfg(windows)]
use crate::ffi::mem::LocalAllocGuard;
use crate::{AcError, Result};

#[cfg(windows)]
use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_PARAMETER, HANDLE,
};
#[cfg(windows)]
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
#[cfg(windows)]
use windows::Win32::Security::{
    GetTokenInformation, TOKEN_APPCONTAINER_INFORMATION, TOKEN_GROUPS, TOKEN_INFORMATION_CLASS,
    TOKEN_QUERY, TokenAppContainerSid, TokenCapabilities, TokenIsAppContainer,
    TokenIsLessPrivilegedAppContainer,
};
#[cfg(windows)]
use windows::Win32::System::Threading::GetCurrentProcess;

#[cfg(windows)]
use windows::core::HRESULT;

#[cfg(windows)]
#[link(name = "Advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(
        ProcessHandle: windows::Win32::Foundation::HANDLE,
        DesiredAccess: u32,
        TokenHandle: *mut windows::Win32::Foundation::HANDLE,
    ) -> i32;
}

#[cfg(windows)]
struct HandleGuard(HANDLE);

#[cfg(windows)]
impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[derive(Debug, Default)]
pub struct TokenInfo {
    pub is_appcontainer: bool,
    pub is_lpac: bool,
    pub package_sid: Option<AppContainerSid>,
    pub capability_sids: Vec<String>,
}

pub fn query_current_process_token() -> Result<TokenInfo> {
    #[cfg(windows)]
    {
        unsafe {
            let mut raw = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY.0, &mut raw) == 0 {
                return Err(AcError::Win32("OpenProcessToken failed".into()));
            }
            let guard = HandleGuard(raw);
            let token = guard.0;

            let info = TokenInfo {
                is_appcontainer: query_bool(token, TokenIsAppContainer)?,
                is_lpac: query_bool(token, TokenIsLessPrivilegedAppContainer)?,
                package_sid: query_appcontainer_sid(token)?,
                capability_sids: query_capabilities(token)?,
            };
            Ok(info)
        }
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn query_bool(token: HANDLE, class: TOKEN_INFORMATION_CLASS) -> Result<bool> {
    let mut value: u32 = 0;
    let mut retlen: u32 = 0;
    match unsafe {
        GetTokenInformation(
            token,
            class,
            Some((&mut value) as *mut _ as *mut _),
            std::mem::size_of::<u32>() as u32,
            &mut retlen,
        )
    } {
        Ok(_) => Ok(value != 0),
        Err(err) => {
            if is_win32_error(&err, ERROR_INVALID_PARAMETER.0) {
                Ok(false)
            } else {
                Err(AcError::Win32(format!(
                    "GetTokenInformation(class={:?}) failed: {}",
                    class.0, err
                )))
            }
        }
    }
}

#[cfg(windows)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn query_appcontainer_sid(token: HANDLE) -> Result<Option<AppContainerSid>> {
    let mut needed: u32 = 0;
    if let Err(err) =
        unsafe { GetTokenInformation(token, TokenAppContainerSid, None, 0, &mut needed) }
    {
        if is_win32_error(&err, ERROR_INVALID_PARAMETER.0) {
            return Ok(None);
        }
        if !is_win32_error(&err, ERROR_INSUFFICIENT_BUFFER.0) {
            return Err(AcError::Win32(format!(
                "GetTokenInformation(TokenAppContainerSid size) failed: {}",
                err
            )));
        }
    }
    if needed == 0 {
        return Ok(None);
    }
    let mut buffer = vec![0u8; needed as usize];
    unsafe {
        GetTokenInformation(
            token,
            TokenAppContainerSid,
            Some(buffer.as_mut_ptr() as *mut _),
            needed,
            &mut needed,
        )
    }
    .map_err(|e| {
        AcError::Win32(format!(
            "GetTokenInformation(TokenAppContainerSid) failed: {}",
            e
        ))
    })?;

    let info_ptr = buffer.as_ptr() as *const TOKEN_APPCONTAINER_INFORMATION;
    let sid = unsafe { (*info_ptr).TokenAppContainer };
    if sid.0.is_null() {
        return Ok(None);
    }
    let sid_string = sid_to_string(sid)?;
    Ok(Some(AppContainerSid::from_sddl(sid_string)))
}

#[cfg(windows)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn query_capabilities(token: HANDLE) -> Result<Vec<String>> {
    let mut needed: u32 = 0;
    if let Err(err) = unsafe { GetTokenInformation(token, TokenCapabilities, None, 0, &mut needed) }
    {
        if is_win32_error(&err, ERROR_INVALID_PARAMETER.0) {
            return Ok(Vec::new());
        }
        if !is_win32_error(&err, ERROR_INSUFFICIENT_BUFFER.0) {
            return Err(AcError::Win32(format!(
                "GetTokenInformation(TokenCapabilities size) failed: {}",
                err
            )));
        }
    }
    if needed == 0 {
        return Ok(Vec::new());
    }
    let mut buffer = vec![0u8; needed as usize];
    unsafe {
        GetTokenInformation(
            token,
            TokenCapabilities,
            Some(buffer.as_mut_ptr() as *mut _),
            needed,
            &mut needed,
        )
    }
    .map_err(|e| {
        AcError::Win32(format!(
            "GetTokenInformation(TokenCapabilities) failed: {}",
            e
        ))
    })?;

    let groups = buffer.as_ptr() as *const TOKEN_GROUPS;
    let count = unsafe { (*groups).GroupCount as usize };
    let mut out = Vec::with_capacity(count);
    if count == 0 {
        return Ok(out);
    }
    let slice = unsafe { std::slice::from_raw_parts((*groups).Groups.as_ptr(), count) };
    for entry in slice {
        if entry.Sid.0.is_null() {
            continue;
        }
        let sid_str = unsafe { sid_to_string(entry.Sid)? };
        out.push(sid_str);
    }
    Ok(out)
}

#[cfg(windows)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sid_to_string(psid: windows::Win32::Security::PSID) -> Result<String> {
    if psid.0.is_null() {
        return Err(AcError::Win32(
            "ConvertSidToStringSidW received null SID".into(),
        ));
    }
    let mut out = windows::core::PWSTR::null();
    unsafe { ConvertSidToStringSidW(psid, &mut out) }
        .map_err(|e| AcError::Win32(format!("ConvertSidToStringSidW failed: {}", e)))?;
    let guard = unsafe { LocalAllocGuard::<u16>::from_raw(out.0) };
    Ok(unsafe { guard.to_string_lossy() })
}

#[cfg(windows)]
fn is_win32_error(err: &windows::core::Error, code: u32) -> bool {
    err.code() == HRESULT::from_win32(code)
}
