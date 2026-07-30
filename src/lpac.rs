use crate::{AcError, Result};

const MINIMUM_LPAC_MAJOR: u32 = 10;
const MINIMUM_LPAC_BUILD: u32 = 15_063;

/// Returns `Ok(())` if LPAC is supported on this OS (Windows 10 1703+).
pub fn supports_lpac() -> Result<()> {
    #[cfg(windows)]
    {
        let version = query_windows_version()?;
        evaluate_lpac_support(version.major, version.build)
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

fn evaluate_lpac_support(major: u32, build: u32) -> Result<()> {
    if major < MINIMUM_LPAC_MAJOR || build < MINIMUM_LPAC_BUILD {
        return Err(AcError::UnsupportedLpac);
    }
    Ok(())
}

#[cfg(windows)]
#[repr(C)]
struct OsVersionInfoW {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform: u32,
    csd: [u16; 128],
}

#[cfg(windows)]
fn query_windows_version() -> Result<OsVersionInfoW> {
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
    }

    let mut version = OsVersionInfoW {
        size: std::mem::size_of::<OsVersionInfoW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform: 0,
        csd: [0; 128],
    };
    // SAFETY: `version` is a valid writable structure for the duration of the call.
    let status = unsafe { RtlGetVersion(&mut version) };
    if status != 0 {
        return Err(AcError::UnsupportedLpac);
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_windows_versions_before_lpac() {
        assert!(matches!(
            evaluate_lpac_support(6, 15_063),
            Err(AcError::UnsupportedLpac)
        ));
        assert!(matches!(
            evaluate_lpac_support(10, 15_062),
            Err(AcError::UnsupportedLpac)
        ));
    }

    #[test]
    fn accepts_lpac_baseline_and_newer_versions() {
        assert!(evaluate_lpac_support(10, 15_063).is_ok());
        assert!(evaluate_lpac_support(11, 26_100).is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn reports_unsupported_platform_off_windows() {
        assert!(matches!(supports_lpac(), Err(AcError::UnsupportedPlatform)));
    }
}
