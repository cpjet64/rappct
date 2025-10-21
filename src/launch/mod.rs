//! Process launch in AppContainer / LPAC with STARTUPINFOEX and security capabilities.

#[cfg(windows)]
mod attr;

use crate::capability::SecurityCapabilities;
use crate::{AcError, Result};

#[cfg(windows)]
use crate::launch::attr::AttrList;
#[cfg(windows)]
use crate::util::{LocalFreeGuard, OwnedHandle};

// Use fully-qualified macros (tracing::trace!, etc.) to avoid unused import warnings
#[cfg(windows)]
use windows::core::{PCWSTR, PWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, TRUE};
#[cfg(windows)]
use windows::Win32::Foundation::{SetHandleInformation, HANDLE_FLAG_INHERIT};
#[cfg(windows)]
use windows::Win32::Security::Authorization::ConvertStringSidToSidW;
#[cfg(windows)]
use windows::Win32::Security::{
    PSID, SECURITY_ATTRIBUTES, SECURITY_CAPABILITIES, SID_AND_ATTRIBUTES,
};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(windows)]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectCpuRateControlInformation,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
    JOBOBJECT_CPU_RATE_CONTROL_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE, JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};
#[cfg(windows)]
use windows::Win32::System::Pipes::CreatePipe;
#[cfg(windows)]
use windows::Win32::System::Threading::{
    CreateProcessW, UpdateProcThreadAttribute, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT,
    EXTENDED_STARTUPINFO_PRESENT, PROCESS_INFORMATION,
    PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
    PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, STARTUPINFOEXW, STARTUPINFOW,
};
#[cfg(windows)]
use windows::Win32::System::WindowsProgramming::PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT;

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
        }
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
    pub(crate) process: OwnedHandle,
}

#[cfg(not(windows))]
pub struct LaunchedIo;

#[cfg(windows)]
#[derive(Debug)]
pub struct JobGuard(OwnedHandle);
#[cfg(windows)]
impl JobGuard {
    /// Returns the underlying job handle for inspection without taking ownership.
    pub fn as_handle(&self) -> HANDLE {
        self.0.as_raw()
    }
}

pub fn launch_in_container(_sec: &SecurityCapabilities, _opts: &LaunchOptions) -> Result<Launched> {
    #[cfg(windows)]
    {
        unsafe { launch_impl(_sec, _opts).map(|io| Launched { pid: io.pid }) }
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
fn build_env_block(env: &[(std::ffi::OsString, std::ffi::OsString)]) -> Vec<u16> {
    let mut block: Vec<u16> = Vec::new();
    for (k, v) in env {
        let mut kv = std::ffi::OsString::from(k);
        kv.push("=");
        kv.push(v);
        let mut w: Vec<u16> =
            std::os::windows::ffi::OsStrExt::encode_wide(kv.as_os_str()).collect();
        w.push(0);
        block.extend_from_slice(&w);
    }
    block.push(0);
    block
}

#[cfg(windows)]
const SE_GROUP_ENABLED_CONST: u32 = 0x0000_0004;

#[cfg(windows)]
struct AttributeContext {
    attr_list: AttrList,
    _caps_struct: Box<SECURITY_CAPABILITIES>,
    _cap_attrs: Vec<SID_AND_ATTRIBUTES>,
    _cap_sid_guards: Vec<LocalFreeGuard<std::ffi::c_void>>,
    _package_sid_guard: LocalFreeGuard<std::ffi::c_void>,
    _handle_list: Option<Vec<HANDLE>>,
    _lpac_policy: Option<Box<u32>>,
}

#[cfg(windows)]
impl AttributeContext {
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn new(sec: &SecurityCapabilities, handle_list: Option<Vec<HANDLE>>) -> Result<Self> {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            "setup_attributes: lpac={}, caps_named_count={}, stdio_handles={}",
            sec.lpac,
            sec.caps.len(),
            handle_list.as_ref().map(|s| s.len()).unwrap_or(0)
        );
        #[cfg(feature = "tracing")]
        if sec.caps.is_empty() {
            tracing::trace!("setup_attributes: pure AppContainer (no capabilities)");
        } else {
            for cap in &sec.caps {
                tracing::trace!("setup_attributes: capability SDDL={}", cap.sid_sddl);
            }
        }

        let pkg_w: Vec<u16> = crate::util::to_utf16(sec.package.as_string());
        let mut pkg_psid_raw = PSID(std::ptr::null_mut());
        if ConvertStringSidToSidW(PCWSTR(pkg_w.as_ptr()), &mut pkg_psid_raw).is_err() {
            return Err(AcError::LaunchFailed {
                stage: "ConvertStringSidToSidW(package)",
                hint: "invalid package SID",
                source: Box::new(std::io::Error::last_os_error()),
            });
        }
        let package_sid_guard = LocalFreeGuard::<std::ffi::c_void>::new(pkg_psid_raw.0);
        let package_sid = PSID(package_sid_guard.as_ptr());

        let mut cap_sid_guards: Vec<LocalFreeGuard<std::ffi::c_void>> =
            Vec::with_capacity(sec.caps.len());
        let mut cap_attrs: Vec<SID_AND_ATTRIBUTES> = Vec::with_capacity(sec.caps.len());
        for cap in &sec.caps {
            let sddl_w: Vec<u16> = crate::util::to_utf16(&cap.sid_sddl);
            let mut psid_raw = PSID(std::ptr::null_mut());
            if ConvertStringSidToSidW(PCWSTR(sddl_w.as_ptr()), &mut psid_raw).is_err() {
                return Err(AcError::LaunchFailed {
                    stage: "ConvertStringSidToSidW(capability)",
                    hint: "invalid capability SID",
                    source: Box::new(std::io::Error::last_os_error()),
                });
            }
            let guard = LocalFreeGuard::<std::ffi::c_void>::new(psid_raw.0);
            let sid_handle = PSID(guard.as_ptr());
            cap_attrs.push(SID_AND_ATTRIBUTES {
                Sid: sid_handle,
                Attributes: SE_GROUP_ENABLED_CONST,
            });
            cap_sid_guards.push(guard);
        }

        let mut caps_struct = Box::new(SECURITY_CAPABILITIES {
            AppContainerSid: package_sid,
            Capabilities: if cap_attrs.is_empty() {
                std::ptr::null_mut()
            } else {
                cap_attrs.as_mut_ptr()
            },
            CapabilityCount: cap_attrs.len() as u32,
            Reserved: 0,
        });
        #[cfg(feature = "tracing")]
        tracing::trace!(
            "SECURITY_CAPABILITIES: pkg_sid_ptr={:p}, caps_ptr={:p}, cap_count={}",
            caps_struct.AppContainerSid.0,
            caps_struct.Capabilities,
            caps_struct.CapabilityCount
        );

        let mut attr_count = 1;
        if sec.lpac {
            attr_count += 1;
        }
        if handle_list.is_some() {
            attr_count += 1;
        }
        #[cfg(feature = "tracing")]
        tracing::debug!("AttrList: count={}", attr_count);
        let mut attr_list = AttrList::new(attr_count)?;

        let mut si_ex: STARTUPINFOEXW = std::mem::zeroed();
        si_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        si_ex.lpAttributeList = attr_list.as_mut_ptr();

        let res = UpdateProcThreadAttribute(
            si_ex.lpAttributeList,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            Some(caps_struct.as_mut() as *mut _ as *const std::ffi::c_void),
            std::mem::size_of::<SECURITY_CAPABILITIES>(),
            None,
            None,
        );
        #[cfg(feature = "tracing")]
        tracing::trace!(
            "UpdateProcThreadAttribute(security): attr_list_ptr={:p}, value_ptr={:p}, value_size={}",
            si_ex.lpAttributeList.0,
            caps_struct.as_ref() as *const _,
            std::mem::size_of::<SECURITY_CAPABILITIES>()
        );
        if res.is_err() {
            #[cfg(feature = "tracing")]
            {
                use windows::Win32::Foundation::GetLastError;
                let gle = GetLastError().0;
                tracing::error!("UpdateProcThreadAttribute(security) failed: GLE={}", gle);
            }
            return Err(AcError::LaunchFailed {
                stage: "UpdateProcThreadAttribute(security)",
                hint: "attach SECURITY_CAPABILITIES",
                source: Box::new(std::io::Error::last_os_error()),
            });
        }

        let mut lpac_policy: Option<Box<u32>> = None;
        if sec.lpac {
            lpac_policy = Some(Box::new(PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT));
            let res = UpdateProcThreadAttribute(
                si_ex.lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY as usize,
                lpac_policy
                    .as_mut()
                    .map(|p| &mut **p as *mut u32 as *const std::ffi::c_void),
                std::mem::size_of::<u32>(),
                None,
                None,
            );
            #[cfg(feature = "tracing")]
            tracing::trace!(
                "UpdateProcThreadAttribute(AAPolicy): attr_list_ptr={:p}, policy_ptr={:p}, size={}",
                si_ex.lpAttributeList.0,
                lpac_policy
                    .as_ref()
                    .map(|p| &**p as *const u32)
                    .unwrap_or(std::ptr::null()),
                std::mem::size_of::<u32>()
            );
            if res.is_err() {
                #[cfg(feature = "tracing")]
                {
                    use windows::Win32::Foundation::GetLastError;
                    let gle = GetLastError().0;
                    tracing::error!("UpdateProcThreadAttribute(AAPolicy) failed: GLE={}", gle);
                }
                return Err(AcError::LaunchFailed {
                    stage: "UpdateProcThreadAttribute(lpac)",
                    hint: "opt-out AAP policy",
                    source: Box::new(std::io::Error::last_os_error()),
                });
            }
        }

        if let Some(ref handles) = handle_list {
            let res = UpdateProcThreadAttribute(
                si_ex.lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                Some(handles.as_ptr() as *const std::ffi::c_void),
                std::mem::size_of::<HANDLE>() * handles.len(),
                None,
                None,
            );
            #[cfg(feature = "tracing")]
            {
                tracing::trace!(
                    "UpdateProcThreadAttribute(handles): attr_list_ptr={:p}, count={}, bytes={}",
                    si_ex.lpAttributeList.0,
                    handles.len(),
                    std::mem::size_of::<HANDLE>() * handles.len()
                );
                for (idx, handle) in handles.iter().enumerate() {
                    tracing::trace!("inherit_handle[{}]=0x{:X}", idx, handle.0 as usize);
                }
            }
            if res.is_err() {
                #[cfg(feature = "tracing")]
                {
                    use windows::Win32::Foundation::GetLastError;
                    let gle = GetLastError().0;
                    tracing::error!("UpdateProcThreadAttribute(handles) failed: GLE={}", gle);
                }
                return Err(AcError::LaunchFailed {
                    stage: "UpdateProcThreadAttribute(handles)",
                    hint: "inherit handles",
                    source: Box::new(std::io::Error::last_os_error()),
                });
            }
        }

        Ok(Self {
            attr_list,
            _caps_struct: caps_struct,
            _cap_attrs: cap_attrs,
            _cap_sid_guards: cap_sid_guards,
            _package_sid_guard: package_sid_guard,
            _handle_list: handle_list,
            _lpac_policy: lpac_policy,
        })
    }

    fn as_mut_ptr(&mut self) -> windows::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST {
        self.attr_list.as_mut_ptr()
    }
}

#[cfg(windows)]
impl Drop for AttributeContext {
    fn drop(&mut self) {
        // Guards drop automatically
    }
}

#[cfg(windows)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn make_cmd_args(cmdline: &Option<String>) -> Option<Vec<u16>> {
    cmdline.as_ref().map(|cl| {
        let mut w: Vec<u16> = cl.encode_utf16().collect();
        w.push(0);
        w
    })
}

#[cfg(windows)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn launch_impl(sec: &SecurityCapabilities, opts: &LaunchOptions) -> Result<LaunchedIo> {
    if sec.lpac {
        crate::supports_lpac()?;
    }

    // Environment
    let env_block = opts.env.as_ref().map(|e| build_env_block(e));

    // Command
    let exe_w: Vec<u16> = crate::util::to_utf16_os(opts.exe.as_os_str());
    let mut args_w = make_cmd_args(&opts.cmdline);
    let cwd_w = opts
        .cwd
        .as_ref()
        .map(|p| crate::util::to_utf16_os(p.as_os_str()));

    // stdio: Inherit/Null or Pipes
    let mut si_ex: STARTUPINFOEXW = std::mem::zeroed();
    si_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;

    let mut child_stdin = HANDLE::default();
    let mut child_stdout = HANDLE::default();
    let mut child_stderr = HANDLE::default();
    let mut parent_stdin: Option<OwnedHandle> = None;
    let mut parent_stdout: Option<OwnedHandle> = None;
    let mut parent_stderr: Option<OwnedHandle> = None;
    let mut inherit_handles = false;

    match opts.stdio {
        StdioConfig::Inherit => {}
        StdioConfig::Null => {
            let mut sa: SECURITY_ATTRIBUTES = std::mem::zeroed();
            sa.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
            sa.bInheritHandle = TRUE;
            let nul: Vec<u16> = crate::util::to_utf16("NUL");
            let h_in = CreateFileW(
                PCWSTR(nul.as_ptr()),
                FILE_GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                Some(&sa as *const _),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .map_err(|_| AcError::LaunchFailed {
                stage: "CreateFileW(NUL)",
                hint: "stdin",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
            let h_out = CreateFileW(
                PCWSTR(nul.as_ptr()),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                Some(&sa as *const _),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .map_err(|_| AcError::LaunchFailed {
                stage: "CreateFileW(NUL)",
                hint: "stdout",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
            let h_err = CreateFileW(
                PCWSTR(nul.as_ptr()),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                Some(&sa as *const _),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .map_err(|_| AcError::LaunchFailed {
                stage: "CreateFileW(NUL)",
                hint: "stderr",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
            si_ex.StartupInfo.hStdInput = h_in;
            si_ex.StartupInfo.hStdOutput = h_out;
            si_ex.StartupInfo.hStdError = h_err;
            si_ex.StartupInfo.dwFlags |= windows::Win32::System::Threading::STARTF_USESTDHANDLES;
            inherit_handles = true;
        }
        StdioConfig::Pipe => {
            let mut sa: SECURITY_ATTRIBUTES = std::mem::zeroed();
            sa.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
            sa.bInheritHandle = TRUE;
            let (mut r_in, mut w_in) = (HANDLE::default(), HANDLE::default());
            CreatePipe(&mut r_in, &mut w_in, Some(&sa), 0).map_err(|_| AcError::LaunchFailed {
                stage: "CreatePipe(stdin)",
                hint: "pipe",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
            let (mut r_out, mut w_out) = (HANDLE::default(), HANDLE::default());
            CreatePipe(&mut r_out, &mut w_out, Some(&sa), 0).map_err(|_| {
                AcError::LaunchFailed {
                    stage: "CreatePipe(stdout)",
                    hint: "pipe",
                    source: Box::new(std::io::Error::last_os_error()),
                }
            })?;
            let (mut r_err, mut w_err) = (HANDLE::default(), HANDLE::default());
            CreatePipe(&mut r_err, &mut w_err, Some(&sa), 0).map_err(|_| {
                AcError::LaunchFailed {
                    stage: "CreatePipe(stderr)",
                    hint: "pipe",
                    source: Box::new(std::io::Error::last_os_error()),
                }
            })?;
            // make correct ends inheritable (child), parent ends non-inheritable
            let _ = SetHandleInformation(
                w_in,
                HANDLE_FLAG_INHERIT.0,
                windows::Win32::Foundation::HANDLE_FLAGS(0),
            );
            let _ = SetHandleInformation(
                r_out,
                HANDLE_FLAG_INHERIT.0,
                windows::Win32::Foundation::HANDLE_FLAGS(0),
            );
            let _ = SetHandleInformation(
                r_err,
                HANDLE_FLAG_INHERIT.0,
                windows::Win32::Foundation::HANDLE_FLAGS(0),
            );
            si_ex.StartupInfo.hStdInput = r_in; // child reads stdin
            si_ex.StartupInfo.hStdOutput = w_out; // child writes stdout
            si_ex.StartupInfo.hStdError = w_err; // child writes stderr
            si_ex.StartupInfo.dwFlags |= windows::Win32::System::Threading::STARTF_USESTDHANDLES;
            inherit_handles = true;
            child_stdin = r_in;
            child_stdout = w_out;
            child_stderr = w_err;
            parent_stdin = Some(unsafe { OwnedHandle::from_raw(w_in) });
            parent_stdout = Some(unsafe { OwnedHandle::from_raw(r_out) });
            parent_stderr = Some(unsafe { OwnedHandle::from_raw(r_err) });
        }
    }

    // Attach attributes (security caps + lpac + handle list for pipes)
    let handles_for_attr: Option<Vec<HANDLE>> =
        if inherit_handles && matches!(opts.stdio, StdioConfig::Pipe) {
            Some(vec![child_stdin, child_stdout, child_stderr])
        } else {
            None
        };
    let mut attr_ctx = AttributeContext::new(sec, handles_for_attr)?;
    si_ex.lpAttributeList = attr_ctx.as_mut_ptr();

    // CreateProcessW
    let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
    let mut flags = EXTENDED_STARTUPINFO_PRESENT;
    if env_block.is_some() {
        flags |= CREATE_UNICODE_ENVIRONMENT;
    }
    if opts.suspended {
        flags |= CREATE_SUSPENDED;
    }
    #[cfg(feature = "tracing")]
    {
        let inherit_handles_dbg = inherit_handles;
        let env_bytes = env_block.as_ref().map(|b| b.len()).unwrap_or(0);
        tracing::trace!(
            "CreateProcessW: exe={:?}, args_present={}, cwd_present={}, lpAttributeList={:p}, inherit_handles={}, flags=0x{:X}, env_bytes={}",
            opts.exe,
            args_w.as_ref().map(|v| v.len()).is_some(),
            cwd_w.as_ref().is_some(),
            si_ex.lpAttributeList.0,
            inherit_handles_dbg,
            flags.0,
            env_bytes
        );
    }
    let ok = CreateProcessW(
        PCWSTR(exe_w.as_ptr()),
        args_w.as_mut().map(|v| PWSTR(v.as_mut_ptr())),
        None,
        None,
        inherit_handles,
        flags,
        env_block
            .as_ref()
            .map(|b| b.as_ptr() as *const std::ffi::c_void),
        cwd_w
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null()),
        &mut si_ex as *mut STARTUPINFOEXW as *mut STARTUPINFOW,
        &mut pi,
    )
    .is_ok();

    if !ok {
        #[cfg(feature = "tracing")]
        {
            use windows::Win32::Foundation::GetLastError;
            let gle = unsafe { GetLastError().0 };
            tracing::error!("CreateProcessW failed: GLE={}", gle);
        }
        // Optional debug log without requiring a tracing subscriber
        if std::env::var_os("RAPPCT_DEBUG_LAUNCH").is_some() {
            use windows::Win32::Foundation::GetLastError;
            let gle = unsafe { GetLastError().0 };
            eprintln!(
                "[rappct] CreateProcessW failed: GLE={} exe={:?} args_present={} cwd_present={} inherit_handles={} flags=0x{:X}",
                gle,
                opts.exe,
                args_w.as_ref().map(|v| v.len()).is_some(),
                cwd_w.as_ref().is_some(),
                inherit_handles,
                flags.0,
            );
        }
        // close child ends if any
        if inherit_handles {
            if child_stdin != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(child_stdin);
            }
            if child_stdout != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(child_stdout);
            }
            if child_stderr != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(child_stderr);
            }
        }
        return Err(AcError::LaunchFailed {
            stage: "CreateProcessW",
            hint: "extended startup with AC/LPAC",
            source: Box::new(std::io::Error::last_os_error()),
        });
    }

    drop(attr_ctx); // release attribute resources once the process is created

    // parent closes child ends
    if inherit_handles {
        if child_stdin != INVALID_HANDLE_VALUE {
            let _ = CloseHandle(child_stdin);
        }
        if child_stdout != INVALID_HANDLE_VALUE {
            let _ = CloseHandle(child_stdout);
        }
        if child_stderr != INVALID_HANDLE_VALUE {
            let _ = CloseHandle(child_stderr);
        }
    }

    // Optional job limits
    let mut job_guard: Option<JobGuard> = None;
    if let Some(limits) = &opts.join_job {
        let hjob = CreateJobObjectW(None, PCWSTR::null())
            .map_err(|e| AcError::Win32(format!("CreateJobObjectW failed: {e}")))?;
        if limits.memory_bytes.is_some() || limits.kill_on_job_close {
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            if let Some(bytes) = limits.memory_bytes {
                info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
                info.ProcessMemoryLimit = bytes;
            }
            if limits.kill_on_job_close {
                info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            }
            SetInformationJobObject(
                hjob,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|_| AcError::LaunchFailed {
                stage: "SetInformationJobObject(ext)",
                hint: "set job limits",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
        }
        if let Some(percent) = limits.cpu_rate_percent {
            let mut info: JOBOBJECT_CPU_RATE_CONTROL_INFORMATION = std::mem::zeroed();
            info.ControlFlags =
                JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP;
            info.Anonymous.CpuRate = percent.clamp(1, 100) * 100;
            SetInformationJobObject(
                hjob,
                JobObjectCpuRateControlInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
            )
            .map_err(|_| AcError::LaunchFailed {
                stage: "SetInformationJobObject(cpu)",
                hint: "set cpu cap",
                source: Box::new(std::io::Error::last_os_error()),
            })?;
        }
        AssignProcessToJobObject(hjob, pi.hProcess).map_err(|_| AcError::LaunchFailed {
            stage: "AssignProcessToJobObject",
            hint: "attach child",
            source: Box::new(std::io::Error::last_os_error()),
        })?;
        if limits.kill_on_job_close {
            job_guard = Some(JobGuard(OwnedHandle(hjob)));
        } else {
            let _ = CloseHandle(hjob);
        }
    }

    let _ = CloseHandle(pi.hThread);
    let proc_handle = OwnedHandle(pi.hProcess);
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
    unsafe { launch_impl(sec, opts) }
}

#[cfg(windows)]
impl LaunchedIo {
    pub fn wait(self, timeout: Option<std::time::Duration>) -> Result<u32> {
        use windows::Win32::Foundation::{STILL_ACTIVE, WAIT_FAILED, WAIT_TIMEOUT};
        use windows::Win32::System::Threading::{
            GetExitCodeProcess, WaitForSingleObject, INFINITE,
        };
        unsafe {
            let ms = timeout
                .map(|d| d.as_millis().min(u128::from(u32::MAX)) as u32)
                .unwrap_or(INFINITE);
            let r = WaitForSingleObject(self.process.as_raw(), ms);
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
            GetExitCodeProcess(self.process.as_raw(), &mut code)
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
