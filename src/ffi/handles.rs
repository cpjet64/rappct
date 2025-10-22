//! RAII wrapper for Win32 `HANDLE` using the standard library's `OwnedHandle`.

#![allow(clippy::undocumented_unsafe_blocks)]

use crate::{AcError, Result};
use std::os::windows::io::{
    AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, IntoRawHandle, OwnedHandle, RawHandle,
};
use windows::Win32::Foundation::HANDLE;

/// Owned Win32 handle that closes exactly once on drop.
#[derive(Debug)]
pub(crate) struct Handle(OwnedHandle);

impl Handle {
    /// Construct from a raw `HANDLE` pointer.
    ///
    /// # Safety
    /// `h` must be a valid, live Win32 handle that is uniquely owned by the caller.
    /// After calling this, the `Handle` takes responsibility to close it exactly once.
    pub(crate) unsafe fn from_raw(h: RawHandle) -> Result<Self> {
        // SAFETY: Caller guarantees a valid, uniquely owned handle.
        if h.is_null() {
            return Err(AcError::Win32("invalid null handle".into()));
        }
        // INVALID_HANDLE_VALUE is -1 casted; guard against it when possible.
        if h as isize == -1 {
            return Err(AcError::Win32("invalid handle value".into()));
        }
        let owned = unsafe { OwnedHandle::from_raw_handle(h) };
        Ok(Self(owned))
    }

    pub(crate) fn as_borrowed(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }

    pub(crate) fn into_owned(self) -> OwnedHandle {
        #[allow(dead_code)]
        self.0
    }

    pub(crate) fn as_win32(&self) -> HANDLE {
        HANDLE(self.as_borrowed().as_raw_handle())
    }

    pub(crate) fn into_file(self) -> std::fs::File {
        // SAFETY: We take ownership of the raw handle and transfer to File
        let raw = self.0.into_raw_handle();
        unsafe { std::fs::File::from_raw_handle(raw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null_mut;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

    #[test]
    fn handle_wraps_event_and_closes() {
        unsafe {
            // SAFETY: CreateEventW returns a live handle on success.
            let raw = CreateEventW(None, true.into(), false.into(), None)
                .expect("create event")
                .0 as *mut _;
            let h = Handle::from_raw(raw).expect("wrap handle");
            // Use the handle before drop.
            let hr = SetEvent(HANDLE(h.as_borrowed().as_raw_handle() as isize));
            assert!(hr.as_bool());
            let wr = WaitForSingleObject(HANDLE(h.as_borrowed().as_raw_handle() as isize), 1000);
            assert_eq!(wr, WAIT_OBJECT_0);
            // Drop closes exactly once; any extra close would be UB but OwnedHandle prevents it.
            let _ = h;
        }
    }
}
