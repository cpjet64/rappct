//! Internal FFI safety helpers and RAII wrappers.
//!
//! This module centralizes ownership and lifetime boundaries for Windows FFI
//! resources so call sites can stay safe and lightweight. See ADR-0001.
#![warn(clippy::undocumented_unsafe_blocks)]

#[cfg(windows)]
pub(crate) mod attr_list;
#[cfg(windows)]
pub(crate) mod handles;
#[cfg(windows)]
pub(crate) mod mem;
#[cfg(windows)]
pub(crate) mod sec_caps;
#[cfg(windows)]
pub(crate) mod sid;
#[cfg(windows)]
pub(crate) mod wstr;

// Non-Windows stubs so the crate type-checks cross-platform.
#[cfg(not(windows))]
pub(crate) mod handles {
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct Handle;
}
#[cfg(not(windows))]
pub(crate) mod mem {
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct LocalAllocGuard {/* no-op */}
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct CoTaskMem<T> {
        /* no-op */
        _phantom: core::marker::PhantomData<T>,
    }
}
#[cfg(not(windows))]
pub(crate) mod sid {
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct OwnedSid;
}
#[cfg(not(windows))]
pub(crate) mod wstr {
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct WideString;
}
#[cfg(not(windows))]
pub(crate) mod sec_caps {
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct OwnedSecurityCapabilities;
}
#[cfg(not(windows))]
pub(crate) mod attr_list {
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub(crate) struct AttrList;
}
