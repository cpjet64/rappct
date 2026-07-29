use std::collections::HashSet;

use crate::ffi::mem::LocalAllocGuard;
use crate::ffi::sid::OwnedSid;
use crate::sid::AppContainerSid;
use crate::{AcError, Result};
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INET_FIREWALL_APP_CONTAINER, NETISO_FLAG_FORCE_COMPUTE_BINARIES,
    NetworkIsolationEnumAppContainers, NetworkIsolationFreeAppContainers,
    NetworkIsolationGetAppContainerConfig, NetworkIsolationSetAppContainerConfig,
};
use windows::Win32::Security::Authorization::{ConvertSidToStringSidW, ConvertStringSidToSidW};
use windows::Win32::Security::{PSID, SID_AND_ATTRIBUTES};
use windows::core::{PCWSTR, PWSTR};

pub(super) fn list_appcontainers() -> Result<Vec<(AppContainerSid, String)>> {
    let enumeration = AppContainerEnumeration::load()?;
    let listed = collect_appcontainer_entries(enumeration.as_slice()?)?;
    validate_firewall_config(&listed.sid_set)?;
    Ok(listed.entries)
}

pub(super) fn set_loopback(allow: bool, sid: &AppContainerSid) -> Result<()> {
    let mut config = LoopbackConfig::load()?;
    let target = LoopbackSid::from_app_container(sid)?;
    update_loopback_entries(&mut config.entries, allow, &target)?;
    apply_loopback_config(&config.entries)
}

struct AppContainerEnumeration {
    ptr: *mut INET_FIREWALL_APP_CONTAINER,
    count: u32,
}

impl AppContainerEnumeration {
    fn load() -> Result<Self> {
        let mut count = 0;
        let mut ptr = std::ptr::null_mut();
        // SAFETY: The API writes an array pointer/count pair that is guarded by Drop.
        let err = unsafe {
            NetworkIsolationEnumAppContainers(
                NETISO_FLAG_FORCE_COMPUTE_BINARIES.0 as u32,
                &mut count,
                &mut ptr,
            )
        };
        if err != 0 {
            return Err(AcError::Win32(format!(
                "NetworkIsolationEnumAppContainers failed: {err}"
            )));
        }
        Ok(Self { ptr, count })
    }

    fn as_slice(&self) -> Result<&[INET_FIREWALL_APP_CONTAINER]> {
        if self.ptr.is_null() {
            return null_array_result(
                self.count,
                "NetworkIsolationEnumAppContainers returned null array with non-zero count",
            );
        }
        // SAFETY: The pointer/count pair comes from NetworkIsolationEnumAppContainers.
        Ok(unsafe { std::slice::from_raw_parts(self.ptr, self.count as usize) })
    }
}

impl Drop for AppContainerEnumeration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: Buffer is allocated by NetworkIsolationEnumAppContainers.
            unsafe { NetworkIsolationFreeAppContainers(self.ptr) };
            self.ptr = std::ptr::null_mut();
        }
    }
}

fn null_array_result<T>(count: u32, message: &'static str) -> Result<&'static [T]> {
    if count == 0 {
        Ok(&[])
    } else {
        Err(AcError::Win32(message.to_string()))
    }
}

struct ListedAppContainers {
    entries: Vec<(AppContainerSid, String)>,
    sid_set: HashSet<String>,
}

fn collect_appcontainer_entries(
    slice: &[INET_FIREWALL_APP_CONTAINER],
) -> Result<ListedAppContainers> {
    let mut entries = Vec::with_capacity(slice.len());
    let mut sid_set = HashSet::with_capacity(slice.len());
    for item in slice {
        let sid_str = psid_to_string(PSID(item.appContainerSid as *mut _))?;
        sid_set.insert(sid_str.clone());
        entries.push((
            AppContainerSid::from_sddl(sid_str),
            pwstr_to_string(item.displayName),
        ));
    }
    Ok(ListedAppContainers { entries, sid_set })
}

fn validate_firewall_config(sid_set: &HashSet<String>) -> Result<()> {
    let config = LoopbackConfig::load()?;
    for sa in &config.entries {
        let sid_str = psid_to_string(sa.Sid)?;
        if !sid_set.contains(&sid_str) {
            trace_missing_config_sid(&sid_str);
        }
    }
    Ok(())
}

#[cfg(feature = "tracing")]
fn trace_missing_config_sid(sid_str: &str) {
    tracing::warn!(
        "Firewall config SID missing from enumeration; continuing: {}",
        sid_str
    );
}

#[cfg(not(feature = "tracing"))]
fn trace_missing_config_sid(_sid_str: &str) {}

struct LoopbackConfig {
    _guard: Option<LocalAllocGuard<SID_AND_ATTRIBUTES>>,
    entries: Vec<SID_AND_ATTRIBUTES>,
}

impl LoopbackConfig {
    fn load() -> Result<Self> {
        let mut count = 0;
        let mut ptr = std::ptr::null_mut();
        // SAFETY: Retrieves a LocalAlloc array pointer/count pair.
        let err = unsafe { NetworkIsolationGetAppContainerConfig(&mut count, &mut ptr) };
        if err != 0 {
            return Err(AcError::Win32(format!(
                "NetworkIsolationGetAppContainerConfig failed: {err}"
            )));
        }
        let guard = config_guard(ptr, count)?;
        let entries = config_entries(&guard, count);
        Ok(Self {
            _guard: guard,
            entries,
        })
    }
}

fn config_guard(
    ptr: *mut SID_AND_ATTRIBUTES,
    count: u32,
) -> Result<Option<LocalAllocGuard<SID_AND_ATTRIBUTES>>> {
    if ptr.is_null() {
        return null_config_result(count);
    }
    // SAFETY: The config API returns a LocalAlloc-managed SID_AND_ATTRIBUTES array.
    Ok(Some(unsafe { LocalAllocGuard::from_raw(ptr) }))
}

fn null_config_result(count: u32) -> Result<Option<LocalAllocGuard<SID_AND_ATTRIBUTES>>> {
    if count == 0 {
        Ok(None)
    } else {
        Err(AcError::Win32(
            "NetworkIsolationGetAppContainerConfig returned null array with non-zero count"
                .to_string(),
        ))
    }
}

fn config_entries(
    guard: &Option<LocalAllocGuard<SID_AND_ATTRIBUTES>>,
    count: u32,
) -> Vec<SID_AND_ATTRIBUTES> {
    guard
        .as_ref()
        .map(|guard| {
            // SAFETY: The guard owns an array with `count` entries from the config API.
            unsafe {
                std::slice::from_raw_parts(
                    guard.as_ptr() as *const SID_AND_ATTRIBUTES,
                    count as usize,
                )
            }
            .to_vec()
        })
        .unwrap_or_default()
}

struct LoopbackSid {
    owned: OwnedSid,
    sddl: String,
}

impl LoopbackSid {
    fn from_app_container(sid: &AppContainerSid) -> Result<Self> {
        let sddl = sid.as_string().to_owned();
        let wide = crate::ffi::wstr::to_utf16(&sddl);
        let mut psid_raw = PSID::default();
        // SAFETY: Convert SDDL to a LocalAlloc-managed PSID and transfer ownership.
        unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid_raw) }
            .map_err(|e| AcError::Win32(format!("ConvertStringSidToSidW failed: {e}")))?;
        // SAFETY: ConvertStringSidToSidW allocates the SID through LocalAlloc.
        let owned = unsafe { OwnedSid::from_localfree_psid(psid_raw.0) }?;
        Ok(Self { owned, sddl })
    }

    fn psid(&self) -> PSID {
        self.owned.as_psid()
    }
}

fn update_loopback_entries(
    entries: &mut Vec<SID_AND_ATTRIBUTES>,
    allow: bool,
    target: &LoopbackSid,
) -> Result<()> {
    if allow {
        add_missing_loopback_entry(entries, target)
    } else {
        remove_loopback_entry(entries, &target.sddl)
    }
}

fn add_missing_loopback_entry(
    entries: &mut Vec<SID_AND_ATTRIBUTES>,
    target: &LoopbackSid,
) -> Result<()> {
    if !contains_sid(entries, &target.sddl)? {
        entries.push(SID_AND_ATTRIBUTES {
            Sid: target.psid(),
            Attributes: 0,
        });
    }
    Ok(())
}

fn remove_loopback_entry(entries: &mut Vec<SID_AND_ATTRIBUTES>, target_sddl: &str) -> Result<()> {
    let mut filtered = Vec::with_capacity(entries.len());
    for sa in entries.drain(..) {
        if !sid_matches(sa.Sid, target_sddl)? {
            filtered.push(sa);
        }
    }
    *entries = filtered;
    Ok(())
}

fn contains_sid(entries: &[SID_AND_ATTRIBUTES], target_sddl: &str) -> Result<bool> {
    for sa in entries {
        if sid_matches(sa.Sid, target_sddl)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn sid_matches(candidate: PSID, target_sddl: &str) -> Result<bool> {
    Ok(psid_to_string(candidate)? == target_sddl)
}

fn apply_loopback_config(entries: &[SID_AND_ATTRIBUTES]) -> Result<()> {
    // SAFETY: Entry SID pointers remain live through this call via LoopbackConfig/LoopbackSid.
    let err = unsafe { NetworkIsolationSetAppContainerConfig(entries) };
    if err != 0 {
        return Err(AcError::Win32(format!(
            "NetworkIsolationSetAppContainerConfig failed: {err}"
        )));
    }
    Ok(())
}

fn pwstr_to_string(ptr: PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    // SAFETY: Walk until trailing NUL, then build a slice over initialized code units.
    unsafe {
        while *ptr.0.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len))
    }
}

fn psid_to_string(psid: PSID) -> Result<String> {
    let mut raw = PWSTR::null();
    // SAFETY: Converts a valid SID to a LocalAlloc-managed SDDL buffer.
    unsafe { ConvertSidToStringSidW(psid, &mut raw) }
        .map_err(|e| AcError::Win32(format!("ConvertSidToStringSidW failed: {e}")))?;
    // SAFETY: ConvertSidToStringSidW returns a LocalAlloc-managed UTF-16 string.
    let guard = unsafe { LocalAllocGuard::<u16>::from_raw(raw.0) };
    // SAFETY: The returned buffer is NUL-terminated.
    Ok(unsafe { guard.to_string_lossy() })
}
