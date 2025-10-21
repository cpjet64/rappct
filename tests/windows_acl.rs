#[cfg(windows)]
use rappct::AppContainerProfile;
#[cfg(windows)]
use rappct::acl::{self, AccessMask, ResourcePath};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW, GetSecurityInfo,
    SDDL_REVISION_1, SE_FILE_OBJECT, SE_REGISTRY_KEY,
};
#[cfg(windows)]
use windows::Win32::Security::{ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};
#[cfg(windows)]
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, KEY_READ, KEY_WRITE,
    REG_CREATE_KEY_DISPOSITION, REG_CREATED_NEW_KEY, REG_OPTION_NON_VOLATILE, RegCloseKey,
    RegCreateKeyExW, RegDeleteTreeW, RegOpenKeyExW,
};
#[cfg(windows)]
use windows::core::PCWSTR;

#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn LocalFree(h: isize) -> isize;
}

#[cfg(windows)]
fn pwstr_to_string(ptr: windows::core::PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0usize;
        while *ptr.0.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len))
    }
}

#[cfg(windows)]
fn security_sddl_for_path(path: &std::path::Path) -> String {
    unsafe {
        let path_w: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut sd = PSECURITY_DESCRIPTOR::default();
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let status = GetNamedSecurityInfoW(
            PCWSTR(path_w.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut dacl),
            None,
            &mut sd,
        );
        assert_eq!(status.0, 0, "GetNamedSecurityInfoW failed: {:?}", status);
        let mut sddl_ptr = windows::core::PWSTR::null();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            None,
        )
        .expect("ConvertSecurityDescriptorToStringSecurityDescriptorW");
        let value = pwstr_to_string(sddl_ptr);
        if !sd.0.is_null() {
            let _ = LocalFree(sd.0 as isize);
        }
        if !sddl_ptr.is_null() {
            let _ = LocalFree(sddl_ptr.0 as isize);
        }
        value
    }
}

#[cfg(windows)]
fn parse_registry_spec(spec: &str) -> Option<(HKEY, Vec<u16>)> {
    let up = spec.to_ascii_uppercase();
    let (root, rest) = if let Some(_) = up.strip_prefix("HKCU\\") {
        (HKEY_CURRENT_USER, &spec[5..])
    } else if let Some(_) = up.strip_prefix("HKEY_CURRENT_USER\\") {
        (HKEY_CURRENT_USER, &spec[18..])
    } else if let Some(_) = up.strip_prefix("HKLM\\") {
        (HKEY_LOCAL_MACHINE, &spec[5..])
    } else if let Some(_) = up.strip_prefix("HKEY_LOCAL_MACHINE\\") {
        (HKEY_LOCAL_MACHINE, &spec[19..])
    } else {
        return None;
    };
    let wide: Vec<u16> = std::ffi::OsStr::new(rest)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    Some((root, wide))
}

#[cfg(windows)]
fn security_sddl_for_registry(spec: &str) -> String {
    unsafe {
        let (root, subkey_w) = parse_registry_spec(spec).expect("unsupported registry root");
        let mut hkey = HKEY::default();
        let status = RegOpenKeyExW(
            root,
            PCWSTR(subkey_w.as_ptr()),
            None,
            KEY_READ | KEY_WRITE,
            &mut hkey,
        );
        assert_eq!(status.0, 0, "RegOpenKeyExW failed: {:?}", status);
        let mut sd = PSECURITY_DESCRIPTOR::default();
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let status2 = GetSecurityInfo(
            HANDLE(hkey.0),
            SE_REGISTRY_KEY,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut dacl),
            None,
            Some(&mut sd),
        );
        assert_eq!(status2.0, 0, "GetSecurityInfo(reg) failed: {:?}", status2);
        let mut sddl_ptr = windows::core::PWSTR::null();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            None,
        )
        .expect("ConvertSecurityDescriptorToStringSecurityDescriptorW(reg)");
        let value = pwstr_to_string(sddl_ptr);
        if !sd.0.is_null() {
            let _ = LocalFree(sd.0 as isize);
        }
        if !sddl_ptr.is_null() {
            let _ = LocalFree(sddl_ptr.0 as isize);
        }
        let _ = RegCloseKey(hkey);
        value
    }
}

#[cfg(windows)]
#[test]
fn grant_to_package_updates_file_dacl() {
    use std::io::Write;

    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let path = temp.path().to_path_buf();
    writeln!(&mut temp.as_file().try_clone().unwrap(), "hello").unwrap();

    let profile =
        AppContainerProfile::ensure("rappct.test.acl.file", "rappct acl", Some("acl test"))
            .expect("ensure profile");
    let sid_str = profile.sid.as_string().to_string();

    let before = security_sddl_for_path(&path);
    assert!(
        !before.contains(&sid_str),
        "pre-grant DACL unexpectedly contained test SID: {before}"
    );

    acl::grant_to_package(
        ResourcePath::File(path.clone()),
        &profile.sid,
        AccessMask(0x120089),
    )
    .expect("grant file access");

    let after = security_sddl_for_path(&path);
    assert!(
        after.contains(&sid_str),
        "post-grant DACL missing SID {sid_str}: {after}"
    );

    profile.delete().ok();
}

#[cfg(windows)]
#[test]
fn grant_to_package_updates_registry_dacl() {
    use std::ffi::OsStr;

    let profile =
        AppContainerProfile::ensure("rappct.test.acl.reg", "rappct acl", Some("acl test"))
            .expect("ensure profile");
    let sid_str = profile.sid.as_string().to_string();

    let subkey = format!(r"Software\\rappct\\acl\\{}", std::process::id());
    let w: Vec<u16> = OsStr::new(&subkey)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut hkey = HKEY::default();
    let mut disposition = REG_CREATE_KEY_DISPOSITION(0);
    unsafe {
        let status = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(w.as_ptr()),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            None,
            &mut hkey,
            Some(&mut disposition),
        );
        assert_eq!(status.0, 0, "RegCreateKeyExW failed: {:?}", status);
        assert_eq!(disposition, REG_CREATED_NEW_KEY);
        let _ = RegCloseKey(hkey);
    }

    let full_spec = format!("HKCU\\{}", subkey);
    let before = security_sddl_for_registry(&full_spec);
    assert!(
        !before.contains(&sid_str),
        "pre-grant registry DACL unexpectedly contained SID"
    );

    acl::grant_to_package(
        ResourcePath::RegistryKey(full_spec.clone()),
        &profile.sid,
        AccessMask(0x20019),
    )
    .expect("grant registry access");

    let after = security_sddl_for_registry(&full_spec);
    assert!(
        after.contains(&sid_str),
        "post-grant registry DACL missing SID"
    );

    unsafe {
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(w.as_ptr()));
    }
    profile.delete().ok();
}
