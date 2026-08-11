#[cfg(windows)]
use rappct::*;

#[cfg(windows)]
fn profile_name(scope: &str) -> String {
    format!("rappct.test.launch.{scope}.{}", std::process::id())
}

#[cfg(windows)]
fn profile_ensure_and_delete_roundtrip() {
    let name = profile_name("prof");
    let prof = AppContainerProfile::ensure(&name, "rappct test", Some("rappct test"))
        .expect("ensure profile");
    assert!(prof.sid.as_string().starts_with("S-1-15-"));
    // Folder path and named object path should resolve
    let _folder = prof.folder_path().expect("folder path");
    let named_obj = prof.named_object_path().expect("named object path");
    assert!(!named_obj.is_empty());
    // Cleanup
    prof.delete().expect("delete profile");
}

#[cfg(windows)]
fn profile_open_resolves_existing_name() {
    let name = profile_name("open");
    let first = AppContainerProfile::ensure(&name, "rappct open", Some("rappct open test"))
        .expect("ensure profile");
    let first_sid = first.sid.as_string().to_string();
    drop(first);

    let opened = AppContainerProfile::open(&name).expect("open existing profile");
    assert_eq!(first_sid, opened.sid.as_string());
    opened.delete().expect("delete opened profile");
}

#[cfg(windows)]
fn profile_open_matches_derived_sid_for_name() {
    let name = "rappct.invalid\\name";
    let derived = derive_sid_from_name(name).expect("derive sid");
    let opened = AppContainerProfile::open(name).expect("open profile by name");
    assert_eq!(opened.sid.as_string(), derived.as_string());
}

#[cfg(windows)]
fn profile_ensure_existing_handles_metadata_mismatch() {
    let name = profile_name("meta");
    let first =
        AppContainerProfile::ensure(&name, "rappct ensure", Some("display one")).expect("ensure");
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
fn profile_folder_path_fails_after_delete() {
    let name = profile_name("fold");
    let prof =
        AppContainerProfile::ensure(&name, "rappct folder", Some("folder test")).expect("ensure");
    let sid = prof.sid.clone();
    let pname = prof.name.clone();
    prof.delete().expect("delete");
    let ghost = AppContainerProfile {
        name: pname,
        sid: sid.clone(),
    };
    let error = ghost
        .folder_path()
        .expect_err("deleted profile folder lookup must fail closed");
    assert!(
        error
            .to_string()
            .contains("GetAppContainerFolderPath failed")
    );
}

#[cfg(windows)]
#[test]
fn profile_mutation_contracts() {
    profile_ensure_and_delete_roundtrip();
    profile_open_resolves_existing_name();
    profile_open_matches_derived_sid_for_name();
    profile_ensure_existing_handles_metadata_mismatch();
    profile_folder_path_fails_after_delete();
    profile_named_object_path_invalid_sid_errors();
}

#[cfg(windows)]
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
        other => panic!("unexpected error: {other:?}"),
    }
}
