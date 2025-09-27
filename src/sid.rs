//! AppContainer SID wrappers (skeleton). In v0.2 this will own PSIDs properly.

#[derive(Clone, Debug)]
pub struct AppContainerSid {
    sddl: String,
}

impl AppContainerSid {
    pub fn from_sddl(s: impl Into<String>) -> Self {
        Self { sddl: s.into() }
    }
    pub fn as_string(&self) -> &str {
        &self.sddl
    }
}

/// Placeholder for capability SID + attributes.
#[derive(Clone, Debug)]
pub struct SidAndAttributes {
    pub sid_sddl: String,
    pub attributes: u32,
}
