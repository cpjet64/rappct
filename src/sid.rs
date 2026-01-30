//! AppContainer SID wrappers (skeleton). In v0.2 this will own PSIDs properly.

use crate::{AcError, Result};

/// AppContainer SID prefix: revision 1, identifier authority 15 (App Package),
/// sub-authority 2 (AppContainer).
const AC_SID_PREFIX: &str = "S-1-15-2-";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct AppContainerSid {
    sddl: String,
}

impl AppContainerSid {
    /// Create from an SDDL string without validation.
    pub fn from_sddl(s: impl Into<String>) -> Self {
        Self { sddl: s.into() }
    }

    /// Create from an SDDL string, validating that it looks like a well-formed
    /// AppContainer SID (`S-1-15-2-<sub-authorities>`).
    ///
    /// Returns `Err(AcError::InvalidSid)` if the string does not start with
    /// the AppContainer SID prefix or contains non-numeric sub-authority
    /// components.
    pub fn try_from_sddl(s: impl Into<String>) -> Result<Self> {
        let sddl: String = s.into();
        if !sddl.starts_with(AC_SID_PREFIX) {
            return Err(AcError::InvalidSid(format!(
                "expected prefix '{AC_SID_PREFIX}', got '{sddl}'"
            )));
        }
        let tail = &sddl[AC_SID_PREFIX.len()..];
        if tail.is_empty() {
            return Err(AcError::InvalidSid(
                "missing sub-authority values after prefix".into(),
            ));
        }
        for part in tail.split('-') {
            if part.is_empty() || part.parse::<u64>().is_err() {
                return Err(AcError::InvalidSid(format!(
                    "invalid sub-authority component '{part}' in '{sddl}'"
                )));
            }
        }
        Ok(Self { sddl })
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

#[cfg(test)]
mod tests {
    use super::AppContainerSid;

    #[test]
    fn try_from_sddl_accepts_valid_sid() {
        let sid = AppContainerSid::try_from_sddl("S-1-15-2-1").unwrap();
        assert_eq!(sid.as_string(), "S-1-15-2-1");
    }

    #[test]
    fn try_from_sddl_accepts_multi_component_sid() {
        let s =
            "S-1-15-2-1430448594-2639229838-973813799-439329657-1197984847-4069167804-277127516";
        let sid = AppContainerSid::try_from_sddl(s).unwrap();
        assert_eq!(sid.as_string(), s);
    }

    #[test]
    fn try_from_sddl_rejects_wrong_prefix() {
        let err = AppContainerSid::try_from_sddl("S-1-5-21-123").unwrap_err();
        assert!(err.to_string().contains("expected prefix"));
    }

    #[test]
    fn try_from_sddl_rejects_empty_tail() {
        let err = AppContainerSid::try_from_sddl("S-1-15-2-").unwrap_err();
        assert!(err.to_string().contains("missing sub-authority"));
    }

    #[test]
    fn try_from_sddl_rejects_non_numeric_component() {
        let err = AppContainerSid::try_from_sddl("S-1-15-2-abc").unwrap_err();
        assert!(err.to_string().contains("invalid sub-authority"));
    }

    #[test]
    fn try_from_sddl_rejects_garbage() {
        assert!(AppContainerSid::try_from_sddl("not-a-sid").is_err());
    }

    #[test]
    fn from_sddl_still_accepts_anything() {
        // Unchecked constructor remains for backwards compatibility
        let sid = AppContainerSid::from_sddl("anything");
        assert_eq!(sid.as_string(), "anything");
    }
}
