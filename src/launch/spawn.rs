use super::attributes::{
    InheritList, LaunchAttributes, StartUpInfoExGuard, duplicate_additional_handles,
    inflate_security_caps,
};
use super::env::{WideBlock, make_wide_block};
use super::job;
use super::startup;
use super::stdio::{StdioSetupResult, setup_stdio};
use super::{LaunchOptions, LaunchedIo};
use crate::capability::SecurityCapabilities;
use crate::ffi::handles::{self, Handle as FHandle};
use crate::ffi::wstr::WideString;
use crate::{AcError, Result};
use core::ffi::c_void;
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, EXTENDED_STARTUPINFO_PRESENT,
    PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOEXW,
};
use windows::core::{PCWSTR, PWSTR};

struct ProcessInputs {
    env_block: Option<WideBlock>,
    exe_w: WideString,
    args_w: Option<Vec<u16>>,
    cwd_w: Option<WideString>,
}

struct ChildHandles {
    thread: FHandle,
    process: FHandle,
}

pub(super) fn launch_impl(sec: &SecurityCapabilities, opts: &LaunchOptions) -> Result<LaunchedIo> {
    ensure_lpac_supported(sec)?;
    let mut inputs = ProcessInputs::from_options(opts)?;
    let mut inherit_list = InheritList::default();
    let mut startup_info = base_startup_info();

    let stdio = setup_stdio(opts, &mut startup_info, &mut inherit_list)?;
    duplicate_additional_handles(&opts.extra.handle_list, &mut inherit_list)?;

    let security_caps = inflate_security_caps(sec)?;
    let attributes = LaunchAttributes::new(security_caps, sec.lpac, inherit_list.slice())?;
    let mut startup_guard = StartUpInfoExGuard::new(startup_info, attributes);
    let mut pi = PROCESS_INFORMATION::default();
    let flags = creation_flags(opts, inputs.env_block.is_some());
    let inherit_handles = !inherit_list.is_empty();

    create_child_process(
        &mut inputs,
        startup_guard.info_mut(),
        inherit_handles,
        flags,
        &mut pi,
    )?;
    drop(inherit_list);

    let child = process_handles(pi)?;
    let job_guard = job::create_job_guard(opts.join_job.as_ref(), child.process.as_win32())?;
    drop(child.thread);
    startup::wait_for_startup(opts, child.process.as_win32())?;

    Ok(launched_io(pi.dwProcessId, stdio, child.process, job_guard))
}

pub(super) fn make_cmd_args(cmdline: &Option<String>) -> Option<Vec<u16>> {
    cmdline.as_ref().map(|cl| {
        let mut w: Vec<u16> = cl.encode_utf16().collect();
        w.push(0);
        w
    })
}

impl ProcessInputs {
    fn from_options(opts: &LaunchOptions) -> Result<Self> {
        Ok(Self {
            env_block: build_env_block(opts)?,
            exe_w: WideString::from_os_str(opts.exe.as_os_str()),
            args_w: make_cmd_args(&opts.cmdline),
            cwd_w: build_cwd(opts),
        })
    }

    fn env_ptr(&self) -> Option<*const c_void> {
        self.env_block
            .as_ref()
            .map(|block| block.as_ptr() as *const c_void)
    }

    fn cwd_ptr(&mut self) -> PCWSTR {
        self.cwd_w
            .as_mut()
            .map(|c| c.as_pcwstr())
            .unwrap_or(PCWSTR::null())
    }

    fn cmd_ptr(&mut self) -> Option<PWSTR> {
        self.args_w.as_mut().map(|v| PWSTR(v.as_mut_ptr()))
    }
}

fn ensure_lpac_supported(sec: &SecurityCapabilities) -> Result<()> {
    if sec.lpac {
        crate::supports_lpac()
    } else {
        Ok(())
    }
}

pub(super) fn build_env_block(opts: &LaunchOptions) -> Result<Option<WideBlock>> {
    opts.env
        .as_ref()
        .map(|env| make_wide_block(env))
        .transpose()
}

pub(super) fn build_cwd(opts: &LaunchOptions) -> Option<WideString> {
    opts.cwd
        .as_ref()
        .map(|p| WideString::from_os_str(p.as_os_str()))
}

fn base_startup_info() -> STARTUPINFOEXW {
    let mut startup_info = STARTUPINFOEXW::default();
    startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup_info
}

fn creation_flags(opts: &LaunchOptions, has_env_block: bool) -> PROCESS_CREATION_FLAGS {
    let mut flags = EXTENDED_STARTUPINFO_PRESENT;
    if has_env_block {
        flags |= CREATE_UNICODE_ENVIRONMENT;
    }
    if opts.suspended {
        flags |= CREATE_SUSPENDED;
    }
    flags
}

fn create_child_process(
    inputs: &mut ProcessInputs,
    info: &mut STARTUPINFOEXW,
    inherit_handles: bool,
    flags: PROCESS_CREATION_FLAGS,
    pi: &mut PROCESS_INFORMATION,
) -> Result<()> {
    trace_create_process(inputs, inherit_handles, flags);
    let env_ptr = inputs.env_ptr();
    let cwd_ptr = inputs.cwd_ptr();
    let cmd_ptr = inputs.cmd_ptr();

    // SAFETY: CreateProcessW receives buffers that remain alive for the call duration.
    let create_res = unsafe {
        CreateProcessW(
            inputs.exe_w.as_pcwstr(),
            cmd_ptr,
            None,
            None,
            inherit_handles,
            flags,
            env_ptr,
            cwd_ptr,
            &info.StartupInfo,
            pi,
        )
    };

    create_res.map_err(|_| AcError::LaunchFailed {
        stage: "CreateProcessW",
        hint: "launch",
        source: Box::new(std::io::Error::last_os_error()),
    })
}

fn process_handles(pi: PROCESS_INFORMATION) -> Result<ChildHandles> {
    let thread = handles::from_win32(pi.hThread)
        .map_err(|_| AcError::Win32("invalid thread handle".into()))?;
    let process = handles::from_win32(pi.hProcess)
        .map_err(|_| AcError::Win32("invalid process handle".into()))?;
    Ok(ChildHandles { thread, process })
}

fn launched_io(
    pid: u32,
    stdio: StdioSetupResult,
    process: FHandle,
    job_guard: Option<job::JobGuard>,
) -> LaunchedIo {
    LaunchedIo {
        pid,
        stdin: stdio.parent_stdin.map(|h| h.into_file()),
        stdout: stdio.parent_stdout.map(|h| h.into_file()),
        stderr: stdio.parent_stderr.map(|h| h.into_file()),
        job_guard,
        process,
    }
}

#[cfg(feature = "tracing")]
fn trace_create_process(
    inputs: &ProcessInputs,
    inherit_handles: bool,
    flags: PROCESS_CREATION_FLAGS,
) {
    let env_bytes = inputs
        .env_block
        .as_ref()
        .map(|block| block.len() * std::mem::size_of::<u16>())
        .unwrap_or(0);
    tracing::trace!(
        "CreateProcessW: args_present={}, cwd_present={}, inherit_handles={}, flags=0x{:X}, env_bytes={}",
        inputs.args_w.is_some(),
        inputs.cwd_w.is_some(),
        inherit_handles,
        flags.0,
        env_bytes
    );
}

#[cfg(not(feature = "tracing"))]
fn trace_create_process(
    _inputs: &ProcessInputs,
    _inherit_handles: bool,
    _flags: PROCESS_CREATION_FLAGS,
) {
}
