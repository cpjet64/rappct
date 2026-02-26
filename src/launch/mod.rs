//! Process launch in AppContainer / LPAC with STARTUPINFOEX and security capabilities.

// legacy launch::attr module no longer used; relying on ffi::attr_list wrappers

#[cfg(windows)]
mod env;

use crate::capability::SecurityCapabilities;
use crate::{AcError, Result};

#[cfg(windows)]
use crate::ffi::attr_list::AttrList as FAttrList;
#[cfg(windows)]
use crate::ffi::handles::{self, Handle as FHandle};
#[cfg(windows)]
use crate::ffi::sec_caps::OwnedSecurityCapabilities;
#[cfg(windows)]
use crate::ffi::sid::OwnedSid;
#[cfg(windows)]
use crate::ffi::wstr::WideString;
#[cfg(windows)]
use core::ffi::c_void;
#[cfg(windows)]
use env::make_wide_block;
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, BorrowedHandle, RawHandle};
#[cfg(windows)]
use std::rc::Rc;

// Use fully-qualified macros (tracing::trace!, etc.) to avoid unused import warnings
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
#[cfg(windows)]
use windows::Win32::Security::SECURITY_ATTRIBUTES;
#[cfg(windows)]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectCpuRateControlInformation,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, EXTENDED_STARTUPINFO_PRESENT,
    PROCESS_INFORMATION, STARTUPINFOEXW,
};
#[cfg(windows)]
use windows::Win32::System::WindowsProgramming::PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT;

#[cfg(windows)]
use windows::core::{PCWSTR, PWSTR};

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
    pub startup_timeout: Option<std::time::Duration>,
    /// Reserved for internal use; use `..Default::default()` when constructing.
    #[cfg(windows)]
    #[doc(hidden)]
    pub extra: LaunchExtra,
}

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct LaunchExtra {
    security_caps: Option<Rc<OwnedSecurityCapabilities>>,
    handle_list: Vec<RawHandle>,
    stdio: StdioOverrides,
}

#[cfg(windows)]
#[derive(Clone, Debug, Default)]
struct StdioOverrides {
    stdin: Option<RawHandle>,
    stdout: Option<RawHandle>,
    stderr: Option<RawHandle>,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let cwd = Some(std::path::PathBuf::from("C:\\Windows\\System32"));
        #[cfg(not(target_os = "windows"))]
        let cwd = None;
        Self {
            exe: std::path::PathBuf::from("C:\\Windows\\System32\\notepad.exe"),
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
    pub fn with_handle_list(mut self, inheritable: &[BorrowedHandle<'_>]) -> Self {
        for h in inheritable {
            self.extra.handle_list.push(h.as_raw_handle());
        }
        self
    }

    #[cfg(windows)]
    pub fn with_stdio_inherit(
        mut self,
        stdin: Option<BorrowedHandle<'_>>,
        stdout: Option<BorrowedHandle<'_>>,
        stderr: Option<BorrowedHandle<'_>>,
    ) -> Self {
        self.extra.stdio.stdin = stdin.map(|h| h.as_raw_handle());
        self.extra.stdio.stdout = stdout.map(|h| h.as_raw_handle());
        self.extra.stdio.stderr = stderr.map(|h| h.as_raw_handle());
        self
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

#[cfg(windows)]
#[derive(Debug)]
pub struct JobGuard(FHandle);
#[cfg(windows)]
impl JobGuard {
    /// Returns the underlying job handle for inspection without taking ownership.
    pub fn as_handle(&self) -> HANDLE {
        self.0.as_win32()
    }
}

/// Job object drop-guard that enables kill-on-close by default.
/// Dropping this guard will terminate attached processes unless explicitly disabled.
#[cfg(windows)]
#[derive(Debug)]
pub struct JobObjectDropGuard {
    handle: FHandle,
    kill_on_drop: bool,
}

#[cfg(windows)]
impl JobObjectDropGuard {
    pub fn new() -> Result<Self> {
        use windows::Win32::System::JobObjects::{
            CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };
        use windows::core::PCWSTR;
        // SAFETY: Create a new job object with no name; returns a live HANDLE on success.
        let hjob = unsafe {
            CreateJobObjectW(None, PCWSTR::null())
                .map_err(|e| AcError::Win32(format!("CreateJobObjectW failed: {e}")))?
        };
        // SAFETY: Zero-initialize the structure per Win32 API requirements.
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: Pass a valid reference to the initialized struct; size matches the type.
        unsafe {
            SetInformationJobObject(
                hjob,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|_| AcError::Win32("SetInformationJobObject(kill_on_close) failed".into()))?;
        }
        Ok(Self {
            // SAFETY: `hjob` is a live HANDLE returned from CreateJobObjectW; take ownership.
            handle: unsafe { FHandle::from_raw(hjob.0 as *mut _) }
                .map_err(|_| AcError::Win32("invalid job handle".into()))?,
            kill_on_drop: true,
        })
    }

    pub fn as_handle(&self) -> HANDLE {
        self.handle.as_win32()
    }

    pub fn assign_process_handle(&self, process: HANDLE) -> Result<()> {
        use windows::Win32::System::JobObjects::AssignProcessToJobObject;
        // SAFETY: Attach the provided process to the job represented by this guard.
        unsafe {
            AssignProcessToJobObject(self.handle.as_win32(), process)
                .map_err(|_| AcError::Win32("AssignProcessToJobObject failed".into()))
        }
    }

    /// Clears the kill-on-close flag so dropping this guard will not
    /// terminate attached processes.
    pub fn disable_kill_on_drop(&mut self) -> Result<()> {
        use windows::Win32::System::JobObjects::{
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };
        // SAFETY: Clear the extended limits by setting a zeroed structure.
        let info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: Pass a valid reference with correct size; clears kill-on-close.
        unsafe {
            SetInformationJobObject(
                self.handle.as_win32(),
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|_| AcError::Win32("SetInformationJobObject(clear) failed".into()))?;
        }
        self.kill_on_drop = false;
        Ok(())
    }
}

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
    custom.reserve(KEYS.len());
    for key in KEYS {
        if custom.iter().any(|(k, _)| k == key) {
            continue;
        }
        if let Some(val) = std::env::var_os(key) {
            custom.push((OsString::from(key), val));
        }
    }
    custom
}

#[cfg(windows)]
#[derive(Default)]
struct InheritList {
    handles: Vec<FHandle>,
    raw: Vec<HANDLE>,
}

#[cfg(windows)]
impl InheritList {
    fn push(&mut self, handle: FHandle) {
        let raw = handle.as_win32();
        self.raw.push(raw);
        self.handles.push(handle);
    }

    fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    fn slice(&self) -> &[HANDLE] {
        &self.raw
    }
}

#[cfg(windows)]
struct LaunchAttributes {
    attr_list: FAttrList,
    security_caps: Rc<OwnedSecurityCapabilities>,
    lpac_policy: Option<Box<u32>>,
    handle_list: Vec<HANDLE>,
}

#[cfg(windows)]
impl LaunchAttributes {
    fn new(
        security_caps: Rc<OwnedSecurityCapabilities>,
        lpac: bool,
        handles: &[HANDLE],
    ) -> Result<Self> {
        let mut attr_count = 1;
        if lpac {
            attr_count += 1;
        }
        if !handles.is_empty() {
            attr_count += 1;
        }

        let mut attr_list = FAttrList::with_capacity(attr_count as u32)?;
        attr_list.set_security_capabilities(security_caps.as_ref())?;

        let mut lpac_policy = None;
        if lpac {
            lpac_policy = Some(Box::new(PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT));
            let policy_ref = lpac_policy.as_ref().unwrap();
            attr_list.set_all_app_packages_policy(policy_ref)?;
        }

        let handle_list = handles.to_vec();
        if !handle_list.is_empty() {
            attr_list.set_handle_list(&handle_list)?;
        }

        Ok(Self {
            attr_list,
            security_caps,
            lpac_policy,
            handle_list,
        })
    }

    fn keep_alive(&self) {
        let _ = (&self.security_caps, &self.lpac_policy, &self.handle_list);
    }

    fn as_mut_ptr(&mut self) -> windows::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST {
        self.keep_alive();
        self.attr_list.as_mut_ptr()
    }
}

#[cfg(windows)]
struct StartUpInfoExGuard {
    info: STARTUPINFOEXW,
    attrs: LaunchAttributes,
}

#[cfg(windows)]
impl StartUpInfoExGuard {
    fn new(mut info: STARTUPINFOEXW, mut attrs: LaunchAttributes) -> Self {
        info.lpAttributeList = attrs.as_mut_ptr();
        Self { info, attrs }
    }

    fn info_mut(&mut self) -> &mut STARTUPINFOEXW {
        self.info.lpAttributeList = self.attrs.as_mut_ptr();
        &mut self.info
    }
}

#[cfg(windows)]
struct StdioSetupResult {
    parent_stdin: Option<FHandle>,
    parent_stdout: Option<FHandle>,
    parent_stderr: Option<FHandle>,
}

#[cfg(windows)]
fn inflate_security_caps(
    sec: &SecurityCapabilities,
    override_caps: Option<Rc<OwnedSecurityCapabilities>>,
) -> Result<Rc<OwnedSecurityCapabilities>> {
    if let Some(sc) = override_caps {
        return Ok(sc);
    }

    let app_sid = OwnedSid::from_sddl(sec.package.as_string())?;
    let mut caps_owned = Vec::with_capacity(sec.caps.len());
    for cap in &sec.caps {
        caps_owned.push(OwnedSid::from_sddl(&cap.sid_sddl)?);
    }

    Ok(Rc::new(OwnedSecurityCapabilities::new(app_sid, caps_owned)))
}

#[cfg(windows)]
fn duplicate_additional_handles(
    handles: &[RawHandle],
    inherit_list: &mut InheritList,
) -> Result<()> {
    for &raw in handles {
        if raw.is_null() {
            continue;
        }
        let dup = handles::duplicate_from_raw(raw, true)?;
        inherit_list.push(dup);
    }
    Ok(())
}

#[cfg(windows)]
fn setup_stdio(
    opts: &LaunchOptions,
    info: &mut STARTUPINFOEXW,
    inherit_list: &mut InheritList,
) -> Result<StdioSetupResult> {
    use windows::Win32::System::Threading::STARTF_USESTDHANDLES;

    let mut parent_stdin: Option<FHandle> = None;
    let mut parent_stdout: Option<FHandle> = None;
    let mut parent_stderr: Option<FHandle> = None;

    match opts.stdio {
        StdioConfig::Inherit => {
            if let Some(raw) = opts.extra.stdio.stdin {
                let dup = handles::duplicate_from_raw(raw, true)?;
                let raw_handle = dup.as_win32();
                info.StartupInfo.hStdInput = raw_handle;
                inherit_list.push(dup);
            }
            if let Some(raw) = opts.extra.stdio.stdout {
                let dup = handles::duplicate_from_raw(raw, true)?;
                let raw_handle = dup.as_win32();
                info.StartupInfo.hStdOutput = raw_handle;
                inherit_list.push(dup);
            }
            if let Some(raw) = opts.extra.stdio.stderr {
                let dup = handles::duplicate_from_raw(raw, true)?;
                let raw_handle = dup.as_win32();
                info.StartupInfo.hStdError = raw_handle;
                inherit_list.push(dup);
            }
            if !inherit_list.is_empty() {
                info.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;
            }
        }
        StdioConfig::Null => {
            use windows::Win32::Foundation::TRUE;
            use windows::Win32::Storage::FileSystem::{
                CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
                FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
            };

            let sa: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: std::ptr::null_mut(),
                bInheritHandle: TRUE,
            };
            let nul = WideString::from_str("NUL");

            // SAFETY: NUL device path is static and ACCESS flags are valid for read/write access.
            let stdin_handle = unsafe {
                CreateFileW(
                    nul.as_pcwstr(),
                    FILE_GENERIC_READ.0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    Some(&sa as *const _),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            }
            .map_err(|_| AcError::LaunchFailed {
                stage: "CreateFileW(NUL)",
                hint: "stdin",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
            // SAFETY: NUL device path is static and ACCESS flags are valid for read/write access.
            let stdout_handle = unsafe {
                CreateFileW(
                    nul.as_pcwstr(),
                    FILE_GENERIC_WRITE.0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    Some(&sa as *const _),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            }
            .map_err(|_| AcError::LaunchFailed {
                stage: "CreateFileW(NUL)",
                hint: "stdout",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
            // SAFETY: NUL device path is static and ACCESS flags are valid for read/write access.
            let stderr_handle = unsafe {
                CreateFileW(
                    nul.as_pcwstr(),
                    FILE_GENERIC_WRITE.0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    Some(&sa as *const _),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            }
            .map_err(|_| AcError::LaunchFailed {
                stage: "CreateFileW(NUL)",
                hint: "stderr",
                source: Box::new(std::io::Error::last_os_error()),
            })?;

            let stdin_owned = handles::from_win32(stdin_handle)?;
            let stdout_owned = handles::from_win32(stdout_handle)?;
            let stderr_owned = handles::from_win32(stderr_handle)?;

            let raw_in = stdin_owned.as_win32();
            let raw_out = stdout_owned.as_win32();
            let raw_err = stderr_owned.as_win32();

            inherit_list.push(stdin_owned);
            inherit_list.push(stdout_owned);
            inherit_list.push(stderr_owned);

            info.StartupInfo.hStdInput = raw_in;
            info.StartupInfo.hStdOutput = raw_out;
            info.StartupInfo.hStdError = raw_err;
            info.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;
        }
        StdioConfig::Pipe => {
            use windows::Win32::Foundation::{HANDLE_FLAGS, TRUE};
            use windows::Win32::System::Pipes::CreatePipe;

            let sa: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: std::ptr::null_mut(),
                bInheritHandle: TRUE,
            };

            let (child_stdin_handle, parent_write) = {
                let (mut read_end, mut write_end) = (HANDLE::default(), HANDLE::default());
                // SAFETY: HANDLE buffers are initialized from `default`; `sa` remains valid during call.
                unsafe { CreatePipe(&mut read_end, &mut write_end, Some(&sa), 0) }.map_err(
                    |_| AcError::LaunchFailed {
                        stage: "CreatePipe(stdin)",
                        hint: "pipe",
                        source: Box::new(std::io::Error::last_os_error()),
                    },
                )?;
                let child = handles::from_win32(read_end)?;
                let parent = handles::from_win32(write_end)?;
                // SAFETY: Handles are valid; clearing inherit on the parent end is intentional.
                let _ = unsafe {
                    SetHandleInformation(parent.as_win32(), HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0))
                };
                (child, parent)
            };

            let (parent_read, child_stdout_handle) = {
                let (mut read_end, mut write_end) = (HANDLE::default(), HANDLE::default());
                // SAFETY: HANDLE buffers are initialized from `default`; `sa` remains valid during call.
                unsafe { CreatePipe(&mut read_end, &mut write_end, Some(&sa), 0) }.map_err(
                    |_| AcError::LaunchFailed {
                        stage: "CreatePipe(stdout)",
                        hint: "pipe",
                        source: Box::new(std::io::Error::last_os_error()),
                    },
                )?;
                let parent = handles::from_win32(read_end)?;
                let child = handles::from_win32(write_end)?;
                // SAFETY: Handles are valid; clearing inherit on the parent end is intentional.
                let _ = unsafe {
                    SetHandleInformation(parent.as_win32(), HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0))
                };
                (parent, child)
            };

            let (parent_err_read, child_stderr_handle) = {
                let (mut read_end, mut write_end) = (HANDLE::default(), HANDLE::default());
                // SAFETY: HANDLE buffers are initialized from `default`; `sa` remains valid during call.
                unsafe { CreatePipe(&mut read_end, &mut write_end, Some(&sa), 0) }.map_err(
                    |_| AcError::LaunchFailed {
                        stage: "CreatePipe(stderr)",
                        hint: "pipe",
                        source: Box::new(std::io::Error::last_os_error()),
                    },
                )?;
                let parent = handles::from_win32(read_end)?;
                let child = handles::from_win32(write_end)?;
                // SAFETY: Handles are valid; clearing inherit on the parent end is intentional.
                let _ = unsafe {
                    SetHandleInformation(parent.as_win32(), HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0))
                };
                (parent, child)
            };

            let raw_in = child_stdin_handle.as_win32();
            let raw_out = child_stdout_handle.as_win32();
            let raw_err = child_stderr_handle.as_win32();

            inherit_list.push(child_stdin_handle);
            inherit_list.push(child_stdout_handle);
            inherit_list.push(child_stderr_handle);

            info.StartupInfo.hStdInput = raw_in;
            info.StartupInfo.hStdOutput = raw_out;
            info.StartupInfo.hStdError = raw_err;
            info.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;

            parent_stdin = Some(parent_write);
            parent_stdout = Some(parent_read);
            parent_stderr = Some(parent_err_read);
        }
    }

    Ok(StdioSetupResult {
        parent_stdin,
        parent_stdout,
        parent_stderr,
    })
}

#[cfg(windows)]
fn make_cmd_args(cmdline: &Option<String>) -> Option<Vec<u16>> {
    cmdline.as_ref().map(|cl| {
        let mut w: Vec<u16> = cl.encode_utf16().collect();
        w.push(0);
        w
    })
}

#[cfg(windows)]
fn launch_impl(sec: &SecurityCapabilities, opts: &LaunchOptions) -> Result<LaunchedIo> {
    if sec.lpac {
        crate::supports_lpac()?;
    }

    let force_env = std::env::var_os("RAPPCT_TEST_FORCE_ENV").is_some();
    let env_block = if let Some(env) = opts.env.as_ref() {
        Some(make_wide_block(env))
    } else if force_env {
        let all: Vec<(OsString, OsString)> = std::env::vars_os().collect();
        Some(make_wide_block(&all))
    } else {
        None
    };

    let exe_w = WideString::from_os_str(opts.exe.as_os_str());
    let mut args_w = make_cmd_args(&opts.cmdline);
    let mut cwd_w = opts
        .cwd
        .as_ref()
        .map(|p| WideString::from_os_str(p.as_os_str()));
    if std::env::var_os("RAPPCT_TEST_NO_CWD").is_some() {
        cwd_w = None;
    }

    let mut inherit_list = InheritList::default();
    let mut startup_info = STARTUPINFOEXW::default();
    startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;

    let stdio = setup_stdio(opts, &mut startup_info, &mut inherit_list)?;

    duplicate_additional_handles(&opts.extra.handle_list, &mut inherit_list)?;

    let security_caps = inflate_security_caps(sec, opts.extra.security_caps.clone())?;
    let attributes = LaunchAttributes::new(security_caps, sec.lpac, inherit_list.slice())?;
    let mut startup_guard = StartUpInfoExGuard::new(startup_info, attributes);
    let info = startup_guard.info_mut();

    let mut pi: PROCESS_INFORMATION = PROCESS_INFORMATION::default();
    let mut flags = EXTENDED_STARTUPINFO_PRESENT;
    if env_block.is_some() {
        flags |= CREATE_UNICODE_ENVIRONMENT;
    }
    if opts.suspended {
        flags |= CREATE_SUSPENDED;
    }

    let _env_bytes = env_block
        .as_ref()
        .map(|block| block.len() * std::mem::size_of::<u16>());

    #[cfg(feature = "tracing")]
    {
        let env_bytes = _env_bytes.unwrap_or(0);
        tracing::trace!(
            "CreateProcessW: exe={:?}, args_present={}, cwd_present={}, inherit_handles={}, flags=0x{:X}, env_bytes={}",
            opts.exe,
            args_w.is_some(),
            cwd_w.is_some(),
            !inherit_list.is_empty(),
            flags.0,
            env_bytes
        );
    }

    let inherit_handles = !inherit_list.is_empty();

    let env_ptr = env_block
        .as_ref()
        .map(|block| block.as_ptr() as *const c_void);
    let cwd_ptr = cwd_w
        .as_mut()
        .map(|c| c.as_pcwstr())
        .unwrap_or(PCWSTR::null());
    let cmd_ptr = args_w.as_mut().map(|v| PWSTR(v.as_mut_ptr()));

    // SAFETY: `CreateProcessW` is called with valid startup/process handles for the current
    // process creation path and with correctly sized pointers to argument/env/cwd buffers.
    let create_res = unsafe {
        CreateProcessW(
            exe_w.as_pcwstr(),
            cmd_ptr,
            None,
            None,
            inherit_handles,
            flags,
            env_ptr,
            cwd_ptr,
            &info.StartupInfo,
            &mut pi,
        )
    };

    if create_res.is_err() {
        return Err(AcError::LaunchFailed {
            stage: "CreateProcessW",
            hint: "launch",
            source: Box::new(std::io::Error::last_os_error()),
        });
    }

    drop(inherit_list);

    let mut job_guard: Option<JobGuard> = None;
    if let Some(limits) = &opts.join_job {
        // SAFETY: `CreateJobObjectW` creates a valid job object handle for the current process.
        let hjob = unsafe { CreateJobObjectW(None, PCWSTR::null()) }
            .map_err(|e| AcError::Win32(format!("CreateJobObjectW failed: {e}")))?;
        if limits.memory_bytes.is_some() || limits.kill_on_job_close {
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            if let Some(bytes) = limits.memory_bytes {
                info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
                info.ProcessMemoryLimit = bytes;
            }
            if limits.kill_on_job_close {
                info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            }
            // SAFETY: `hjob` is a live handle and `info` structure is fully initialized.
            unsafe {
                SetInformationJobObject(
                    hjob,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            }
            .map_err(|_| AcError::LaunchFailed {
                stage: "SetInformationJobObject(ext)",
                hint: "set job limits",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
        }
        if let Some(percent) = limits.cpu_rate_percent {
            let info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
                ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE
                    | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
                Anonymous:
                    windows::Win32::System::JobObjects::JOBOBJECT_CPU_RATE_CONTROL_INFORMATION_0 {
                        CpuRate: percent.clamp(1, 100) * 100,
                    },
            };
            // SAFETY: `hjob` is a live handle and `info` structure is fully initialized.
            unsafe {
                SetInformationJobObject(
                    hjob,
                    JobObjectCpuRateControlInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
                )
            }
            .map_err(|_| AcError::LaunchFailed {
                stage: "SetInformationJobObject(cpu)",
                hint: "set cpu cap",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
        }
        // SAFETY: Both `hjob` and `pi.hProcess` are valid live handles for this operation.
        unsafe { AssignProcessToJobObject(hjob, pi.hProcess) }.map_err(|_| {
            AcError::LaunchFailed {
                stage: "AssignProcessToJobObject",
                hint: "attach child",
                source: Box::new(std::io::Error::last_os_error()),
            }
        })?;
        if limits.kill_on_job_close {
            job_guard = Some(JobGuard(
                handles::from_win32(hjob)
                    .map_err(|_| AcError::Win32("invalid job handle".into()))?,
            ));
        } else {
            // SAFETY: Job handle is not needed after successful assignment when kill-on-close disabled.
            let _ = unsafe { CloseHandle(hjob) };
        }
    }

    // SAFETY: Closing primary thread handle after process creation is safe and expected.
    let _ = unsafe { CloseHandle(pi.hThread) };
    let proc_handle = handles::from_win32(pi.hProcess)
        .map_err(|_| AcError::Win32("invalid process handle".into()))?;

    let StdioSetupResult {
        parent_stdin,
        parent_stdout,
        parent_stderr,
        ..
    } = stdio;

    Ok(LaunchedIo {
        pid: pi.dwProcessId,
        stdin: parent_stdin.map(|h| h.into_file()),
        stdout: parent_stdout.map(|h| h.into_file()),
        stderr: parent_stderr.map(|h| h.into_file()),
        job_guard,
        process: proc_handle,
    })
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
                // This branch is intentionally defensive and remains effectively uncoverable in
                // regular CI: reaching WAIT_FAILED requires a kernel-level handle/state fault
                // (invalidated PROCESS handle, object manager corruption, or API contract break).
                // Normal tests only exercise valid handles and timeout/success outcomes.
                // Safety is established by strict handle ownership (ffi::Handle) plus integration
                // tests validating wait success/timeout behavior on real waitable objects.
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

#[cfg(all(test, windows))]
mod tests {
    use super::{
        InheritList, JobObjectDropGuard, LaunchOptions, LaunchedIo, make_cmd_args, merge_parent_env,
    };
    use crate::capability::CapabilityName;
    use crate::ffi::sec_caps::OwnedSecurityCapabilities;
    use crate::ffi::sid::OwnedSid;
    use std::ffi::OsString;
    use std::os::windows::io::AsRawHandle;
    use std::time::Duration;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Threading::CreateEventW;

    #[test]
    fn merge_parent_env_includes_essentials_if_present() {
        let out = merge_parent_env(vec![(OsString::from("RAPPCT_X"), OsString::from("1"))]);
        assert!(out.iter().any(|(k, v)| k == "RAPPCT_X" && v == "1"));
        if std::env::var_os("SystemRoot").is_some() {
            assert!(out.iter().any(|(k, _)| k == "SystemRoot"));
        }
    }

    #[test]
    fn merge_parent_env_does_not_duplicate_existing_keys() {
        let out = merge_parent_env(vec![(
            OsString::from("SystemRoot"),
            OsString::from("X:\\Custom"),
        )]);
        let matches = out.iter().filter(|(k, _)| k == "SystemRoot").count();
        assert_eq!(matches, 1, "SystemRoot should not be duplicated");
        assert!(
            out.iter()
                .any(|(k, v)| k == "SystemRoot" && v == "X:\\Custom")
        );
    }

    #[test]
    fn make_cmd_args_handles_none_and_some() {
        assert!(make_cmd_args(&None).is_none());

        let args = make_cmd_args(&Some(" /C exit 0".to_string())).expect("expected args");
        assert_eq!(args.last().copied(), Some(0));
        assert_eq!(
            String::from_utf16_lossy(&args[..args.len() - 1]),
            " /C exit 0"
        );
    }

    #[test]
    fn with_env_merge_accumulates_existing_and_added_values() {
        let opts = LaunchOptions {
            env: Some(vec![(OsString::from("A"), OsString::from("1"))]),
            ..Default::default()
        }
        .with_env_merge(&[(OsString::from("B"), OsString::from("2"))]);

        let env = opts.env.expect("env should be populated");
        assert!(env.iter().any(|(k, v)| k == "A" && v == "1"));
        assert!(env.iter().any(|(k, v)| k == "B" && v == "2"));
    }

    #[test]
    fn with_handle_list_and_stdio_inherit_record_raw_handles() {
        let file = std::fs::File::open("C:\\Windows\\System32\\cmd.exe").expect("open fixture");
        // SAFETY: Borrowed handle is valid while `file` remains alive.
        let borrowed =
            unsafe { std::os::windows::io::BorrowedHandle::borrow_raw(file.as_raw_handle()) };
        let opts = LaunchOptions::default()
            .with_handle_list(&[borrowed])
            .with_stdio_inherit(Some(borrowed), Some(borrowed), Some(borrowed));

        assert_eq!(opts.extra.handle_list.len(), 1);
        assert_eq!(opts.extra.handle_list[0], file.as_raw_handle());
        assert_eq!(opts.extra.stdio.stdin, Some(file.as_raw_handle()));
        assert_eq!(opts.extra.stdio.stdout, Some(file.as_raw_handle()));
        assert_eq!(opts.extra.stdio.stderr, Some(file.as_raw_handle()));
    }

    #[test]
    fn launch_options_default_matches_expected_windows_shape() {
        let opts = LaunchOptions::default();
        assert_eq!(
            opts.exe,
            std::path::PathBuf::from("C:\\Windows\\System32\\notepad.exe")
        );
        assert_eq!(
            opts.cwd,
            Some(std::path::PathBuf::from("C:\\Windows\\System32"))
        );
        assert!(opts.cmdline.is_none());
        assert!(opts.env.is_none());
        assert!(matches!(opts.stdio, super::StdioConfig::Inherit));
        assert!(!opts.suspended);
        assert!(opts.join_job.is_none());
        assert!(opts.startup_timeout.is_none());
    }

    #[test]
    fn job_object_drop_guard_disable_flips_state_and_invalid_assign_fails() {
        let mut guard = JobObjectDropGuard::new().expect("create guard");
        assert!(guard.kill_on_drop);
        assert_ne!(guard.as_handle().0, std::ptr::null_mut());
        guard.disable_kill_on_drop().expect("disable kill on drop");
        assert!(!guard.kill_on_drop);

        let err = guard
            .assign_process_handle(HANDLE::default())
            .expect_err("assigning null process handle should fail");
        assert!(err.to_string().contains("AssignProcessToJobObject"));
    }

    #[test]
    fn inherit_list_tracks_pushed_handles_and_raw_slice() {
        let mut list = InheritList::default();
        assert!(list.is_empty());

        // SAFETY: CreateEventW returns an owned event handle on success.
        let event = unsafe { CreateEventW(None, true, false, None).expect("create event") };
        let owned = super::handles::from_win32(event).expect("wrap event handle");
        let raw = owned.as_win32();

        list.push(owned);
        assert!(!list.is_empty());
        assert_eq!(list.slice(), &[raw]);
    }

    #[test]
    fn launched_io_wait_returns_timeout_for_unsignaled_waitable_handle() {
        // SAFETY: CreateEventW returns an owned event handle on success.
        let event = unsafe { CreateEventW(None, true, false, None).expect("create event") };
        let process = super::handles::from_win32(event).expect("wrap event handle");
        let io = LaunchedIo {
            pid: 42,
            stdin: None,
            stdout: None,
            stderr: None,
            job_guard: None,
            process,
        };

        let err = io
            .wait(Some(Duration::from_millis(1)))
            .expect_err("unsignaled handle should time out");
        match err {
            crate::AcError::LaunchFailed { stage, hint, .. } => {
                assert_eq!(stage, "wait");
                assert_eq!(hint, "timeout");
            }
            other => panic!("expected timeout LaunchFailed, got: {other:?}"),
        }
    }

    #[test]
    fn with_security_capabilities_sets_internal_override() {
        let sid = OwnedSid::from_sddl("S-1-15-2-1").expect("owned sid");
        let caps = OwnedSecurityCapabilities::from_catalog(sid, &[CapabilityName::InternetClient])
            .expect("security caps");
        let opts = LaunchOptions::default().with_security_capabilities(caps);
        assert!(opts.extra.security_caps.is_some());
    }
}
