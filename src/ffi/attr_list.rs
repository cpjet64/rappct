//! Owned `PROC_THREAD_ATTRIBUTE_LIST` with helpers for security capabilities.

use crate::{AcError, Result};
use windows::Win32::System::Threading::{
    DeleteProcThreadAttributeList, InitializeProcThreadAttributeList, UpdateProcThreadAttribute,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
};
use windows::Win32::Foundation::HANDLE;
use core::ffi::c_void;

#[derive(Debug)]
pub(crate) struct AttrList {
    buf: Vec<u8>,
    ptr: LPPROC_THREAD_ATTRIBUTE_LIST,
}

impl AttrList {
    pub(crate) fn with_capacity(count: u32) -> Result<Self> {
        let mut bytes: usize = 0;
        unsafe {
            // SAFETY: Probe for size; passing None and flags=0 per API contract.
            let _ = InitializeProcThreadAttributeList(None, count, Some(0), &mut bytes as *mut usize);
        }
        let mut buf = vec![0u8; bytes];
        let ptr = LPPROC_THREAD_ATTRIBUTE_LIST(buf.as_mut_ptr() as _);
        unsafe {
            // SAFETY: Initialize with computed size.
            InitializeProcThreadAttributeList(Some(ptr), count, Some(0), &mut bytes as *mut usize)
                .map_err(|e| AcError::Win32(format!("InitializeProcThreadAttributeList: {}", e)))?;
        }
        Ok(Self { buf, ptr })
    }

    pub(crate) fn as_mut_ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.ptr
    }

    pub(crate) fn set_security_capabilities(
        &mut self,
        sc: &crate::ffi::sec_caps::OwnedSecurityCapabilities,
    ) -> Result<()> {
        let size = core::mem::size_of::<windows::Win32::Security::SECURITY_CAPABILITIES>();
        unsafe {
            // SAFETY: `sc` points to stable, owned memory; attribute list initialized.
            UpdateProcThreadAttribute(
                self.ptr,
                0,
                PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
                Some(sc.as_ptr() as *const _),
                size,
                None,
                None,
            )
            .map_err(|e| AcError::Win32(format!("UpdateProcThreadAttribute: {}", e)))
        }
    }

    /// Attach the All App Packages policy (LPAC opt-out) attribute.
    /// Caller must ensure `policy` outlives CreateProcessW.
    pub(crate) fn set_all_app_packages_policy(&mut self, policy: &u32) -> Result<()> {
        use windows::Win32::System::Threading::PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY;
        let size = core::mem::size_of::<u32>();
        unsafe {
            UpdateProcThreadAttribute(
                self.ptr,
                0,
                PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY as usize,
                Some(policy as *const u32 as *const c_void),
                size,
                None,
                None,
            )
            .map_err(|e| AcError::Win32(format!("UpdateProcThreadAttribute(AAPolicy): {}", e)))
        }
    }

    /// Attach a handle inheritance list.
    /// The slice must remain valid until CreateProcessW returns.
    pub(crate) fn set_handle_list(&mut self, handles: &[HANDLE]) -> Result<()> {
        use windows::Win32::System::Threading::PROC_THREAD_ATTRIBUTE_HANDLE_LIST;
        let bytes = core::mem::size_of::<HANDLE>() * handles.len();
        unsafe {
            UpdateProcThreadAttribute(
                self.ptr,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                Some(handles.as_ptr() as *const c_void),
                bytes,
                None,
                None,
            )
            .map_err(|e| AcError::Win32(format!("UpdateProcThreadAttribute(HandleList): {}", e)))
        }
    }
}

impl Drop for AttrList {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: Attribute list was initialized; ok to delete once.
            DeleteProcThreadAttributeList(self.ptr);
        }
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Security::ConvertStringSidToSidW;

    #[test]
    fn attr_list_init_and_set_sc() {
        unsafe {
            let s_app = crate::ffi::wstr::WideString::from_str("S-1-5-32-544");
            let mut app_sid = windows::Win32::Security::PSID::default();
            ConvertStringSidToSidW(PCWSTR(s_app.as_pcwstr().0), &mut app_sid).unwrap();
            let app_owned = crate::ffi::sid::OwnedSid::from_localfree_psid(app_sid.0);

            let s_cap = crate::ffi::wstr::WideString::from_str("S-1-15-3-1024-0-0-0-0");
            let mut cap_sid = windows::Win32::Security::PSID::default();
            ConvertStringSidToSidW(PCWSTR(s_cap.as_pcwstr().0), &mut cap_sid).unwrap();
            let cap_owned = crate::ffi::sid::OwnedSid::from_localfree_psid(cap_sid.0);

            let sc = crate::ffi::sec_caps::OwnedSecurityCapabilities::new(app_owned, [cap_owned]);
            let mut al = AttrList::with_capacity(1).unwrap();
            al.set_security_capabilities(&sc).unwrap();
        }
    }
}
