#[cfg(windows)]
use rappct::acl::{self, AccessMask, ResourcePath};
#[cfg(windows)]
use rappct::derive_sid_from_name;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW, GetSecurityInfo,
    SDDL_REVISION_1, SE_FILE_OBJECT, SE_REGISTRY_KEY, SetNamedSecurityInfoW,
};
#[cfg(windows)]
use windows::Win32::Security::{ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
#[cfg(windows)]
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, KEY_READ, KEY_WRITE,
    REG_CREATE_KEY_DISPOSITION, REG_CREATED_NEW_KEY, REG_OPENED_EXISTING_KEY,
    REG_OPTION_NON_VOLATILE, RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegOpenKeyExW,
};
#[cfg(windows)]
use windows::core::PCWSTR;

#[cfg(windows)]
#[path = "support/windows_test_utils.rs"]
mod windows_test_utils;
#[cfg(windows)]
use crate::windows_test_utils::{LocalAlloc, LocalWideString};

#[cfg(windows)]
const DEFAULT_TEST_MINIMUM_FREE_BYTES: u64 = 5 * 1024 * 1024 * 1024;

#[cfg(windows)]
fn test_minimum_free_bytes() -> u64 {
    match std::env::var("RAPPCT_TEST_MINIMUM_FREE_BYTES") {
        Ok(value) => value
            .parse::<u64>()
            .ok()
            .filter(|minimum| *minimum > 0)
            .expect("RAPPCT_TEST_MINIMUM_FREE_BYTES must be a positive integer"),
        Err(std::env::VarError::NotPresent) => DEFAULT_TEST_MINIMUM_FREE_BYTES,
        Err(error) => panic!("RAPPCT_TEST_MINIMUM_FREE_BYTES is not valid Unicode: {error}"),
    }
}

#[cfg(windows)]
fn repo_local_scratch_root() -> std::path::PathBuf {
    let repo_root = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR"))
        .expect("canonicalize repository root for test scratch");
    let scratch_root = repo_root.join(".tmp").join("tests").join("windows-acl");
    std::fs::create_dir_all(&scratch_root).expect("create repository-local test scratch root");
    let scratch_root = std::fs::canonicalize(&scratch_root)
        .expect("canonicalize repository-local test scratch root");
    assert!(
        scratch_root != repo_root && scratch_root.starts_with(&repo_root),
        "refusing test scratch outside repository root: {}",
        scratch_root.display()
    );

    let scratch_root_w: Vec<u16> = scratch_root
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut available_free_bytes = 0;
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(scratch_root_w.as_ptr()),
            Some(&mut available_free_bytes),
            None,
            None,
        )
    }
    .expect("query repository-local test scratch capacity");
    let minimum_free_bytes = test_minimum_free_bytes();
    assert!(
        available_free_bytes >= minimum_free_bytes,
        "insufficient free space for repository-local test scratch at {}: \
         {available_free_bytes} bytes available; {minimum_free_bytes} bytes required",
        scratch_root.display()
    );

    scratch_root
}

#[cfg(windows)]
fn repo_local_tempdir(prefix: &str) -> tempfile::TempDir {
    let scratch_root = repo_local_scratch_root();
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(&scratch_root)
        .expect("create repository-local test scratch directory")
}

#[cfg(windows)]
fn repo_local_named_tempfile(prefix: &str) -> tempfile::NamedTempFile {
    let scratch_root = repo_local_scratch_root();
    tempfile::Builder::new()
        .prefix(prefix)
        .tempfile_in(&scratch_root)
        .expect("create repository-local test scratch file")
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
        assert_eq!(status.0, 0, "GetNamedSecurityInfoW failed: {status:?}");
        let mut sddl_ptr = windows::core::PWSTR::null();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            None,
        )
        .expect("ConvertSecurityDescriptorToStringSecurityDescriptorW");
        let _sd = LocalAlloc::<u8>::from_raw(sd.0 as *mut u8);
        LocalWideString::from_raw(sddl_ptr).to_string_lossy()
    }
}

#[cfg(windows)]
fn path_has_null_dacl(path: &std::path::Path) -> bool {
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
        assert_eq!(status.0, 0, "GetNamedSecurityInfoW failed: {status:?}");
        let _sd = LocalAlloc::<u8>::from_raw(sd.0 as *mut u8);
        dacl.is_null()
    }
}

#[cfg(windows)]
fn parse_registry_spec(spec: &str) -> Option<(HKEY, Vec<u16>)> {
    let up = spec.to_ascii_uppercase();
    let (root, rest) = if up.strip_prefix("HKCU\\").is_some() {
        (HKEY_CURRENT_USER, &spec[5..])
    } else if up.strip_prefix("HKEY_CURRENT_USER\\").is_some() {
        (HKEY_CURRENT_USER, &spec[18..])
    } else if up.strip_prefix("HKLM\\").is_some() {
        (HKEY_LOCAL_MACHINE, &spec[5..])
    } else if up.strip_prefix("HKEY_LOCAL_MACHINE\\").is_some() {
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
        assert_eq!(status.0, 0, "RegOpenKeyExW failed: {status:?}");
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
        assert_eq!(status2.0, 0, "GetSecurityInfo(reg) failed: {status2:?}");
        let mut sddl_ptr = windows::core::PWSTR::null();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            None,
        )
        .expect("ConvertSecurityDescriptorToStringSecurityDescriptorW(reg)");
        let _sd = LocalAlloc::<u8>::from_raw(sd.0 as *mut u8);
        let sddl = LocalWideString::from_raw(sddl_ptr).to_string_lossy();
        let _ = RegCloseKey(hkey);
        sddl
    }
}

#[cfg(windows)]
fn grant_to_package_updates_file_dacl() {
    use std::io::Write;

    let temp = repo_local_named_tempfile("file-dacl-");
    let path = temp.path().to_path_buf();
    writeln!(&mut temp.as_file().try_clone().unwrap(), "hello").unwrap();

    let name = format!("rappct.test.acl.file.{}", std::process::id());
    let sid = derive_sid_from_name(&name).expect("derive package SID");
    let sid_str = sid.as_string().to_string();

    let before = security_sddl_for_path(&path);
    assert!(
        !before.contains(&sid_str),
        "pre-grant DACL unexpectedly contained test SID: {before}"
    );

    acl::grant_to_package(ResourcePath::File(path.clone()), &sid, AccessMask(0x120089))
        .expect("grant file access");

    let after = security_sddl_for_path(&path);
    assert!(
        after.contains(&sid_str),
        "post-grant DACL missing SID {sid_str}: {after}"
    );
}

#[cfg(windows)]
fn grant_preserves_existing_null_file_dacl() {
    let temp = repo_local_named_tempfile("null-dacl-");
    let path = temp.path().to_path_buf();
    let path_w: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let status = unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(path_w.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            None,
            None,
        )
    };
    assert_eq!(status.0, 0, "set null DACL failed: {status:?}");
    assert!(path_has_null_dacl(&path));

    let sid = derive_sid_from_name("rappct.test.acl.null").expect("derive package SID");
    acl::grant_to_package(ResourcePath::File(path.clone()), &sid, AccessMask(0x120089))
        .expect("null DACL already grants requested access");
    assert!(path_has_null_dacl(&path), "grant replaced the null DACL");
}

#[cfg(windows)]
fn grant_to_package_updates_registry_dacl() {
    use std::ffi::OsStr;
    use std::time::{SystemTime, UNIX_EPOCH};

    let name = format!("rappct.test.acl.reg.{}", std::process::id());
    let sid = derive_sid_from_name(&name).expect("derive package SID");
    let sid_str = sid.as_string().to_string();

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let subkey = format!(r"Software\rappct-acl-{}-{nonce}", std::process::id());
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
        assert_eq!(status.0, 0, "RegCreateKeyExW failed: {status:?}");
        assert!(
            disposition == REG_CREATED_NEW_KEY || disposition == REG_OPENED_EXISTING_KEY,
            "unexpected key creation disposition: {disposition:?}"
        );
        let _ = RegCloseKey(hkey);
    }

    let full_spec = format!("HKCU\\{subkey}");
    let before = security_sddl_for_registry(&full_spec);
    assert!(
        !before.contains(&sid_str),
        "pre-grant registry DACL unexpectedly contained SID"
    );

    acl::grant_to_package(
        ResourcePath::RegistryKey(full_spec.clone()),
        &sid,
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
}

#[cfg(windows)]
fn grant_to_package_updates_directory_custom_dacl() {
    let root = repo_local_tempdir("custom-dacl-");
    let dir_path = root.path().join("acl-dir");
    std::fs::create_dir_all(&dir_path).expect("create dir");

    let name = format!("rappct.test.acl.dir.{}", std::process::id());
    let sid = derive_sid_from_name(&name).expect("derive package SID");
    let sid_str = sid.as_string().to_string();

    let before = security_sddl_for_path(&dir_path);
    assert!(
        !before.contains(&sid_str),
        "pre-grant directory DACL unexpectedly contained SID"
    );

    acl::grant_to_package(
        ResourcePath::DirectoryCustom(dir_path.clone(), acl::AceInheritance::OBJECTS_ONLY),
        &sid,
        AccessMask(0x120089),
    )
    .expect("grant directory access");

    let after = security_sddl_for_path(&dir_path);
    assert!(
        after.contains(&sid_str),
        "post-grant directory DACL missing SID {sid_str}: {after}"
    );
}

#[cfg(windows)]
fn grant_to_package_updates_directory_default_inheritance_dacl() {
    let root = repo_local_tempdir("default-inheritance-dacl-");
    let dir_path = root.path().join("acl-dir-default");
    std::fs::create_dir_all(&dir_path).expect("create dir");

    let name = format!("rappct.test.acl.dir.default.{}", std::process::id());
    let sid = derive_sid_from_name(&name).expect("derive package SID");
    let sid_str = sid.as_string().to_string();

    let before = security_sddl_for_path(&dir_path);
    assert!(
        !before.contains(&sid_str),
        "pre-grant directory DACL unexpectedly contained SID"
    );

    acl::grant_to_package(
        ResourcePath::Directory(dir_path.clone()),
        &sid,
        AccessMask(0x120089),
    )
    .expect("grant directory access");

    let after = security_sddl_for_path(&dir_path);
    assert!(
        after.contains(&sid_str),
        "post-grant directory DACL missing SID {sid_str}: {after}"
    );
}

#[cfg(windows)]
#[test]
fn acl_grant_contracts() {
    grant_to_package_updates_registry_dacl();
    grant_to_package_updates_file_dacl();
    grant_preserves_existing_null_file_dacl();
    grant_to_package_updates_directory_custom_dacl();
    grant_to_package_updates_directory_default_inheritance_dacl();
}
