use super::attributes::InheritList;
use super::{LaunchOptions, StdioConfig};
use crate::ffi::handles::{self, Handle as FHandle};
use crate::ffi::wstr::WideString;
use crate::{AcError, Result};
use std::sync::Arc;
use windows::Win32::Foundation::{
    HANDLE, HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation, TRUE,
};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{STARTF_USESTDHANDLES, STARTUPINFOEXW};

#[derive(Default)]
pub(super) struct StdioSetupResult {
    pub(super) parent_stdin: Option<FHandle>,
    pub(super) parent_stdout: Option<FHandle>,
    pub(super) parent_stderr: Option<FHandle>,
}

pub(super) fn setup_stdio(
    opts: &LaunchOptions,
    info: &mut STARTUPINFOEXW,
    inherit_list: &mut InheritList,
) -> Result<StdioSetupResult> {
    match opts.stdio {
        StdioConfig::Inherit => {
            setup_inherited_stdio(opts, info, inherit_list)?;
            Ok(StdioSetupResult::default())
        }
        StdioConfig::Null => {
            setup_null_stdio(info, inherit_list)?;
            Ok(StdioSetupResult::default())
        }
        StdioConfig::Pipe => setup_piped_stdio(info, inherit_list),
    }
}

fn setup_inherited_stdio(
    opts: &LaunchOptions,
    info: &mut STARTUPINFOEXW,
    inherit_list: &mut InheritList,
) -> Result<()> {
    let mut inherited_any = false;
    inherited_any |= inherit_override(
        opts.extra.stdio.stdin.clone(),
        &mut info.StartupInfo.hStdInput,
        inherit_list,
    )?;
    inherited_any |= inherit_override(
        opts.extra.stdio.stdout.clone(),
        &mut info.StartupInfo.hStdOutput,
        inherit_list,
    )?;
    inherited_any |= inherit_override(
        opts.extra.stdio.stderr.clone(),
        &mut info.StartupInfo.hStdError,
        inherit_list,
    )?;
    if inherited_any {
        info.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;
    }
    Ok(())
}

fn inherit_override(
    handle: Option<Arc<FHandle>>,
    target: &mut HANDLE,
    inherit_list: &mut InheritList,
) -> Result<bool> {
    let Some(handle) = handle else {
        return Ok(false);
    };

    *target = handle.as_win32();
    inherit_list.push_shared(handle);
    Ok(true)
}

fn setup_null_stdio(info: &mut STARTUPINFOEXW, inherit_list: &mut InheritList) -> Result<()> {
    let stdin = open_null(FILE_GENERIC_READ.0, "stdin")?;
    let stdout = open_null(FILE_GENERIC_WRITE.0, "stdout")?;
    let stderr = open_null(FILE_GENERIC_WRITE.0, "stderr")?;
    attach_child_stdio(info, inherit_list, stdin, stdout, stderr);
    Ok(())
}

fn setup_piped_stdio(
    info: &mut STARTUPINFOEXW,
    inherit_list: &mut InheritList,
) -> Result<StdioSetupResult> {
    let (child_stdin, parent_stdin) = pipe_for_child("stdin", true)?;
    let (child_stdout, parent_stdout) = pipe_for_child("stdout", false)?;
    let (child_stderr, parent_stderr) = pipe_for_child("stderr", false)?;

    attach_child_stdio(info, inherit_list, child_stdin, child_stdout, child_stderr);

    Ok(StdioSetupResult {
        parent_stdin: Some(parent_stdin),
        parent_stdout: Some(parent_stdout),
        parent_stderr: Some(parent_stderr),
    })
}

fn open_null(access: u32, hint: &'static str) -> Result<FHandle> {
    let sa = inheritable_security_attributes();
    let nul = WideString::from_str("NUL");
    // SAFETY: NUL device path is static and access/share/disposition flags are valid.
    let handle = unsafe {
        CreateFileW(
            nul.as_pcwstr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            Some(&sa as *const _),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
    .map_err(|_| launch_stdio_error("CreateFileW(NUL)", hint))?;
    handles::from_win32(handle)
}

fn pipe_for_child(stage: &'static str, child_reads: bool) -> Result<(FHandle, FHandle)> {
    let (read_end, write_end) = create_pipe_handles(stage)?;
    if child_reads {
        clear_parent_inherit(&write_end, stage)?;
        Ok((read_end, write_end))
    } else {
        clear_parent_inherit(&read_end, stage)?;
        Ok((write_end, read_end))
    }
}

fn create_pipe_handles(stage: &'static str) -> Result<(FHandle, FHandle)> {
    let sa = inheritable_security_attributes();
    let (mut read_end, mut write_end) = (HANDLE::default(), HANDLE::default());
    // SAFETY: HANDLE buffers are initialized and `sa` describes inheritable pipe handles.
    unsafe { CreatePipe(&mut read_end, &mut write_end, Some(&sa), 0) }
        .map_err(|_| launch_stdio_error(pipe_create_stage(stage), "pipe"))?;
    Ok((
        handles::from_win32(read_end)?,
        handles::from_win32(write_end)?,
    ))
}

fn clear_parent_inherit(handle: &FHandle, stage: &'static str) -> Result<()> {
    // SAFETY: Handle is valid; clearing inheritance on the parent side is intentional.
    unsafe { SetHandleInformation(handle.as_win32(), HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }
        .map_err(|_| launch_stdio_error(pipe_inherit_stage(stage), "pipe"))
}

fn attach_child_stdio(
    info: &mut STARTUPINFOEXW,
    inherit_list: &mut InheritList,
    stdin: FHandle,
    stdout: FHandle,
    stderr: FHandle,
) {
    let raw_in = stdin.as_win32();
    let raw_out = stdout.as_win32();
    let raw_err = stderr.as_win32();
    inherit_list.push(stdin);
    inherit_list.push(stdout);
    inherit_list.push(stderr);
    info.StartupInfo.hStdInput = raw_in;
    info.StartupInfo.hStdOutput = raw_out;
    info.StartupInfo.hStdError = raw_err;
    info.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;
}

fn inheritable_security_attributes() -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: TRUE,
    }
}

fn launch_stdio_error(stage: &'static str, hint: &'static str) -> AcError {
    AcError::LaunchFailed {
        stage,
        hint,
        source: Box::new(std::io::Error::last_os_error()),
    }
}

fn pipe_create_stage(stage: &'static str) -> &'static str {
    match stage {
        "stdin" => "CreatePipe(stdin)",
        "stdout" => "CreatePipe(stdout)",
        "stderr" => "CreatePipe(stderr)",
        _ => "CreatePipe(stdio)",
    }
}

fn pipe_inherit_stage(stage: &'static str) -> &'static str {
    match stage {
        "stdin" => "SetHandleInformation(stdin parent)",
        "stdout" => "SetHandleInformation(stdout parent)",
        "stderr" => "SetHandleInformation(stderr parent)",
        _ => "SetHandleInformation(stdio parent)",
    }
}
