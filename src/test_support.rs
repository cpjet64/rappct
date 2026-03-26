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

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use crate::capability::CapabilityName;
    use crate::sid::AppContainerSid;

    #[test]
    fn app_sid_converts_from_app_container() {
        let sid = AppContainerSid::from_sddl("S-1-15-2-1");
        let wrapped = AppSid::from_app_container(&sid).expect("from app container sid");
        let _ = wrapped.into_inner();
    }

    #[test]
    fn app_sid_converts_from_sddl() {
        let wrapped = AppSid::from_sddl("S-1-15-2-1").expect("from sddl");
        let _ = wrapped.into_inner();
    }

    #[test]
    fn app_sid_invalid_sddl_is_rejected() {
        assert!(AppSid::from_sddl("not-a-sid").is_err());
    }

    #[test]
    fn catalog_caps_from_known_names() {
        let sid = AppSid::from_sddl("S-1-15-2-1").expect("app sid");
        let wrapped = CatalogCaps::from_catalog(sid, &[CapabilityName::InternetClient])
            .expect("catalog caps");
        let _ = wrapped.inner();
    }

    #[test]
    fn attribute_list_roundtrips_security_caps() {
        let sid = AppSid::from_sddl("S-1-15-2-1").expect("app sid");
        let caps = CatalogCaps::from_catalog(sid, &[CapabilityName::InternetClient]).expect("caps");
        let mut list = AttributeList::with_capacity(1).expect("attribute list");
        list.set_security_capabilities(&caps).expect("set caps");
        assert!(!list.as_mut_ptr().0.is_null());
    }
}
