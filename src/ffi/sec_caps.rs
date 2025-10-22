//! Owned SECURITY_CAPABILITIES with owned SIDs and SID_AND_ATTRIBUTES.

use crate::ffi::sid::OwnedSid;
use windows::Win32::Security::{SECURITY_CAPABILITIES, SID_AND_ATTRIBUTES};

// Not always available in windows crate: documented value for SE_GROUP_ENABLED
const SE_GROUP_ENABLED_CONST: u32 = 0x0000_0004;

#[derive(Debug)]
pub(crate) struct OwnedSecurityCapabilities {
    _appcontainer_sid: OwnedSid,
    // Keep capability SIDs alive for the lifetime of SECURITY_CAPABILITIES
    _cap_sids: Vec<OwnedSid>,
    _caps: Box<[SID_AND_ATTRIBUTES]>,
    sc: SECURITY_CAPABILITIES,
}

impl OwnedSecurityCapabilities {
    pub(crate) fn new(app_sid: OwnedSid, caps_in: impl IntoIterator<Item = OwnedSid>) -> Self {
        let cap_sids: Vec<OwnedSid> = caps_in.into_iter().collect();
        let caps_vec: Vec<SID_AND_ATTRIBUTES> = cap_sids
            .iter()
            .map(|sid| SID_AND_ATTRIBUTES {
                Sid: sid.as_psid(),
                Attributes: SE_GROUP_ENABLED_CONST,
            })
            .collect();
        let caps = caps_vec.into_boxed_slice();
        let sc = SECURITY_CAPABILITIES {
            AppContainerSid: app_sid.as_psid(),
            Capabilities: caps.as_ptr() as *mut _,
            CapabilityCount: caps.len() as u32,
            Reserved: 0,
        };
        Self { _appcontainer_sid: app_sid, _cap_sids: cap_sids, _caps: caps, sc }
    }

    pub(crate) fn as_ptr(&self) -> *const SECURITY_CAPABILITIES {
        &self.sc as *const _
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use windows::Win32::Security::ConvertStringSidToSidW;
    use windows::core::PCWSTR;

    #[test]
    fn build_security_capabilities() {
        unsafe {
            let s_app = crate::ffi::wstr::WideString::from_str("S-1-5-32-544");
            let mut app_sid = windows::Win32::Security::PSID::default();
            ConvertStringSidToSidW(PCWSTR(s_app.as_pcwstr().0), &mut app_sid).unwrap();
            let app_owned = OwnedSid::from_localfree_psid(app_sid.0);

            let s_cap = crate::ffi::wstr::WideString::from_str("S-1-15-3-1024-0-0-0-0");
            let mut cap_sid = windows::Win32::Security::PSID::default();
            ConvertStringSidToSidW(PCWSTR(s_cap.as_pcwstr().0), &mut cap_sid).unwrap();
            let cap_owned = OwnedSid::from_localfree_psid(cap_sid.0);

            let sc = OwnedSecurityCapabilities::new(app_owned, [cap_owned]);
            let _p = sc.as_ptr();
        }
    }
}
