use super::cmd_exe;
use rappct::*;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::STILL_ACTIVE;
use windows::Win32::System::JobObjects::{
    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE, JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    JOBOBJECT_CPU_RATE_CONTROL_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectCpuRateControlInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject,
};

fn launch_profile(scope: &str) -> AppContainerProfile {
    let name = format!("rappct.test.launch.{scope}.{}", std::process::id());
    AppContainerProfile::ensure(&name, &name, Some("rappct test")).expect("ensure")
}

fn internet_caps(profile: &AppContainerProfile) -> SecurityCapabilities {
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .with_known(&[KnownCapability::InternetClient])
        .build()
        .expect("build caps")
}

#[test]
fn launch_ac_with_job_limits() {
    let prof = launch_profile("job");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C exit 0".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: Some(32 * 1024 * 1024),
            cpu_rate_percent: Some(50),
            kill_on_job_close: false,
        }),
        ..Default::default()
    };
    let child = launch_in_container(&caps, &opts).expect("launch with job limits");
    assert!(child.pid > 0);
    prof.delete().ok();
}

#[test]
fn launch_job_limits_reported_by_query() {
    let prof = launch_profile("jobinfo");
    let caps = internet_caps(&prof);
    let memory_limit = 8 * 1024 * 1024;
    let cpu_percent = 25;
    let child = launch_reportable_job(&caps, memory_limit, cpu_percent);
    let job_handle = child
        .job_guard
        .as_ref()
        .expect("job guard missing")
        .as_handle();

    assert_memory_limit(job_handle, memory_limit);
    assert_cpu_limit(job_handle, cpu_percent);
    drop_guard_and_wait(child);
    prof.delete().ok();
}

fn launch_reportable_job(
    caps: &SecurityCapabilities,
    memory_limit: usize,
    cpu_percent: u32,
) -> rappct::launch::LaunchedIo {
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C timeout /T 30 /NOBREAK > NUL".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: Some(memory_limit),
            cpu_rate_percent: Some(cpu_percent),
            kill_on_job_close: true,
        }),
        ..Default::default()
    };
    launch_in_container_with_io(caps, &opts).expect("launch with job limits")
}

fn assert_memory_limit(job_handle: windows::Win32::Foundation::HANDLE, memory_limit: usize) {
    unsafe {
        let mut ext: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        QueryInformationJobObject(
            Some(job_handle),
            JobObjectExtendedLimitInformation,
            &mut ext as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            None,
        )
        .expect("QueryInformationJobObject(ext) failed");
        assert_ne!(
            (ext.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_PROCESS_MEMORY).0,
            0,
            "process memory limit flag not set",
        );
        assert_ne!(
            (ext.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE).0,
            0,
            "kill-on-close flag not set",
        );
        assert_eq!(
            ext.ProcessMemoryLimit, memory_limit,
            "memory limit mismatch"
        );
    }
}

fn assert_cpu_limit(job_handle: windows::Win32::Foundation::HANDLE, cpu_percent: u32) {
    unsafe {
        let mut cpu: JOBOBJECT_CPU_RATE_CONTROL_INFORMATION = std::mem::zeroed();
        QueryInformationJobObject(
            Some(job_handle),
            JobObjectCpuRateControlInformation,
            &mut cpu as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
            None,
        )
        .expect("QueryInformationJobObject(cpu) failed");
        assert_ne!(
            (cpu.ControlFlags & JOB_OBJECT_CPU_RATE_CONTROL_ENABLE).0,
            0,
            "cpu control enable not set",
        );
        assert_ne!(
            (cpu.ControlFlags & JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP).0,
            0,
            "cpu hard cap not set",
        );
        assert_eq!(
            cpu.Anonymous.CpuRate,
            cpu_percent * 100,
            "cpu rate mismatch"
        );
    }
}

fn drop_guard_and_wait(mut child: rappct::launch::LaunchedIo) {
    let guard = child.job_guard.take().expect("job guard missing");
    drop(guard);
    let exit = child
        .wait(Some(Duration::from_secs(5)))
        .expect("wait after dropping guard");
    assert_ne!(
        exit, STILL_ACTIVE.0 as u32,
        "child still active after guard drop"
    );
}

#[test]
fn launch_job_guard_drop_terminates_process() {
    let prof = launch_profile("jobkill");
    let caps = internet_caps(&prof);
    let opts = LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(" /C timeout /T 30 /NOBREAK > NUL".to_string()),
        join_job: Some(JobLimits {
            memory_bytes: None,
            cpu_rate_percent: None,
            kill_on_job_close: true,
        }),
        ..Default::default()
    };
    let mut child = launch_in_container_with_io(&caps, &opts).expect("launch with kill-on-close");
    let guard = child.job_guard.take().expect("job guard missing");
    drop(guard);
    let start = Instant::now();
    let exit = child
        .wait(Some(Duration::from_secs(5)))
        .expect("wait after guard drop");
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "job guard drop did not terminate in time"
    );
    assert_ne!(exit, STILL_ACTIVE.0 as u32, "child remained active");
    prof.delete().ok();
}
