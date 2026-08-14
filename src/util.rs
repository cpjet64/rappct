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
    /// Converts a Rust string into portable, null-terminated UTF-16.
    pub fn to_utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Converts a platform string lossily into portable, null-terminated UTF-16.
    pub fn to_utf16_os(s: &std::ffi::OsStr) -> Vec<u16> {
        s.to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect()
    }
}

#[cfg(not(windows))]
pub use win::to_utf16;
#[cfg(not(windows))]
pub use win::to_utf16_os;
#[cfg(windows)]
pub use win::{to_utf16, to_utf16_os};

#[cfg(test)]
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

    #[test]
    fn to_utf16_preserves_non_ascii_code_units() {
        let value = "Straße 😀";
        let expected: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();

        assert_eq!(super::to_utf16(value), expected);
        assert_eq!(super::to_utf16_os(OsStr::new(value)), expected);
    }
}
