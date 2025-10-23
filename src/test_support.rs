//! Test-only helpers that wrap internal FFI types for integration coverage.

use crate::Result;
use crate::capability::CapabilityName;
use crate::ffi::attr_list::AttrList;
use crate::ffi::sec_caps::OwnedSecurityCapabilities;
use crate::ffi::sid::OwnedSid;
use crate::sid::AppContainerSid;
use windows::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;

/// Owned AppContainer SID for integration tests.
pub struct AppSid(OwnedSid);

impl AppSid {
    /// Convert an AppContainer SID string into an owned SID.
    pub fn from_app_container(sid: &AppContainerSid) -> Result<Self> {
        OwnedSid::from_sddl(sid.as_string()).map(Self)
    }

    /// Convert an SDDL string into an owned SID.
    pub fn from_sddl(sddl: &str) -> Result<Self> {
        OwnedSid::from_sddl(sddl).map(Self)
    }

    fn into_inner(self) -> OwnedSid {
        self.0
    }
}

/// Wrapper around `OwnedSecurityCapabilities` for smoke tests.
pub struct CatalogCaps(OwnedSecurityCapabilities);

impl CatalogCaps {
    /// Build the catalog-backed security capabilities set.
    pub fn from_catalog(app_sid: AppSid, caps: &[CapabilityName]) -> Result<Self> {
        OwnedSecurityCapabilities::from_catalog(app_sid.into_inner(), caps).map(Self)
    }

    fn inner(&self) -> &OwnedSecurityCapabilities {
        &self.0
    }
}

/// AttrList wrapper that exposes only the minimal surface needed by tests.
pub struct AttributeList(AttrList);

impl AttributeList {
    pub fn with_capacity(count: u32) -> Result<Self> {
        AttrList::with_capacity(count).map(Self)
    }

    pub fn set_security_capabilities(&mut self, caps: &CatalogCaps) -> Result<()> {
        self.0.set_security_capabilities(caps.inner())
    }

    pub fn as_mut_ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.0.as_mut_ptr()
    }
}
