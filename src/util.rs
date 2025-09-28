//! Shared utility helpers for platform interop.

#[cfg(windows)]
pub mod win {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL};
    use windows::Win32::Security::{FreeSid, PSID};
    use windows::Win32::System::Memory::LocalFree;

    /// Converts a Rust string into a null-terminated UTF-16 buffer.
    pub fn to_utf16(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
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

        pub unsafe fn from_raw(handle: HANDLE) -> Self {
            Self(handle)
        }

        pub fn into_file(self) -> std::fs::File {
            use std::os::windows::io::FromRawHandle;
            let handle = self.0;
            std::mem::forget(self);
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
                    let _ = LocalFree(HLOCAL(self.ptr as isize));
                }
                self.ptr = std::ptr::null_mut();
            }
        }
    }

    impl LocalFreeGuard<u16> {
        /// Converts the guarded wide string into UTF-8 (without trailing null).
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

    /// RAII guard that releases a `PSID` via `FreeSid` on drop.
    #[derive(Debug)]
    pub struct FreeSidGuard {
        psid: PSID,
    }

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
}

#[cfg(not(windows))]
pub use win::to_utf16;
#[cfg(windows)]
pub use win::{to_utf16, FreeSidGuard, LocalFreeGuard, OwnedHandle};
