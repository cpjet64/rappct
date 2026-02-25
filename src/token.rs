//! Token introspection (skeleton).

#[cfg(windows)]
use crate::ffi::mem::LocalAllocGuard;
use crate::sid::AppContainerSid;
use crate::{AcError, Result};

#[cfg(windows)]
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_PARAMETER, HANDLE};
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

// Use crate-private RAII handle wrapper for safety

/// Information extracted from a process token about its AppContainer status.
#[derive(Debug, Default)]
pub struct TokenInfo {
    /// Whether the token belongs to an AppContainer process.
    pub is_appcontainer: bool,
    /// Whether the token belongs to a Less Privileged AppContainer (LPAC) process.
    pub is_lpac: bool,
    /// The package SID if the process is running inside an AppContainer.
    pub package_sid: Option<AppContainerSid>,
    /// SDDL strings of all capability SIDs granted to the token.
    pub capability_sids: Vec<String>,
}

/// Queries the current process token for AppContainer/LPAC status and capabilities.
pub fn query_current_process_token() -> Result<TokenInfo> {
    #[cfg(windows)]
    {
        // SAFETY: Open process token for the current process with TOKEN_QUERY; wrap handle with RAII.
        unsafe {
            let mut raw = HANDLE::default();
            // SAFETY: We pass a valid process handle from GetCurrentProcess and request TOKEN_QUERY.
            // On success, `raw` receives a live HANDLE which we immediately wrap in RAII to ensure
            // it is closed exactly once on all paths.
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY.0, &mut raw) == 0 {
                return Err(AcError::Win32("OpenProcessToken failed".into()));
            }
            // SAFETY: `raw` is a live, uniquely-owned HANDLE from OpenProcessToken; wrap it.
            let token_handle = crate::ffi::handles::Handle::from_raw(raw.0 as *mut _)
                .map_err(|_| AcError::Win32("invalid token handle".into()))?;
            let token = token_handle.as_win32();

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
    // SAFETY: `token` is a live HANDLE, and we pass a valid output buffer for a u32 value.
    // Size matches the buffer and retlen is writable.
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
    // SAFETY: Size probe with null buffer; API fills `needed` with required size.
    let size_probe =
        unsafe { GetTokenInformation(token, TokenAppContainerSid, None, 0, &mut needed) };
    if let Err(err) = size_probe {
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
    // SAFETY: Buffer is allocated with `needed` bytes; API writes into it and updates retlen.
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
    // SAFETY: Buffer holds a TOKEN_APPCONTAINER_INFORMATION per API contract; read the SID field.
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
    // SAFETY: Size probe to retrieve required bytes for TOKEN_GROUPS.
    let cap_probe = unsafe { GetTokenInformation(token, TokenCapabilities, None, 0, &mut needed) };
    if let Err(err) = cap_probe {
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
    // SAFETY: Buffer is large enough for TOKEN_GROUPS; API writes the groups and size back.
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
    // SAFETY: Read the group count from the TOKEN_GROUPS header.
    let count = unsafe { (*groups).GroupCount as usize };
    let mut out = Vec::with_capacity(count);
    if count == 0 {
        return Ok(out);
    }
    // SAFETY: TOKEN_GROUPS header indicates `count`; Groups points to an array of that length.
    let slice = unsafe { std::slice::from_raw_parts((*groups).Groups.as_ptr(), count) };
    for entry in slice {
        if entry.Sid.0.is_null() {
            continue;
        }
        // SAFETY: Convert a valid SID to SDDL via helper; returns owned String.
        // SAFETY: Convert a valid SID to SDDL via helper; returns owned String.
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
    // SAFETY: `psid` is a valid SID from the token. API returns a LocalAlloc-managed PWSTR.
    unsafe { ConvertSidToStringSidW(psid, &mut out) }
        .map_err(|e| AcError::Win32(format!("ConvertSidToStringSidW failed: {}", e)))?;
    // SAFETY: `out` now points to a LocalAlloc buffer; wrap to free exactly once.
    let guard = unsafe { LocalAllocGuard::<u16>::from_raw(out.0) };
    // SAFETY: Guarded pointer references a NUL-terminated UTF-16 string.
    Ok(unsafe { guard.to_string_lossy() })
}

#[cfg(windows)]
fn is_win32_error(err: &windows::core::Error, code: u32) -> bool {
    err.code() == HRESULT::from_win32(code)
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::PSID;

    #[test]
    fn sid_to_string_rejects_null() {
        // SAFETY: Passing a null SID pointer validates the conversion helper's error-path behavior.
        let err = unsafe { sid_to_string(PSID(std::ptr::null_mut())) }.unwrap_err();
        assert!(err.to_string().contains("null SID"));
    }

    #[test]
    fn query_bool_errors_on_invalid_handle() {
        let handle = HANDLE::default();
        // SAFETY: A null/zero handle is intentionally invalid and should trigger the error branch.
        let err = unsafe { query_bool(handle, windows::Win32::Security::TokenIsAppContainer) }
            .expect_err("invalid handle should fail");
        assert!(err.to_string().contains("GetTokenInformation"));
    }

    #[test]
    fn query_appcontainer_sid_errors_on_invalid_handle() {
        let handle = HANDLE::default();
        // SAFETY: A null/zero handle is intentionally invalid and should trigger the error branch.
        let err =
            unsafe { query_appcontainer_sid(handle) }.expect_err("invalid handle should fail");
        assert!(err.to_string().contains("GetTokenInformation"));
    }

    #[test]
    fn query_capabilities_errors_on_invalid_handle() {
        let handle = HANDLE::default();
        // SAFETY: A null/zero handle is intentionally invalid and should trigger the error branch.
        let err = unsafe { query_capabilities(handle) }.expect_err("invalid handle should fail");
        assert!(err.to_string().contains("GetTokenInformation"));
    }

    #[test]
    fn is_win32_error_matches() {
        let expected = windows::core::HRESULT::from_win32(1234);
        let err = windows::core::Error::from_hresult(expected);
        assert!(is_win32_error(&err, 1234));
        assert!(!is_win32_error(&err, 4321));
    }
}
