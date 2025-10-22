//! RAII guards for Windows memory allocators (LocalFree, CoTaskMemFree).

#![allow(clippy::undocumented_unsafe_blocks)]

use core::ffi::c_void;

/// Guard for memory that must be released with LocalFree.
#[derive(Debug)]
pub(crate) struct LocalAllocGuard<T> {
    ptr: *mut T,
}

impl<T> LocalAllocGuard<T> {
    /// # Safety
    /// `ptr` must be allocated by a Win32 API that requires LocalFree.
    pub(crate) unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { ptr }
    }
    pub(crate) fn as_ptr(&self) -> *mut T {
        self.ptr
    }
    #[allow(dead_code)]
    pub(crate) fn into_raw(mut self) -> *mut T {
        let p = self.ptr;
        self.ptr = core::ptr::null_mut();
        p
    }
}

impl<T> Drop for LocalAllocGuard<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // Pointer came from LocalAlloc-compatible API; ok to free once.
            local_free(self.ptr.cast::<c_void>());
            self.ptr = core::ptr::null_mut();
        }
    }
}

impl LocalAllocGuard<u16> {
    /// Convert a NUL-terminated wide string to Rust String.
    ///
    /// # Safety
    /// The pointer must reference a valid, NUL-terminated UTF-16 buffer.
    pub(crate) unsafe fn to_string_lossy(&self) -> String {
        if self.ptr.is_null() {
            return String::new();
        }
        let mut len = 0usize;
        // SAFETY: Walk until NUL; pointer validity per safety contract.
        while unsafe { *self.ptr.add(len) } != 0 {
            len += 1;
        }
        // SAFETY: Buffer is valid for at least `len` elements.
        let slice = unsafe { core::slice::from_raw_parts(self.ptr, len) };
        String::from_utf16_lossy(slice)
    }
}

/// Guard for memory allocated by COM task allocator (CoTaskMem*).
#[derive(Debug)]
pub(crate) struct CoTaskMem<T> {
    ptr: *mut T,
}

impl<T> CoTaskMem<T> {
    /// # Safety
    /// `ptr` must be allocated by CoTaskMem* API and is exclusively owned.
    pub(crate) unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { ptr }
    }
    pub(crate) fn as_ptr(&self) -> *mut T {
        self.ptr
    }
    #[allow(dead_code)]
    pub(crate) fn into_raw(mut self) -> *mut T {
        let p = self.ptr;
        self.ptr = core::ptr::null_mut();
        p
    }
}

impl<T> Drop for CoTaskMem<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: Pointer came from CoTaskMem* API; ok to free once.
            unsafe {
                windows::Win32::System::Com::CoTaskMemFree(Some(self.ptr.cast::<c_void>()));
            }
            self.ptr = core::ptr::null_mut();
        }
    }
}

// Minimal binding for LocalFree as the windows crate binding is not exposed.
#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(windows)]
fn local_free(ptr: *mut c_void) {
    // SAFETY: calling Win32 LocalFree on a pointer provided by a LocalAlloc-compatible API.
    unsafe {
        let _ = LocalFree(ptr as isize);
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::ConvertStringSidToSidW;
    use windows::core::PCWSTR;

    #[test]
    fn localalloc_guard_round_trip_string_sid() {
        unsafe {
            // SAFETY: Provide valid SDDL for well-known SID (Administrators).
            let sddl = super::super::wstr::WideString::from_str("S-1-5-32-544");
            let mut psid = windows::Win32::Security::PSID::default();
            ConvertStringSidToSidW(PCWSTR(sddl.as_pcwstr().0), &mut psid)
                .expect("ConvertStringSidToSidW");
            // Convert to string; API returns LocalAlloc-managed buffer.
            let mut out = windows::core::PWSTR::null();
            ConvertSidToStringSidW(psid, &mut out).expect("ConvertSidToStringSidW");
            let s = LocalAllocGuard::<u16> { ptr: out.0 }.to_string_lossy();
            assert!(s.starts_with("S-1-5-32-544"));
            // Free input SID (also LocalAlloc) using LocalAllocGuard.
            let _ = LocalAllocGuard::<core::ffi::c_void>::from_raw(psid.0);
        }
    }
}
