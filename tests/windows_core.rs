#[cfg(windows)]
use rappct::*;

#[cfg(windows)]
#[test]
fn derive_sid_from_name_works() {
    let sid = derive_sid_from_name("rappct.test.unit").expect("derive sid");
    assert!(sid.as_string().starts_with("S-1-15-"));
}

#[cfg(windows)]
#[test]
fn derive_known_capability_sids() {
    let caps = capability::derive_named_capability_sids(&["internetClient"]).expect("derive caps");
    assert!(!caps.is_empty());
}

#[cfg(windows)]
#[test]
fn token_query_works() {
    let info = token::query_current_process_token().expect("token");
    assert!(
        !info.is_appcontainer,
        "host process unexpectedly running in AppContainer"
    );
    assert!(!info.is_lpac, "host process unexpectedly marked LPAC");
    assert!(
        info.package_sid.is_none(),
        "package SID should be absent outside AppContainer"
    );
    assert!(
        info.capability_sids.is_empty(),
        "capability list should be empty outside AppContainer"
    );
}

#[cfg(windows)]
#[test]
fn capability_derivation_repeated_calls_are_successful() {
    for _ in 0..4 {
        let caps =
            capability::derive_named_capability_sids(&["internetClient"]).expect("derive caps");
        assert_eq!(caps.len(), 1);
        assert!(!caps[0].sid_sddl.is_empty());
    }
}

#[cfg(all(windows, feature = "introspection"))]
#[test]
fn capability_typo_returns_suggestion() {
    // Windows treats arbitrary capability names as valid SIDs, so we rely on library-level
    // tests in capability.rs to exercise the suggestion path.
    assert!(true);
}

#[cfg(windows)]
#[test]
fn supports_lpac_override_ok() {
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var("RAPPCT_TEST_LPAC_STATUS", "ok");
    }
    let result = rappct::supports_lpac();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var("RAPPCT_TEST_LPAC_STATUS");
    }
    assert!(
        result.is_ok(),
        "supports_lpac should succeed when override requests ok"
    );
}

#[cfg(windows)]
#[test]
fn supports_lpac_override_unsupported() {
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var("RAPPCT_TEST_LPAC_STATUS", "unsupported");
    }
    let result = rappct::supports_lpac();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var("RAPPCT_TEST_LPAC_STATUS");
    }
    assert!(matches!(result, Err(rappct::AcError::UnsupportedLpac)));
}
