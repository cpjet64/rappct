#[cfg(windows)]
#[path = "support/windows_test_utils.rs"]
mod windows_test_utils;

#[cfg(windows)]
fn cmd_exe() -> std::path::PathBuf {
    std::path::PathBuf::from("C:/Windows/System32/cmd.exe")
}

#[cfg(windows)]
#[path = "windows_launch/basic.rs"]
mod basic;
#[cfg(all(windows, feature = "introspection"))]
#[path = "windows_launch/diagnostics.rs"]
mod diagnostics;
#[cfg(windows)]
#[path = "windows_launch/job.rs"]
mod job;
#[cfg(windows)]
#[path = "windows_launch/lifecycle.rs"]
mod lifecycle;
#[cfg(windows)]
#[path = "windows_launch/stdio.rs"]
mod stdio;
#[cfg(windows)]
#[path = "windows_launch/token.rs"]
mod token;
