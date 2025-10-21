//! AppContainer SID wrappers (skeleton). In v0.2 this will own PSIDs properly.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
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

impl std::fmt::Display for AppContainerSid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.sddl)
    }
}

impl AsRef<str> for AppContainerSid {
    fn as_ref(&self) -> &str {
        self.as_string()
    }
}

/// Placeholder for capability SID + attributes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SidAndAttributes {
    pub sid_sddl: String,
    pub attributes: u32,
}
