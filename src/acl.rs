//! ACL helpers for files/directories and registry keys (DACL grant).

use crate::Result;
use crate::sid::AppContainerSid;

#[cfg(windows)]
mod grant;
#[cfg(test)]
mod tests;

/// ACE inheritance flags for directory ACL grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AceInheritance(pub u32);

impl AceInheritance {
    /// Inherited by child containers and objects (default for directories).
    /// Equivalent to `CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE`.
    pub const SUB_CONTAINERS_AND_OBJECTS: Self = Self(0x3);
    /// Inherited by child containers only (`CONTAINER_INHERIT_ACE`).
    pub const SUB_CONTAINERS_ONLY: Self = Self(0x2);
    /// Inherited by child objects only (`OBJECT_INHERIT_ACE`).
    pub const OBJECTS_ONLY: Self = Self(0x1);
    /// No inheritance. The ACE applies only to the directory itself.
    pub const NONE: Self = Self(0x0);
}

/// Target resource for granting AppContainer or capability access.
///
/// Notes:
/// - `RegistryKey` supports only `HKCU` and `HKLM` roots (case-insensitive shorthands
///   `HKCU\\`/`HKLM\\` and full names `HKEY_CURRENT_USER\\`/`HKEY_LOCAL_MACHINE\\`).
/// - `Directory` uses [`AceInheritance::SUB_CONTAINERS_AND_OBJECTS`] by default.
///   Use `DirectoryCustom` to override the inheritance flags.
#[derive(Clone, Debug)]
pub enum ResourcePath {
    File(std::path::PathBuf),
    Directory(std::path::PathBuf),
    /// Directory with custom ACE inheritance flags.
    DirectoryCustom(std::path::PathBuf, AceInheritance),
    RegistryKey(String),
}

#[derive(Clone, Copy, Debug)]
pub struct AccessMask(pub u32);

impl AccessMask {
    /// Full (generic) access commonly used in examples/tests.
    pub const GENERIC_ALL: Self = Self(0x001F_01FF);

    /// FILE_GENERIC_READ access mask.
    #[cfg(windows)]
    pub const FILE_GENERIC_READ: Self =
        Self(windows::Win32::Storage::FileSystem::FILE_GENERIC_READ.0);
    /// FILE_GENERIC_WRITE access mask.
    #[cfg(windows)]
    pub const FILE_GENERIC_WRITE: Self =
        Self(windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE.0);

    /// FILE_GENERIC_READ (non-Windows fallback value).
    #[cfg(not(windows))]
    pub const FILE_GENERIC_READ: Self = Self(0x0001_20089);
    /// FILE_GENERIC_WRITE (non-Windows fallback value).
    #[cfg(not(windows))]
    pub const FILE_GENERIC_WRITE: Self = Self(0x0001_20116);
}

/// Grants the specified access to the AppContainer package SID on the target resource.
#[cfg_attr(not(windows), allow(unused_variables))]
pub fn grant_to_package(
    target: ResourcePath,
    sid: &AppContainerSid,
    access: AccessMask,
) -> Result<()> {
    #[cfg(windows)]
    {
        grant::grant_sid_access(target, sid.as_string(), access.0)
    }
    #[cfg(not(windows))]
    {
        Err(crate::AcError::UnsupportedPlatform)
    }
}

/// Grants the specified access to a capability SID on the target resource.
#[cfg_attr(not(windows), allow(unused_variables))]
pub fn grant_to_capability(
    target: ResourcePath,
    cap_sid_sddl: &str,
    access: AccessMask,
) -> Result<()> {
    #[cfg(windows)]
    {
        grant::grant_sid_access(target, cap_sid_sddl, access.0)
    }
    #[cfg(not(windows))]
    {
        Err(crate::AcError::UnsupportedPlatform)
    }
}
