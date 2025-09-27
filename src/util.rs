//! Internal helpers (UTF-16 conversions, handle ownership, freeing helpers).

#[cfg(windows)]
pub fn to_utf16(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
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
