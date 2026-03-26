//! Helpers for building environment blocks for `CreateProcessW`.

use crate::{AcError, Result};
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
pub(crate) fn make_wide_block(entries: &[(OsString, OsString)]) -> Result<WideBlock> {
    let mut pairs: Vec<(OsString, OsString)> = Vec::with_capacity(entries.len());
    for (key, value) in entries {
        if key.is_empty() {
            return Err(AcError::Win32(
                "invalid environment key: empty key is not allowed".into(),
            ));
        }
        if key.encode_wide().any(|ch| ch == 0) {
            return Err(AcError::Win32(
                "invalid environment key: embedded NUL is not allowed".into(),
            ));
        }
        if value.encode_wide().any(|ch| ch == 0) {
            return Err(AcError::Win32(
                "invalid environment value: embedded NUL is not allowed".into(),
            ));
        }
        if let Some((existing_key, existing_value)) = pairs.iter_mut().find(|(existing_key, _)| {
            existing_key
                .to_string_lossy()
                .eq_ignore_ascii_case(&key.to_string_lossy())
        }) {
            *existing_key = key.clone();
            *existing_value = value.clone();
            continue;
        }
        pairs.push((key.clone(), value.clone()));
    }

    pairs.sort_by_cached_key(|(key, _)| key.to_string_lossy().to_ascii_lowercase());

    let mut buf: Vec<u16> = Vec::with_capacity(entries.len().saturating_mul(24));
    for (key, value) in pairs {
        buf.extend(key.encode_wide());
        buf.push(b'=' as u16);
        buf.extend(value.encode_wide());
        buf.push(0);
    }
    if buf.is_empty() {
        // Windows environment blocks must be double-NUL terminated even when empty.
        buf.push(0);
    }
    buf.push(0);

    Ok(WideBlock::new(buf))
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
        ])
        .expect("build env block");

        assert!(block.buf.len() >= 6, "block too short");
        assert_eq!(block.buf[block.buf.len() - 1], 0);
        assert_eq!(block.buf[block.buf.len() - 2], 0);

        let utf16 = &block.buf[..block.buf.len() - 1];
        let strings: Vec<String> = utf16
            .split(|c| *c == 0)
            .filter(|s| !s.is_empty())
            .map(String::from_utf16_lossy)
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

    #[test]
    fn wide_block_exposes_pointer_and_len() {
        let data = vec![b'K' as u16, b'=' as u16, b'V' as u16, 0, 0];
        let block = WideBlock::new(data.clone());
        assert_eq!(block.len(), data.len());
        assert!(!block.as_ptr().is_null());
    }

    #[test]
    fn make_block_empty_entries_still_double_terminated() {
        let block = make_wide_block(&[]).expect("build empty env block");
        assert_eq!(block.len(), 2);
        assert_eq!(block.buf, vec![0, 0]);
    }

    #[test]
    fn make_block_deduplicates_case_insensitive_keys() {
        let block = make_wide_block(&[
            (OsString::from("PATH"), OsString::from(r"C:\\first")),
            (OsString::from("path"), OsString::from(r"C:\\second")),
        ])
        .expect("build dedup env block");

        let utf16 = &block.buf[..block.buf.len() - 1];
        let strings: Vec<String> = utf16
            .split(|c| *c == 0)
            .filter(|s| !s.is_empty())
            .map(String::from_utf16_lossy)
            .collect();
        assert_eq!(strings, vec![r"path=C:\\second"]);
    }

    #[test]
    fn make_block_rejects_empty_key() {
        let err = make_wide_block(&[(OsString::from(""), OsString::from("value"))]).unwrap_err();
        assert!(err.to_string().contains("empty key"));
    }
}
