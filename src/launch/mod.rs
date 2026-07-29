//! Process launch in AppContainer / LPAC with STARTUPINFOEX and security capabilities.

// legacy launch::attr module no longer used; relying on ffi::attr_list wrappers

#[cfg(windows)]
mod attributes;
#[cfg(windows)]
mod env;
#[cfg(windows)]
mod job;
#[cfg(windows)]
mod spawn;
#[cfg(windows)]
mod startup;
#[cfg(windows)]
mod stdio;
#[cfg(all(test, windows))]
mod tests;

use crate::capability::SecurityCapabilities;
use crate::{AcError, Result};

#[cfg(windows)]
use crate::ffi::handles::{self, Handle as FHandle};
#[cfg(windows)]
use crate::ffi::sec_caps::OwnedSecurityCapabilities;
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::io::BorrowedHandle;
#[cfg(windows)]
use std::rc::Rc;

#[cfg(windows)]
pub use job::{JobGuard, JobObjectDropGuard};
#[cfg(windows)]
use spawn::launch_impl;
#[cfg(all(test, windows))]
use spawn::make_cmd_args;

#[derive(Clone, Copy, Debug)]
pub enum StdioConfig {
    Inherit,
    Null,
    Pipe,
}

#[derive(Clone, Debug, Default)]
pub struct JobLimits {
    pub memory_bytes: Option<usize>,
    pub cpu_rate_percent: Option<u32>,
    pub kill_on_job_close: bool,
}

#[derive(Clone, Debug)]
pub struct LaunchOptions {
    pub exe: std::path::PathBuf,
    pub cmdline: Option<String>,
    pub cwd: Option<std::path::PathBuf>,
    pub env: Option<Vec<(std::ffi::OsString, std::ffi::OsString)>>,
    pub stdio: StdioConfig,
    pub suspended: bool,
    pub join_job: Option<JobLimits>,
    /// Maximum wait for child startup to reach input-idle.
    /// Ignored when `suspended` is `true`, because the child thread is not running yet.
    pub startup_timeout: Option<std::time::Duration>,
    /// Reserved for internal use; use `..Default::default()` when constructing.
    #[cfg(windows)]
    #[doc(hidden)]
    pub extra: LaunchExtra,
}

#[cfg(windows)]
#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct LaunchExtra {
    security_caps: Option<Rc<OwnedSecurityCapabilities>>,
    handle_list: Vec<Rc<FHandle>>,
    stdio: StdioOverrides,
}

#[cfg(windows)]
#[derive(Clone, Debug, Default)]
struct StdioOverrides {
    stdin: Option<Rc<FHandle>>,
    stdout: Option<Rc<FHandle>>,
    stderr: Option<Rc<FHandle>>,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let cwd = Some(std::path::PathBuf::from("C:\\Windows\\System32"));
        #[cfg(not(target_os = "windows"))]
        let cwd = None;
        #[cfg(target_os = "windows")]
        let exe = std::path::PathBuf::from("C:\\Windows\\System32\\notepad.exe");
        #[cfg(not(target_os = "windows"))]
        let exe = std::path::PathBuf::new();
        Self {
            exe,
            cmdline: None,
            cwd,
            env: None,
            stdio: StdioConfig::Inherit,
            suspended: false,
            join_job: None,
            startup_timeout: None,
            #[cfg(windows)]
            extra: LaunchExtra::default(),
        }
    }
}

impl LaunchOptions {
    #[cfg(windows)]
    #[allow(dead_code)]
    pub(crate) fn with_security_capabilities(mut self, sc: OwnedSecurityCapabilities) -> Self {
        self.extra.security_caps = Some(Rc::new(sc));
        self
    }

    #[cfg(windows)]
    pub fn with_handle_list(self, inheritable: &[BorrowedHandle<'_>]) -> Self {
        self.try_with_handle_list(inheritable)
            .expect("failed to duplicate inheritable handle list")
    }

    #[cfg(windows)]
    pub fn try_with_handle_list(mut self, inheritable: &[BorrowedHandle<'_>]) -> Result<Self> {
        for h in inheritable {
            let duplicated = handles::duplicate_handle(*h, true)?;
            self.extra.handle_list.push(Rc::new(duplicated));
        }
        Ok(self)
    }

    #[cfg(windows)]
    pub fn with_stdio_inherit(
        self,
        stdin: Option<BorrowedHandle<'_>>,
        stdout: Option<BorrowedHandle<'_>>,
        stderr: Option<BorrowedHandle<'_>>,
    ) -> Self {
        self.try_with_stdio_inherit(stdin, stdout, stderr)
            .expect("failed to duplicate inherited stdio handle")
    }

    #[cfg(windows)]
    pub fn try_with_stdio_inherit(
        mut self,
        stdin: Option<BorrowedHandle<'_>>,
        stdout: Option<BorrowedHandle<'_>>,
        stderr: Option<BorrowedHandle<'_>>,
    ) -> Result<Self> {
        self.extra.stdio.stdin = Self::duplicate_stdio_handle(stdin)?;
        self.extra.stdio.stdout = Self::duplicate_stdio_handle(stdout)?;
        self.extra.stdio.stderr = Self::duplicate_stdio_handle(stderr)?;
        Ok(self)
    }

    #[cfg(windows)]
    fn duplicate_stdio_handle(handle: Option<BorrowedHandle<'_>>) -> Result<Option<Rc<FHandle>>> {
        let Some(handle) = handle else {
            return Ok(None);
        };
        let duplicated = handles::duplicate_handle(handle, true)?;
        Ok(Some(Rc::new(duplicated)))
    }

    pub fn with_env_merge(mut self, add: &[(OsString, OsString)]) -> Self {
        let mut env = self.env.take().unwrap_or_default();
        env.extend(add.iter().cloned());
        self.env = Some(merge_parent_env(env));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Launched {
    pub pid: u32,
}

#[cfg(windows)]
#[derive(Debug)]
pub struct LaunchedIo {
    pub pid: u32,
    pub stdin: Option<std::fs::File>,
    pub stdout: Option<std::fs::File>,
    pub stderr: Option<std::fs::File>,
    pub job_guard: Option<JobGuard>,
    pub(crate) process: FHandle,
}

#[cfg(not(windows))]
pub struct LaunchedIo;

pub fn launch_in_container(_sec: &SecurityCapabilities, _opts: &LaunchOptions) -> Result<Launched> {
    #[cfg(windows)]
    {
        launch_impl(_sec, _opts).map(|io| Launched { pid: io.pid })
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

/// Merge caller-supplied env vars with essential Windows variables.
/// When passing a custom environment to `CreateProcessW`, the parent env is
/// fully replaced. Including these keys avoids common failures (e.g., error 203).
pub fn merge_parent_env(mut custom: Vec<(OsString, OsString)>) -> Vec<(OsString, OsString)> {
    const KEYS: &[&str] = &[
        "SystemRoot",
        "windir",
        "ComSpec",
        "PATHEXT",
        "TEMP",
        "TMP",
        "PATH",
    ];

    fn key_matches(lhs: &OsString, rhs: &str) -> bool {
        lhs.to_string_lossy().eq_ignore_ascii_case(rhs)
    }

    fn merge_env_entry(
        merged: &mut Vec<(OsString, OsString)>,
        key: OsString,
        value: OsString,
        overwrite: bool,
    ) {
        if let Some((existing_key, existing_value)) = merged
            .iter_mut()
            .find(|(k, _)| key_matches(k, key.to_string_lossy().as_ref()))
        {
            if overwrite {
                *existing_key = key;
                *existing_value = value;
            }
            return;
        }
        merged.push((key, value));
    }

    let mut merged = Vec::with_capacity(custom.len() + KEYS.len());
    for (key, value) in custom.drain(..) {
        merge_env_entry(&mut merged, key, value, true);
    }
    for key in KEYS {
        if let Some(val) = std::env::var_os(key) {
            merge_env_entry(&mut merged, OsString::from(key), val, false);
        }
    }
    merged
}

#[cfg(windows)]
pub fn launch_in_container_with_io(
    sec: &SecurityCapabilities,
    opts: &LaunchOptions,
) -> Result<LaunchedIo> {
    launch_impl(sec, opts)
}

#[cfg(windows)]
impl LaunchedIo {
    pub fn wait(self, timeout: Option<std::time::Duration>) -> Result<u32> {
        use windows::Win32::Foundation::{STILL_ACTIVE, WAIT_FAILED, WAIT_TIMEOUT};
        use windows::Win32::System::Threading::{
            GetExitCodeProcess, INFINITE, WaitForSingleObject,
        };
        // SAFETY: Wait and query exit code for a live process handle; convert duration to ms.
        unsafe {
            let ms = timeout
                .map(|d| d.as_millis().min(u128::from(u32::MAX)) as u32)
                .unwrap_or(INFINITE);
            let r = WaitForSingleObject(self.process.as_win32(), ms);
            if r == WAIT_FAILED {
                return Err(AcError::Win32("WaitForSingleObject failed".into()));
            }
            if r == WAIT_TIMEOUT {
                return Err(AcError::LaunchFailed {
                    stage: "wait",
                    hint: "timeout",
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "wait timeout",
                    )),
                });
            }
            let mut code: u32 = STILL_ACTIVE.0 as u32;
            GetExitCodeProcess(self.process.as_win32(), &mut code)
                .map_err(|_| AcError::Win32("GetExitCodeProcess failed".into()))?;
            Ok(code)
        }
    }
}

#[cfg(not(windows))]
pub fn launch_in_container_with_io(
    _sec: &SecurityCapabilities,
    _opts: &LaunchOptions,
) -> Result<LaunchedIo> {
    Err(AcError::UnsupportedPlatform)
}
