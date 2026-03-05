#![allow(dead_code)]

//! Test-only helpers for Win32 pointers that must be released with `LocalFree`.

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_ABANDONED, WAIT_FAILED, WAIT_OBJECT_0};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    CreateMutexW, INFINITE, ReleaseMutex, WaitForSingleObject,
};
#[cfg(windows)]
use windows::core::PCWSTR;

#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(windows)]
const JOB_TEST_MUTEX_NAME: &str = "Local\\rappct.tests.job-launch";

#[cfg(windows)]
pub(crate) struct NamedMutexGuard(HANDLE);

#[cfg(windows)]
impl NamedMutexGuard {
    pub(crate) fn acquire(name: &str) -> Self {
        let wide_name: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
        let handle = unsafe { CreateMutexW(None, false, PCWSTR(wide_name.as_ptr())) }
            .expect("CreateMutexW test lock");
        let wait = unsafe { WaitForSingleObject(handle, INFINITE) };
        if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
            let _ = unsafe { CloseHandle(handle) };
            if wait == WAIT_FAILED {
                panic!("WaitForSingleObject test lock failed");
            }
            panic!("unexpected WaitForSingleObject test lock result: {wait:?}");
        }
        Self(handle)
    }
}

#[cfg(windows)]
impl Drop for NamedMutexGuard {
    fn drop(&mut self) {
        let _ = unsafe { ReleaseMutex(self.0) };
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(windows)]
pub(crate) fn acquire_job_test_lock() -> NamedMutexGuard {
    NamedMutexGuard::acquire(JOB_TEST_MUTEX_NAME)
}

#[cfg(windows)]
pub(crate) struct LocalAlloc<T>(*mut T);

#[cfg(windows)]
impl<T> LocalAlloc<T> {
    /// # Safety
    ///
    /// Caller must pass a pointer with `LocalAlloc`/`LocalFree` ownership rules.
    pub(crate) unsafe fn from_raw(ptr: *mut T) -> Self {
        Self(ptr)
    }

    /// # Safety
    ///
    /// Returns a caller-managed slice reference backed by this allocation.
    pub(crate) unsafe fn as_slice(&self, len: usize) -> &[T] {
        if self.0.is_null() || len == 0 {
            return &[];
        }
        // SAFETY: Caller guarantees `self.0` is valid for at least `len` elements.
        unsafe { std::slice::from_raw_parts(self.0, len) }
    }

    pub(crate) fn as_ptr(&self) -> *mut T {
        self.0
    }
}

#[cfg(windows)]
impl<T> Drop for LocalAlloc<T> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Pointer ownership comes from LocalAlloc-compatible APIs.
            unsafe {
                let _ = LocalFree(self.0 as isize);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}

#[cfg(windows)]
pub(crate) struct LocalWideString(*mut u16);

#[cfg(windows)]
impl LocalWideString {
    /// # Safety
    ///
    /// `raw` must be a pointer returned from a Win32 API that requires `LocalFree`.
    pub(crate) unsafe fn from_raw(raw: windows::core::PWSTR) -> Self {
        Self(raw.0)
    }

    pub(crate) fn to_string_lossy(&self) -> String {
        if self.0.is_null() {
            return String::new();
        }
        // SAFETY: `self.0` points to a valid NUL-terminated UTF-16 buffer by contract.
        unsafe {
            let mut len = 0usize;
            while *self.0.add(len) != 0 {
                len += 1;
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(self.0, len))
        }
    }
}

#[cfg(windows)]
impl Drop for LocalWideString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Pointer ownership comes from LocalAlloc-compatible APIs.
            unsafe {
                let _ = LocalFree(self.0 as isize);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}
