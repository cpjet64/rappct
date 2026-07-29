use super::{AccessMask, AceInheritance};

#[cfg(windows)]
fn repo_local_tempdir(prefix: &str) -> tempfile::TempDir {
    let scratch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join("tests")
        .join("acl");
    std::fs::create_dir_all(&scratch_root).expect("create repository-local test scratch root");
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(&scratch_root)
        .expect("create repository-local test scratch directory")
}

#[test]
fn constants_are_consistent() {
    assert_eq!(AccessMask::GENERIC_ALL.0, 0x001F_01FF);
    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE};
        assert_eq!(AccessMask::FILE_GENERIC_READ.0, FILE_GENERIC_READ.0);
        assert_eq!(AccessMask::FILE_GENERIC_WRITE.0, FILE_GENERIC_WRITE.0);
    }
}

#[test]
fn ace_inheritance_constants_match_win32_values() {
    assert_eq!(AceInheritance::NONE.0, 0x0);
    assert_eq!(AceInheritance::OBJECTS_ONLY.0, 0x1);
    assert_eq!(AceInheritance::SUB_CONTAINERS_ONLY.0, 0x2);
    assert_eq!(AceInheritance::SUB_CONTAINERS_AND_OBJECTS.0, 0x3);
}

#[cfg(windows)]
#[test]
fn grant_rejects_nonexistent_file() {
    use super::{ResourcePath, grant_to_package};
    use crate::sid::AppContainerSid;

    let sid = AppContainerSid::from_sddl("S-1-15-2-1");
    let path = std::path::PathBuf::from("C:\\__rappct_nonexistent_file_test__");
    let err =
        grant_to_package(ResourcePath::File(path), &sid, AccessMask::GENERIC_ALL).unwrap_err();
    assert_error_contains(err, &["Resource not found", "create the file"]);
}

#[cfg(windows)]
#[test]
fn grant_rejects_nonexistent_directory() {
    use super::{ResourcePath, grant_to_package};
    use crate::sid::AppContainerSid;

    let sid = AppContainerSid::from_sddl("S-1-15-2-1");
    let path = std::path::PathBuf::from("C:\\__rappct_nonexistent_dir_test__");
    let err =
        grant_to_package(ResourcePath::Directory(path), &sid, AccessMask::GENERIC_ALL).unwrap_err();
    assert_error_contains(err, &["Resource not found", "create the directory"]);
}

#[cfg(windows)]
#[test]
fn grant_rejects_unsupported_registry_root() {
    use super::{ResourcePath, grant_to_package};
    use crate::sid::AppContainerSid;

    let sid = AppContainerSid::from_sddl("S-1-15-2-1");
    let err = grant_to_package(
        ResourcePath::RegistryKey("HKCR\\Software".into()),
        &sid,
        AccessMask::GENERIC_ALL,
    )
    .unwrap_err();
    assert_error_contains(err, &["Unsupported registry root"]);
}

#[cfg(windows)]
#[test]
fn grant_rejects_invalid_sddl() {
    use super::{ResourcePath, grant_to_package};
    use crate::sid::AppContainerSid;

    let sid = AppContainerSid::from_sddl("not-a-valid-sid");
    let scratch = repo_local_tempdir("invalid-package-sddl-");
    let err = grant_to_package(
        ResourcePath::Directory(scratch.path().to_path_buf()),
        &sid,
        AccessMask::GENERIC_ALL,
    )
    .unwrap_err();
    assert_error_contains(err, &["ConvertStringSidToSidW"]);
}

#[cfg(windows)]
#[test]
fn grant_to_capability_rejects_invalid_sddl() {
    use super::{ResourcePath, grant_to_capability};

    let scratch = repo_local_tempdir("invalid-capability-sddl-");
    let err = grant_to_capability(
        ResourcePath::Directory(scratch.path().to_path_buf()),
        "not-a-valid-capability-sid",
        AccessMask::GENERIC_ALL,
    )
    .unwrap_err();
    assert_error_contains(err, &["ConvertStringSidToSidW"]);
}

#[cfg(windows)]
#[test]
fn grant_rejects_nonexistent_registry_key() {
    assert_missing_registry_key("HKCU\\Software\\__rappct_nonexistent_key_test__");
}

#[cfg(windows)]
#[test]
fn grant_rejects_nonexistent_registry_key_with_full_root_name() {
    assert_missing_registry_key(
        "HKEY_CURRENT_USER\\Software\\__rappct_nonexistent_key_test_full__",
    );
}

#[cfg(windows)]
#[test]
fn grant_rejects_nonexistent_registry_key_with_lowercase_root_prefix() {
    assert_missing_registry_key("hkcu\\software\\__rappct_nonexistent_key_test_lowercase__");
}

#[cfg(windows)]
fn assert_missing_registry_key(spec: &str) {
    use super::{ResourcePath, grant_to_package};
    use crate::sid::AppContainerSid;

    let sid = AppContainerSid::from_sddl("S-1-15-2-1");
    let err = grant_to_package(
        ResourcePath::RegistryKey(spec.into()),
        &sid,
        AccessMask::GENERIC_ALL,
    )
    .unwrap_err();
    assert_error_contains(err, &["RegOpenKeyExW"]);
}

#[cfg(windows)]
fn assert_error_contains(err: crate::AcError, expected_parts: &[&str]) {
    let msg = err.to_string();
    for expected in expected_parts {
        assert!(msg.contains(expected), "expected {expected:?}, got: {msg}");
    }
}
