#![cfg(all(windows, feature = "net"))]

use rappct::{AppContainerProfile, net::LoopbackExemptionGuard};

// Opt-in: this test mutates firewall config; set env var to run
#[test]
#[ignore]
fn loopback_guard_roundtrip_opt_in() {
    if std::env::var_os("RAPPCT_ALLOW_NET_TESTS").is_none() {
        return;
    }

    let name = format!("rappct.test.net.guard.{}", std::process::id());
    let prof =
        AppContainerProfile::ensure(&name, &name, Some("rappct guard test")).expect("ensure");
    let sid = prof.sid.clone();

    // Ensure removed before we start
    let _ = rappct::net::remove_loopback_exemption(&sid);

    // Add via guard
    let guard = LoopbackExemptionGuard::new(&sid).expect("guard new");
    // Query and ensure present
    assert!(loopback_config_contains(sid.as_string()).expect("query after add"));
    drop(guard);
    // Ensure removed on drop
    assert!(!loopback_config_contains(sid.as_string()).expect("query after drop"));

    prof.delete().ok();
}

fn loopback_config_contains<S: AsRef<str>>(sid_str: S) -> rappct::Result<bool> {
    use windows::Win32::NetworkManagement::WindowsFirewall::NetworkIsolationGetAppContainerConfig;
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::SID_AND_ATTRIBUTES;
    use windows::core::PWSTR;
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn LocalFree(h: isize) -> isize;
    }
    unsafe fn pwstr_to_string_and_free(ptr: PWSTR) -> String {
        if ptr.is_null() {
            return String::new();
        }
        let mut len = 0usize;
        unsafe {
            while *ptr.0.add(len) != 0 {
                len += 1;
            }
        }
        let s = unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len)) };
        unsafe {
            let _ = LocalFree(ptr.0 as isize);
        }
        s
    }
    unsafe fn local_free_ptr<T>(ptr: *mut T) {
        if !ptr.is_null() {
            unsafe {
                let _ = LocalFree(ptr as isize);
            }
        }
    }
    unsafe {
        let mut count: u32 = 0;
        let mut arr: *mut SID_AND_ATTRIBUTES = std::ptr::null_mut();
        let err = NetworkIsolationGetAppContainerConfig(&mut count, &mut arr);
        if err != 0 {
            return Err(rappct::AcError::Win32(format!(
                "NetworkIsolationGetAppContainerConfig failed: {err}"
            )));
        }
        let slice = if arr.is_null() {
            &[][..]
        } else {
            std::slice::from_raw_parts(arr, count as usize)
        };
        let target = sid_str.as_ref();
        for sa in slice {
            let mut raw = PWSTR::null();
            ConvertSidToStringSidW(sa.Sid, &mut raw).map_err(|e| {
                rappct::AcError::Win32(format!("ConvertSidToStringSidW failed: {e}"))
            })?;
            let s = pwstr_to_string_and_free(raw);
            if s == target {
                return Ok(true);
            }
        }
        if !arr.is_null() {
            local_free_ptr(arr);
        }
        Ok(false)
    }
}
