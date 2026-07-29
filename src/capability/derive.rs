use crate::sid::SidAndAttributes;
use crate::{AcError, Result};

/// Derive capability SIDs from names.
pub fn derive_named_capability_sids(names: &[&str]) -> Result<Vec<SidAndAttributes>> {
    if names.is_empty() {
        return Ok(vec![]);
    }
    #[cfg(windows)]
    {
        let mut out = Vec::new();
        for &name in names {
            out.append(&mut derive_single_capability_sids(name)?);
        }
        Ok(out)
    }
    #[cfg(not(windows))]
    {
        let _ = names;
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
pub(super) fn derive_single_capability_sids(name: &str) -> Result<Vec<SidAndAttributes>> {
    let arrays = derive_raw_sid_arrays(name)?;
    arrays.validate(name)?;
    arrays.groups.release_entries();
    convert_capability_sids(name, &arrays.capabilities)
}

#[cfg(windows)]
type SidPtr = *mut std::ffi::c_void;

#[cfg(windows)]
type SidArrayPtr = *mut SidPtr;

#[cfg(windows)]
struct DerivedSidArrays {
    groups: SidPointerArray,
    capabilities: SidPointerArray,
}

#[cfg(windows)]
struct SidPointerArray {
    ptr: SidArrayPtr,
    count: u32,
    label: &'static str,
}

#[cfg(windows)]
impl DerivedSidArrays {
    fn validate(&self, name: &str) -> Result<()> {
        self.groups.validate(name)?;
        self.capabilities.validate(name)
    }
}

#[cfg(windows)]
impl SidPointerArray {
    fn new(ptr: SidArrayPtr, count: u32, label: &'static str) -> Self {
        Self { ptr, count, label }
    }

    fn validate(&self, name: &str) -> Result<()> {
        if self.count > 0 && self.ptr.is_null() {
            return Err(AcError::Win32(format!(
                "DeriveCapabilitySidsFromName returned null {} SID array for '{name}' (count={})",
                self.label, self.count
            )));
        }
        Ok(())
    }

    fn release_entries(&self) {
        for index in 0..self.count as isize {
            if let Some(sid) = self.sid_at(index) {
                // SAFETY: Per DeriveCapabilitySidsFromName docs, each returned SID must be freed
                // with LocalFree. The guard is dropped immediately because callers do not use it.
                let _guard =
                    unsafe { crate::ffi::mem::LocalAllocGuard::<std::ffi::c_void>::from_raw(sid) };
            }
        }
    }

    fn sid_at(&self, index: isize) -> Option<SidPtr> {
        if self.ptr.is_null() {
            return None;
        }
        // SAFETY: The caller only requests indexes below the API-reported count.
        let sid = unsafe { *self.ptr.offset(index) };
        (!sid.is_null()).then_some(sid)
    }
}

#[cfg(windows)]
impl Drop for SidPointerArray {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: The array pointer is LocalAlloc-managed by DeriveCapabilitySidsFromName.
            let _guard = unsafe { crate::ffi::mem::LocalAllocGuard::<SidPtr>::from_raw(self.ptr) };
            self.ptr = std::ptr::null_mut();
        }
    }
}

#[cfg(windows)]
fn derive_raw_sid_arrays(name: &str) -> Result<DerivedSidArrays> {
    use crate::ffi::wstr::WideString;
    use windows::core::PCWSTR;

    #[link(name = "Userenv")]
    unsafe extern "system" {
        fn DeriveCapabilitySidsFromName(
            CapName: PCWSTR,
            CapGroupSids: *mut SidArrayPtr,
            CapGroupSidCount: *mut u32,
            CapabilitySids: *mut SidArrayPtr,
            CapabilitySidCount: *mut u32,
        ) -> i32;
    }

    let wide = WideString::from_str(name);
    let mut group_sids = std::ptr::null_mut();
    let mut group_count = 0;
    let mut cap_sids = std::ptr::null_mut();
    let mut cap_count = 0;
    // SAFETY: The API receives a valid NUL-terminated capability name and output pointers.
    let ok = unsafe {
        DeriveCapabilitySidsFromName(
            wide.as_pcwstr(),
            &mut group_sids,
            &mut group_count,
            &mut cap_sids,
            &mut cap_count,
        )
    };

    let arrays = DerivedSidArrays {
        groups: SidPointerArray::new(group_sids, group_count, "group"),
        capabilities: SidPointerArray::new(cap_sids, cap_count, "capability"),
    };
    if ok == 0 {
        return Err(unknown_capability(name));
    }
    Ok(arrays)
}

#[cfg(windows)]
fn unknown_capability(name: &str) -> AcError {
    #[cfg(feature = "introspection")]
    let suggestion = super::suggest_capability_name(name);
    #[cfg(not(feature = "introspection"))]
    let suggestion: Option<&'static str> = None;
    AcError::UnknownCapability {
        name: name.to_string(),
        suggestion,
    }
}

#[cfg(windows)]
fn convert_capability_sids(
    name: &str,
    capabilities: &SidPointerArray,
) -> Result<Vec<SidAndAttributes>> {
    let mut out = Vec::new();
    let mut conversion_error = None;
    for index in 0..capabilities.count as isize {
        if let Some(sid) = capabilities.sid_at(index) {
            collect_capability_sid(name, sid, &mut out, &mut conversion_error);
        }
    }
    if let Some(err) = conversion_error {
        return Err(err);
    }
    verify_capability_count(name, capabilities.count, out.len())?;
    Ok(out)
}

#[cfg(windows)]
fn collect_capability_sid(
    name: &str,
    sid: SidPtr,
    out: &mut Vec<SidAndAttributes>,
    conversion_error: &mut Option<AcError>,
) {
    // SAFETY: Each SID pointer is LocalAlloc-managed by DeriveCapabilitySidsFromName.
    let sid_guard = unsafe { crate::ffi::mem::LocalAllocGuard::<std::ffi::c_void>::from_raw(sid) };
    match sid_to_sddl(sid_guard.as_ptr()) {
        Ok(sid_sddl) => out.push(SidAndAttributes {
            sid_sddl,
            attributes: crate::ffi::SE_GROUP_ENABLED,
        }),
        Err(err) if conversion_error.is_none() => {
            *conversion_error = Some(err_with_name(name, err))
        }
        Err(_) => {}
    }
}

#[cfg(windows)]
fn sid_to_sddl(sid: SidPtr) -> Result<String> {
    use crate::ffi::mem::LocalAllocGuard;
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::PSID;
    use windows::core::PWSTR;

    let mut sddl = PWSTR::null();
    // SAFETY: The pointer is a valid SID returned by DeriveCapabilitySidsFromName.
    unsafe { ConvertSidToStringSidW(PSID(sid), &mut sddl) }
        .map_err(|err| AcError::Win32(format!("{err:?}")))?;
    // SAFETY: ConvertSidToStringSidW returns a LocalAlloc-managed UTF-16 string.
    let sddl_guard = unsafe { LocalAllocGuard::<u16>::from_raw(sddl.0) };
    // SAFETY: ConvertSidToStringSidW returns a valid NUL-terminated string.
    Ok(unsafe { sddl_guard.to_string_lossy() })
}

#[cfg(windows)]
fn err_with_name(name: &str, source: AcError) -> AcError {
    AcError::Win32(format!(
        "ConvertSidToStringSidW failed for capability '{name}': {source:?}"
    ))
}

#[cfg(windows)]
fn verify_capability_count(name: &str, expected: u32, actual: usize) -> Result<()> {
    if actual != expected as usize {
        return Err(AcError::Win32(format!(
            "Derived {expected} capability SID(s) for '{name}' but converted {actual}"
        )));
    }
    Ok(())
}
