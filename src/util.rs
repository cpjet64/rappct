//! Internal helpers (UTF-16 conversions, handle ownership, freeing helpers).

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::Security::{FreeSid, PSID};
#[cfg(windows)]
use windows::Win32::System::Memory::LocalFree;

#[cfg(windows)]
pub fn to_utf16(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(not(windows))]
pub fn to_utf16(_s: &str) -> Vec<u16> {
    Vec::new()
}

#[cfg(windows)]
#[derive(Debug)]
pub struct OwnedHandle(pub(crate) windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
impl OwnedHandle {
    pub fn as_raw(&self) -> windows::Win32::Foundation::HANDLE {
        self.0
    }
    pub unsafe fn from_raw(h: windows::Win32::Foundation::HANDLE) -> Self {
        Self(h)
    }
    pub fn into_file(self) -> std::fs::File {
        use std::os::windows::io::FromRawHandle;
        let h = self.0;
        std::mem::forget(self);
        unsafe { std::fs::File::from_raw_handle(h.0 as *mut _) }
    }
}

#[cfg(windows)]
#[derive(Debug)]
pub struct LocalFreeGuard<T> {
    ptr: *mut T,
}

#[cfg(windows)]
impl<T> LocalFreeGuard<T> {
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn into_raw(mut self) -> *mut T {
        let ptr = self.ptr;
        self.ptr = std::ptr::null_mut();
        ptr
    }
}

#[cfg(windows)]
impl<T> Drop for LocalFreeGuard<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = LocalFree(windows::Win32::Foundation::HLOCAL(self.ptr as isize));
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

#[cfg(windows)]
impl LocalFreeGuard<u16> {
    pub unsafe fn to_string_lossy(&self) -> String {
        if self.ptr.is_null() {
            return String::new();
        }
        let mut len = 0usize;
        while *self.ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(self.ptr, len);
        String::from_utf16_lossy(slice)
    }

    pub fn as_pwstr(&self) -> PWSTR {
        PWSTR(self.ptr)
    }
}

#[cfg(windows)]
#[derive(Debug)]
pub struct FreeSidGuard {
    psid: PSID,
}

#[cfg(windows)]
impl FreeSidGuard {
    pub unsafe fn new(psid: PSID) -> Self {
        Self { psid }
    }

    pub fn as_psid(&self) -> PSID {
        self.psid
    }

    pub fn into_raw(mut self) -> PSID {
        let psid = self.psid;
        self.psid = PSID::default();
        psid
    }
}

#[cfg(windows)]
impl Drop for FreeSidGuard {
    fn drop(&mut self) {
        if !self.psid.0.is_null() {
            unsafe {
                let _ = FreeSid(self.psid);
            }
            self.psid = PSID::default();
        }
    }
}
