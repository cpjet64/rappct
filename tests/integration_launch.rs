#[cfg(windows)]
#[test]
fn skeleton_compiles_and_exports() {
    use std::mem::size_of;
    // Sanity: ensure core API types are linkable and sized.
    let _ = size_of::<rappct::LaunchOptions>();
}
