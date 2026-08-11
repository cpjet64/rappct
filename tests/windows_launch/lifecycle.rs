use super::cmd_exe;
use rappct::{
    AppContainerProfile, JobLimits, KnownCapability, LaunchOptions, SecurityCapabilities,
    SecurityCapabilitiesBuilder, StdioConfig, launch_in_container, launch_in_container_with_io,
};
use std::time::Duration;

struct TestProfile(AppContainerProfile);

impl TestProfile {
    fn create() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .subsec_millis();
        let name = format!("rappct.test.life.{}.{nonce}", std::process::id());
        Self(AppContainerProfile::ensure(&name, &name, Some("rappct test")).unwrap())
    }

    fn capabilities(&self) -> SecurityCapabilities {
        SecurityCapabilitiesBuilder::new(&self.0.sid)
            .with_known(&[KnownCapability::InternetClient])
            .build()
            .expect("build capabilities")
    }
}

impl Drop for TestProfile {
    fn drop(&mut self) {
        let _ = self.0.clone().delete();
    }
}

fn bounded_command(command: &str) -> LaunchOptions {
    LaunchOptions {
        exe: cmd_exe(),
        cmdline: Some(format!(" /D /C {command}")),
        stdio: StdioConfig::Null,
        ..Default::default()
    }
}

fn pid_only_launch_rejects_pipe_mode_before_spawn(profile: &TestProfile) {
    let caps = profile.capabilities();
    let mut opts = bounded_command("exit 0");
    opts.stdio = StdioConfig::Pipe;
    let error = launch_in_container(&caps, &opts).expect_err("pipe mode must use I/O API");
    assert!(matches!(
        error,
        rappct::AcError::LaunchFailed {
            stage: "launch_options",
            ..
        }
    ));
}

fn wait_timeout_preserves_termination_control(profile: &TestProfile) {
    let caps = profile.capabilities();
    let mut opts = bounded_command("exit 0");
    opts.suspended = true;
    let mut child = launch_in_container(&caps, &opts).expect("launch bounded child");
    child
        .wait(Some(Duration::from_millis(20)))
        .expect_err("child should still be running");
    let exit = child.terminate(41).expect("terminate after timeout");
    assert_eq!(exit, 41);
}

fn suspended_launch_resumes_and_exits(profile: &TestProfile) {
    let caps = profile.capabilities();
    let mut opts = bounded_command("exit 23");
    opts.suspended = true;
    let mut child = launch_in_container(&caps, &opts).expect("launch suspended child");
    child
        .wait(Some(Duration::from_millis(20)))
        .expect_err("suspended child should not exit");
    child.resume().expect("resume primary thread");
    assert_eq!(child.wait(Some(Duration::from_secs(5))).unwrap(), 23);
    assert!(child.resume().is_err(), "second resume must fail");
}

fn dual_stream_capture_drains_without_deadlock(profile: &TestProfile) {
    let caps = profile.capabilities();
    let mut opts = bounded_command(
        "(for /L %i in (1,1,8000) do @echo OUT-%i) & \
         (for /L %i in (1,1,8000) do @echo ERR-%i 1>&2)",
    );
    opts.stdio = StdioConfig::Pipe;
    let mut child = launch_in_container_with_io(&caps, &opts).expect("launch capture child");
    let capture = child.capture_output();
    assert_eq!(child.wait(Some(Duration::from_secs(10))).unwrap(), 0);
    let output = capture.finish().expect("join capture readers");
    assert!(output.stdout.len() > 64 * 1024);
    assert!(output.stderr.len() > 64 * 1024);
    assert!(output.stdout.ends_with(b"OUT-8000\r\n"));
    assert!(output.stderr.ends_with(b"ERR-8000 \r\n") || output.stderr.ends_with(b"ERR-8000\r\n"));
}

fn pid_only_owner_retains_kill_on_close_job(profile: &TestProfile) {
    let caps = profile.capabilities();
    let mut opts = bounded_command("choice /D Y /T 20 >NUL");
    opts.join_job = Some(JobLimits {
        kill_on_job_close: true,
        ..Default::default()
    });
    let child = launch_in_container(&caps, &opts).expect("launch job-owned child");
    assert!(child.pid > 0);
    drop(child);
}

#[test]
fn launch_lifecycle_contracts() {
    let profile = TestProfile::create();
    pid_only_launch_rejects_pipe_mode_before_spawn(&profile);
    wait_timeout_preserves_termination_control(&profile);
    suspended_launch_resumes_and_exits(&profile);
    dual_stream_capture_drains_without_deadlock(&profile);
    pid_only_owner_retains_kill_on_close_job(&profile);
}
