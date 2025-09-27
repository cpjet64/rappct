use thiserror::Error;

pub type Result<T> = std::result::Result<T, AcError>;

#[derive(Error, Debug)]
pub enum AcError {
    #[error("Unsupported platform (Windows required)")]
    UnsupportedPlatform,

    #[error("Unsupported: LPAC requires Windows 10 (1703+)")]
    UnsupportedLpac,

    #[error("Unknown capability '{name}'")]
    UnknownCapability {
        name: String,
        suggestion: Option<&'static str>,
    },

    #[error("Access denied on {context}")]
    AccessDenied {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Process launch failed at {stage}: {hint}")]
    LaunchFailed {
        stage: &'static str,
        hint: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Win32 error: {0}")]
    Win32(String),

    #[error("Unimplemented: {0}")]
    Unimplemented(&'static str),
}

#[cfg(test)]
mod tests {
    use super::AcError;
    use std::error::Error;

    #[derive(Debug)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error")
        }
    }

    impl Error for TestError {}

    #[test]
    fn access_denied_retains_context_and_source() {
        let err = AcError::AccessDenied {
            context: "registry".to_string(),
            source: Box::new(TestError),
        };
        assert_eq!(err.to_string(), "Access denied on registry");
        let source = Error::source(&err).expect("source");
        assert!(source.downcast_ref::<TestError>().is_some());
    }

    #[test]
    fn launch_failed_retains_stage_hint_and_source() {
        let err = AcError::LaunchFailed {
            stage: "spawn",
            hint: "pipe",
            source: Box::new(TestError),
        };
        assert_eq!(err.to_string(), "Process launch failed at spawn: pipe");
        let source = Error::source(&err).expect("source");
        assert!(source.downcast_ref::<TestError>().is_some());
    }

    #[test]
    fn unknown_capability_preserves_hint() {
        let err = AcError::UnknownCapability {
            name: "internetClientX".to_string(),
            suggestion: Some("internetClient"),
        };
        assert_eq!(err.to_string(), "Unknown capability 'internetClientX'");
        match err {
            AcError::UnknownCapability { name, suggestion } => {
                assert_eq!(name, "internetClientX");
                assert_eq!(suggestion, Some("internetClient"));
            }
            _ => panic!("expected unknown capability variant"),
        }
    }
}
