use super::LaunchOptions;
use crate::{AcError, Result};
use windows::Win32::Foundation::{HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, WaitForInputIdle, WaitForSingleObject,
};

pub(super) fn effective_startup_timeout(opts: &LaunchOptions) -> Option<std::time::Duration> {
    if opts.suspended {
        None
    } else {
        opts.startup_timeout
    }
}

pub(super) fn wait_for_startup(opts: &LaunchOptions, process: HANDLE) -> Result<()> {
    if let Some(timeout) = effective_startup_timeout(opts) {
        wait_for_input_idle_or_probe_process(process, timeout)
    } else {
        trace_suspended_timeout_skip(opts);
        Ok(())
    }
}

fn wait_for_input_idle_or_probe_process(
    process: HANDLE,
    timeout: std::time::Duration,
) -> Result<()> {
    let ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
    // SAFETY: Process handle is live and timeout conversion is bounded to u32.
    let idle_wait = unsafe { WaitForInputIdle(process, ms) };

    if idle_wait == WAIT_TIMEOUT.0 {
        return Err(startup_timeout("child did not become input-idle in time"));
    }
    if idle_wait == WAIT_FAILED.0 {
        return probe_process_after_input_idle_failure(process);
    }
    Ok(())
}

fn probe_process_after_input_idle_failure(process: HANDLE) -> Result<()> {
    // WAIT_FAILED is expected for non-GUI processes; probe process state to detect true failure.
    // SAFETY: Zero-timeout probe on a live process handle.
    let probe = unsafe { WaitForSingleObject(process, 0) };
    if probe == WAIT_FAILED {
        return Err(AcError::Win32(
            "WaitForSingleObject(startup probe) failed".into(),
        ));
    }
    if probe == WAIT_OBJECT_0 {
        return Err(startup_exited(process)?);
    }

    #[cfg(feature = "tracing")]
    tracing::trace!("WaitForInputIdle returned WAIT_FAILED for non-GUI process; continuing");
    Ok(())
}

fn startup_exited(process: HANDLE) -> Result<AcError> {
    let mut code = 0u32;
    // SAFETY: Process handle is valid and out-parameter is initialized.
    unsafe { GetExitCodeProcess(process, &mut code) }
        .map_err(|_| AcError::Win32("GetExitCodeProcess failed".into()))?;
    Ok(AcError::LaunchFailed {
        stage: "startup_timeout",
        hint: "child exited before becoming input-idle",
        source: Box::new(std::io::Error::other(format!("exit code: {code}"))),
    })
}

fn startup_timeout(hint: &'static str) -> AcError {
    AcError::LaunchFailed {
        stage: "startup_timeout",
        hint,
        source: Box::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "WaitForInputIdle timeout",
        )),
    }
}

fn trace_suspended_timeout_skip(opts: &LaunchOptions) {
    if opts.suspended && opts.startup_timeout.is_some() {
        #[cfg(feature = "tracing")]
        tracing::trace!("startup_timeout ignored because launch is suspended");
    }
}
