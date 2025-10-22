//! Owned wide (UTF-16) string helpers for FFI.

use windows::core::{PCWSTR, PWSTR};

/// NUL-terminated UTF-16 buffer for stable FFI pointers.
#[derive(Debug, Clone)]
pub(crate) struct WideString {
    buf: Vec<u16>,
}
impl WideString {
    pub(crate) fn from_os_str(s: &std::ffi::OsStr) -> Self {
        use std::os::windows::ffi::OsStrExt;
        let mut v: Vec<u16> = s.encode_wide().collect();
        v.push(0);
        Self { buf: v }
    }

    pub(crate) fn from_str(s: &str) -> Self {
        use std::os::windows::ffi::OsStrExt;
        let mut v: Vec<u16> = std::ffi::OsStr::new(s).encode_wide().collect();
        v.push(0);
        Self { buf: v }
    }

    pub(crate) fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.buf.as_ptr())
    }

    #[allow(dead_code)]
    pub(crate) fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.buf.as_mut_ptr())
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[test]
    fn wide_from_str_is_nul_terminated() {
        let w = WideString::from_str("abc");
        assert_eq!(w.buf.last().copied(), Some(0));
        assert_eq!(&w.buf[..w.buf.len() - 1], &[97u16, 98, 99]);
        let _p = w.as_pcwstr();
    }
}
