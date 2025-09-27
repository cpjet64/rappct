use rappct::sid::AppContainerSid;
#[cfg(windows)]
use rappct::*;

#[cfg(windows)]
#[test]
fn profile_ensure_and_delete_roundtrip() {
    // Use a quasi-unique name to avoid collisions
    let name = format!("rappct.test.{}", std::process::id());
    let prof =
        AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure profile");
    assert!(prof.sid.as_string().starts_with("S-1-15-"));
    // Folder path and named object path should resolve
    let _folder = prof.folder_path().expect("folder path");
    let named_obj = prof.named_object_path().expect("named object path");
    assert!(!named_obj.is_empty());
    // Cleanup
    prof.delete().expect("delete profile");
}

#[cfg(windows)]
#[test]
fn profile_ensure_existing_handles_metadata_mismatch() {
    let name = format!("rappct.test.profile.ensure.{}", std::process::id());
    let first = AppContainerProfile::ensure(&name, &name, Some("display one")).expect("ensure");
    let sid_first = first.sid.as_string().to_string();
    drop(first);
    let second = AppContainerProfile::ensure(&name, "different display", Some("display two"))
        .expect("ensure existing");
    assert_eq!(
        sid_first,
        second.sid.as_string(),
        "SID changed after metadata mismatch"
    );
    second.delete().expect("delete profile");
}

#[cfg(windows)]
#[test]
fn profile_folder_path_fallback_after_delete() {
    use std::path::PathBuf;
    let name = format!("rappct.test.profile.folder.{}", std::process::id());
    let prof = AppContainerProfile::ensure(&name, &name, Some("folder test")).expect("ensure");
    let sid = prof.sid.clone();
    let pname = prof.name.clone();
    prof.delete().expect("delete");
    let ghost = AppContainerProfile {
        name: pname,
        sid: sid.clone(),
    };
    let path = ghost.folder_path().expect("folder path fallback");
    let base = std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA not set");
    let expected = PathBuf::from(base).join("Packages").join(sid.as_string());
    assert_eq!(path, expected);
}

#[cfg(windows)]
#[test]
fn profile_named_object_path_invalid_sid_errors() {
    let bogus = AppContainerProfile {
        name: "rappct.invalid".to_string(),
        sid: AppContainerSid::from_sddl("invalid-sddl"),
    };
    let err = bogus
        .named_object_path()
        .expect_err("should fail for invalid SID");
    match err {
        AcError::Win32(msg) => assert!(msg.contains("ConvertStringSidToSidW")),
        other => panic!("unexpected error: {:?}", other),
    }
}
