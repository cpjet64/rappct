//! Shared utility helpers for platform interop.

#[cfg(windows)]
pub mod win {
    use std::os::windows::ffi::OsStrExt;

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
pub use win::{to_utf16, to_utf16_os};

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use std::ffi::OsStr;

    #[test]
    fn to_utf16_appends_nul_terminator() {
        let wide = super::to_utf16("abc");
        assert_eq!(wide, vec![97, 98, 99, 0]);
        let empty = super::to_utf16("");
        assert_eq!(empty, vec![0]);
    }

    #[test]
    fn to_utf16_os_appends_nul_terminator() {
        let wide = super::to_utf16_os(OsStr::new("abc"));
        assert_eq!(wide, vec![97, 98, 99, 0]);
    }
}
