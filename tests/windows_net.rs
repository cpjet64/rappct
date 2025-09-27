#[cfg(all(windows, feature = "net"))]
use rappct::*;

#[cfg(all(windows, feature = "net"))]
use windows::core::PWSTR;
#[cfg(all(windows, feature = "net"))]
use windows::Win32::NetworkManagement::WindowsFirewall::NetworkIsolationGetAppContainerConfig;
#[cfg(all(windows, feature = "net"))]
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
#[cfg(all(windows, feature = "net"))]
use windows::Win32::Security::SID_AND_ATTRIBUTES;

#[cfg(all(windows, feature = "net"))]
#[link(name = "Kernel32")]
extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(all(windows, feature = "net"))]
fn pwstr_to_string(ptr: PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    unsafe {
        while *ptr.0.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len))
    }
}

#[cfg(all(windows, feature = "net"))]
fn loopback_config_sids() -> Result<Vec<String>> {
    unsafe {
        let mut count: u32 = 0;
        let mut arr: *mut SID_AND_ATTRIBUTES = std::ptr::null_mut();
        let err = NetworkIsolationGetAppContainerConfig(&mut count, &mut arr);
        if err != 0 {
            return Err(AcError::Win32(format!(
                "NetworkIsolationGetAppContainerConfig failed: {err}"
            )));
        }
        let slice = if arr.is_null() {
            &[][..]
        } else {
            std::slice::from_raw_parts(arr, count as usize)
        };
        let mut out = Vec::with_capacity(slice.len());
        for sa in slice {
            let mut raw = PWSTR::null();
            ConvertSidToStringSidW(sa.Sid, &mut raw)
                .map_err(|e| AcError::Win32(format!("ConvertSidToStringSidW failed: {e}")))?;
            out.push(pwstr_to_string(raw));
            LocalFree(raw.0 as isize);
        }
        if !arr.is_null() {
            LocalFree(arr as isize);
        }
        Ok(out)
    }
}

#[cfg(all(windows, feature = "net"))]
#[test]
fn loopback_requires_confirm() {
    let sid = derive_sid_from_name("rappct.test.net").expect("derive sid");
    let res = net::add_loopback_exemption(net::LoopbackAdd(sid));
    match res {
        Err(AcError::AccessDenied { context, .. }) => {
            assert!(context.contains("confirm_debug_only"));
        }
        other => panic!("expected AccessDenied, got {:?}", other),
    }
}

#[cfg(all(windows, feature = "net"))]
#[test]
fn loopback_add_remove_roundtrip() {
    use std::collections::HashSet;

    let name = format!("rappct.test.net.loopback.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("rappct net test")).expect("ensure");
    let sid = prof.sid.clone();
    let sid_str = sid.as_string();

    net::remove_loopback_exemption(&sid).ok();
    let before: HashSet<String> = loopback_config_sids()
        .expect("query before add")
        .into_iter()
        .collect();
    assert!(
        !before.contains(sid_str),
        "loopback config already contained test SID"
    );

    net::add_loopback_exemption(net::LoopbackAdd(sid.clone()).confirm_debug_only())
        .expect("add loopback exemption");

    let after_add: HashSet<String> = loopback_config_sids()
        .expect("query after add")
        .into_iter()
        .collect();
    assert!(
        after_add.contains(sid_str),
        "loopback config missing SID after add"
    );

    let res = net::add_loopback_exemption(net::LoopbackAdd(sid.clone()));
    match res {
        Err(AcError::AccessDenied { context, .. }) => {
            assert!(context.contains("confirm_debug_only"));
        }
        other => panic!("expected safety latch failure, got {:?}", other),
    }

    net::remove_loopback_exemption(&sid).expect("remove loopback");
    let after_remove: HashSet<String> = loopback_config_sids()
        .expect("query after remove")
        .into_iter()
        .collect();
    assert!(
        !after_remove.contains(sid_str),
        "loopback config still contains SID after removal"
    );

    prof.delete().ok();
}

#[cfg(all(windows, feature = "net"))]
#[test]
fn list_appcontainers_cross_checks() {
    let result = net::list_appcontainers().expect("list appcontainers");
    let _ = result;
}
