//! Owned SID wrapper with correct deallocator selection.

#![allow(clippy::undocumented_unsafe_blocks)]

use windows::Win32::Security::PSID;
use core::ffi::c_void;

#[derive(Debug, Clone, Copy)]
enum FreeKind {
    LocalFree,
    FreeSid,
}

/// Owned security identifier. Drops via the appropriate deallocator.
#[derive(Debug)]
pub(crate) struct OwnedSid {
    raw: *mut core::ffi::c_void,
    kind: FreeKind,
}

impl OwnedSid {
    /// # Safety
    /// `sid` must be a valid PSID allocated by an API requiring `LocalFree`.
    pub(crate) unsafe fn from_localfree_psid(sid: *mut core::ffi::c_void) -> Self {
        Self { raw: sid, kind: FreeKind::LocalFree }
    }

    /// # Safety
    /// `sid` must be a valid PSID allocated by an API requiring `FreeSid`.
    pub(crate) unsafe fn from_freesid_psid(sid: *mut core::ffi::c_void) -> Self {
        Self { raw: sid, kind: FreeKind::FreeSid }
    }

    pub(crate) fn as_psid(&self) -> PSID {
        PSID(self.raw)
    }

    #[allow(dead_code)]
    pub(crate) fn is_valid(&self) -> bool {
        !self.raw.is_null()
    }
}

impl Drop for OwnedSid {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }
        unsafe {
            match self.kind {
                FreeKind::LocalFree => {
                    local_free(self.raw);
                }
                FreeKind::FreeSid => {
                    let _ = windows::Win32::Security::FreeSid(self.as_psid());
                }
            }
        }
        self.raw = core::ptr::null_mut();
    }
}

// Minimal binding for LocalFree for SID memory returned by ConvertStringSidToSidW.
#[cfg(windows)]
#[link(name = "Kernel32")]
extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(windows)]
unsafe fn local_free(ptr: *mut c_void) {
    let _ = LocalFree(ptr as isize);
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use windows::Win32::Security::ConvertStringSidToSidW;
    use windows::core::PCWSTR;

    #[test]
    fn owned_sid_localfree_constructs_and_drops() {
        unsafe {
            let mut psid = PSID::default();
            let sddl = crate::ffi::wstr::WideString::from_str("S-1-5-32-544");
            ConvertStringSidToSidW(PCWSTR(sddl.as_pcwstr().0), &mut psid).expect("ConvertStringSidToSidW");
            let sid = OwnedSid::from_localfree_psid(psid.0);
            assert!(sid.is_valid());
            let _ = sid.as_psid();
        }
    }
}
