//! RAII wrapper for Win32 `HANDLE` using the standard library's `OwnedHandle`.

#![allow(clippy::undocumented_unsafe_blocks)]

use crate::{AcError, Result};
use std::os::windows::io::{
    AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, IntoRawHandle, OwnedHandle, RawHandle,
};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::GetCurrentProcess;

#[link(name = "Kernel32")]
unsafe extern "system" {
    fn DuplicateHandle(
        h_source_process: HANDLE,
        h_source: HANDLE,
        h_target_process: HANDLE,
        lp_target: *mut HANDLE,
        dw_desired_access: u32,
        b_inherit: i32,
        dw_options: u32,
    ) -> i32;
}

const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;

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

    #[allow(dead_code)]
    pub(crate) fn into_owned(self) -> OwnedHandle {
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

    pub(crate) fn as_raw(&self) -> RawHandle {
        self.as_win32().0 as _
    }
}

pub(crate) fn duplicate_handle(handle: BorrowedHandle<'_>, inherit: bool) -> Result<Handle> {
    let mut duplicated = HANDLE::default();
    // SAFETY: DuplicateHandle expects live handles and returns BOOL. Current process handles
    // remain valid for the duration of the call.
    let ok = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            HANDLE(handle.as_raw_handle()),
            GetCurrentProcess(),
            &mut duplicated,
            0,
            inherit as i32,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if ok == 0 {
        return Err(AcError::Win32("DuplicateHandle failed".into()));
    }
    // SAFETY: DuplicateHandle returns a uniquely owned handle on success.
    unsafe { Handle::from_raw(duplicated.0 as *mut _) }
}

pub(crate) fn duplicate_from_raw(handle: RawHandle, inherit: bool) -> Result<Handle> {
    // SAFETY: Caller guarantees `handle` refers to a valid, live handle.
    let borrowed = unsafe { BorrowedHandle::borrow_raw(handle) };
    duplicate_handle(borrowed, inherit)
}

pub(crate) fn from_win32(handle: HANDLE) -> Result<Handle> {
    // SAFETY: `handle` originates from Win32 and is assumed to be uniquely owned at call site.
    unsafe { Handle::from_raw(handle.0 as *mut _) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::WAIT_OBJECT_0;
    use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

    #[test]
    fn handle_wraps_event_and_closes() {
        unsafe {
            // SAFETY: CreateEventW returns a live handle on success.
            let raw = CreateEventW(None, true, false, None)
                .expect("create event")
                .0 as *mut _;
            let h = Handle::from_raw(raw).expect("wrap handle");
            // Use the handle before drop.
            let hr = SetEvent(h.as_win32());
            assert!(hr.is_ok());
            let wr = WaitForSingleObject(h.as_win32(), 1000);
            assert_eq!(wr, WAIT_OBJECT_0);
            // Drop closes exactly once; any extra close would be UB but OwnedHandle prevents it.
            let _ = h;
        }
    }
}
