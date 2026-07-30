use rappct::{CapabilityName, LaunchOptions, SecurityCapabilitiesBuilder};

fn assert_send_sync<T: Send + Sync>() {}

fn main() {
    assert_send_sync::<LaunchOptions>();
    let sid = rappct::AppContainerSid::from_sddl("S-1-15-2-1");
    let capabilities = SecurityCapabilitiesBuilder::new(&sid)
        .with_known(&[CapabilityName::InternetClient])
        .build()
        .expect("build downstream capability set");
    assert_eq!(capabilities.caps.len(), 1);
}
