//! Helpers for building environment blocks for `CreateProcessW`.

use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;

/// Owned UTF-16 environment block terminated with a double NUL.
#[derive(Debug, Default)]
pub(crate) struct WideBlock {
    buf: Vec<u16>,
}

impl WideBlock {
    pub(crate) fn new(buf: Vec<u16>) -> Self {
        Self { buf }
    }

    pub(crate) fn as_ptr(&self) -> *const u16 {
        self.buf.as_ptr()
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len()
    }
}

/// Build a Windows environment block from key-value pairs.
///
/// The block is sorted case-insensitively by key, each `key=value` pair is
/// NUL-terminated, and the entire block ends with an extra NUL (double-NUL).
pub(crate) fn make_wide_block(entries: &[(OsString, OsString)]) -> WideBlock {
    let mut pairs: Vec<&(OsString, OsString)> = entries.iter().collect();

    pairs.sort_by_cached_key(|(key, _)| key.to_string_lossy().to_ascii_lowercase());

    let mut buf: Vec<u16> = Vec::with_capacity(entries.len().saturating_mul(24));
    for (key, value) in pairs {
        buf.extend(key.encode_wide());
        buf.push(b'=' as u16);
        buf.extend(value.encode_wide());
        buf.push(0);
    }
    buf.push(0);

    WideBlock::new(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_block_sorts_and_terminates() {
        let block = make_wide_block(&[
            (OsString::from("PATH"), OsString::from(r"C:\\Windows")),
            (
                OsString::from("ComSpec"),
                OsString::from(r"C:\\Windows\\System32\\cmd.exe"),
            ),
        ]);

        assert!(block.buf.len() >= 6, "block too short");
        assert_eq!(block.buf[block.buf.len() - 1], 0);
        assert_eq!(block.buf[block.buf.len() - 2], 0);

        let utf16 = &block.buf[..block.buf.len() - 1];
        let strings: Vec<String> = utf16
            .split(|c| *c == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf16(s).unwrap())
            .collect();

        assert_eq!(strings[0], "ComSpec=C:\\\\Windows\\\\System32\\\\cmd.exe");
        assert_eq!(strings[1], "PATH=C:\\\\Windows");
    }

    #[test]
    fn wide_block_len_and_is_empty() {
        let empty = WideBlock::new(vec![]);
        assert_eq!(empty.buf.len(), 0);
        assert!(empty.buf.is_empty());
        let with_data = WideBlock::new(vec![b'A' as u16, 0]);
        assert_eq!(with_data.buf.len(), 2);
        assert!(!with_data.buf.is_empty());
    }
}
