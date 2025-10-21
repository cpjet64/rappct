//! Shared utility helpers for platform interop.

#[cfg(windows)]
pub mod win {
    use std::os::windows::ffi::OsStrExt;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{FreeSid, PSID};

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn LocalFree(h: isize) -> isize;
    }

    /// Converts a Rust string into a null-terminated UTF-16 buffer.
    pub fn to_utf16(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Converts a platform string (OsStr) into a null-terminated UTF-16 buffer.
    pub fn to_utf16_os(s: &std::ffi::OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }

    /// Owned Win32 `HANDLE` that closes on drop.
    #[derive(Debug)]
    pub struct OwnedHandle(pub(crate) HANDLE);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    impl OwnedHandle {
        pub fn as_raw(&self) -> HANDLE {
            self.0
        }

        /// Constructs an `OwnedHandle` from a raw Win32 `HANDLE`.
        ///
        /// # Safety
        /// Caller must ensure `handle` is a valid, live HANDLE that is uniquely
        /// owned and must be closed exactly once. After calling this, the guard
        /// will close it on drop. Do not use the handle elsewhere after passing it here.
        pub unsafe fn from_raw(handle: HANDLE) -> Self {
            Self(handle)
        }

        pub fn into_file(self) -> std::fs::File {
            use std::os::windows::io::FromRawHandle;
            let handle = self.0;
            // Prevent Drop from closing the handle twice
            std::mem::forget(self);
            // SAFETY: transfer ownership of the HANDLE to a File
            unsafe { std::fs::File::from_raw_handle(handle.0 as *mut _) }
        }
    }

    /// RAII guard that frees allocations with `LocalFree` when dropped.
    #[derive(Debug)]
    pub struct LocalFreeGuard<T> {
        ptr: *mut T,
    }

    impl<T> LocalFreeGuard<T> {
        /// Creates a new guard from a raw pointer returned by Win32.
        ///
        /// # Safety
        /// The pointer must have been allocated by a Win32 API that requires
        /// `LocalFree` for release and must be valid for the lifetime of the guard.
        /// Do not pass stack or heap pointers that are not `LocalAlloc`-managed.
        pub unsafe fn new(ptr: *mut T) -> Self {
            Self { ptr }
        }

        pub fn as_ptr(&self) -> *mut T {
            self.ptr
        }

        pub fn is_null(&self) -> bool {
            self.ptr.is_null()
        }

        /// Releases ownership without freeing.
        pub fn into_raw(mut self) -> *mut T {
            let ptr = self.ptr;
            self.ptr = std::ptr::null_mut();
            ptr
        }
    }

    impl<T> Drop for LocalFreeGuard<T> {
        fn drop(&mut self) {
            if !self.ptr.is_null() {
                unsafe {
                    let _ = LocalFree(self.ptr as isize);
                }
                self.ptr = std::ptr::null_mut();
            }
        }
    }

    impl LocalFreeGuard<u16> {
        /// Converts the guarded wide string into UTF-8 (without trailing null).
        ///
        /// # Safety
        /// The guarded pointer must reference a valid, NUL-terminated UTF-16
        /// buffer allocated by a Win32 API compatible with `LocalFree`.
        #[allow(unsafe_op_in_unsafe_fn)]
        pub unsafe fn to_string_lossy(&self) -> String {
            if self.ptr.is_null() {
                return String::new();
            }
            let mut len = 0usize;
            while unsafe { *self.ptr.add(len) } != 0 {
                len += 1;
            }
            let slice = unsafe { std::slice::from_raw_parts(self.ptr, len) };
            String::from_utf16_lossy(slice)
        }

        pub fn as_pwstr(&self) -> PWSTR {
            PWSTR(self.ptr)
        }
    }

    /// RAII guard that releases a `PSID` via `FreeSid` on drop.
    #[derive(Debug)]
    pub struct FreeSidGuard {
        psid: PSID,
    }

    impl FreeSidGuard {
        /// Creates a guard that will call `FreeSid` on drop.
        ///
        /// # Safety
        /// `psid` must be a valid, heap-allocated SID returned by APIs that
        /// require `FreeSid` for release. Do not pass borrowed or invalid pointers.
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
}

#[cfg(not(windows))]
pub mod win {
    /// Non-Windows stub conversion; returns an empty buffer.
    pub fn to_utf16(_s: &str) -> Vec<u16> {
        Vec::new()
    }
    pub fn to_utf16_os(_s: &std::ffi::OsStr) -> Vec<u16> {
        Vec::new()
    }
}

#[cfg(not(windows))]
pub use win::to_utf16;
#[cfg(not(windows))]
pub use win::to_utf16_os;
#[cfg(windows)]
pub use win::{to_utf16, to_utf16_os, FreeSidGuard, LocalFreeGuard, OwnedHandle};
