#[cfg(windows)]
#[test]
fn launch_api_is_exported() {
    use std::mem::size_of;
    // Sanity: ensure core API types are linkable and sized.
    let _ = size_of::<rappct::LaunchOptions>();
}
