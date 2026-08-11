use crate::{AcError, Result};
use std::fs::File;
use std::io::Read;
use std::thread::{self, JoinHandle};

/// Concurrent readers for a launched process's stdout and stderr pipes.
#[derive(Debug)]
pub struct OutputCapture {
    stdout: Option<JoinHandle<std::io::Result<Vec<u8>>>>,
    stderr: Option<JoinHandle<std::io::Result<Vec<u8>>>>,
}

/// Bytes captured from a launched process without serial pipe-drain deadlocks.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CapturedOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl OutputCapture {
    pub(super) fn start(stdout: Option<File>, stderr: Option<File>) -> Self {
        Self {
            stdout: stdout.map(spawn_reader),
            stderr: stderr.map(spawn_reader),
        }
    }

    /// Join both readers after the child exits or is terminated.
    pub fn finish(self) -> Result<CapturedOutput> {
        Ok(CapturedOutput {
            stdout: join_reader(self.stdout, "stdout")?,
            stderr: join_reader(self.stderr, "stderr")?,
        })
    }
}

fn spawn_reader(mut pipe: File) -> JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn join_reader(
    reader: Option<JoinHandle<std::io::Result<Vec<u8>>>>,
    stream: &'static str,
) -> Result<Vec<u8>> {
    let Some(reader) = reader else {
        return Ok(Vec::new());
    };
    reader
        .join()
        .map_err(|_| AcError::Win32(format!("{stream} capture thread panicked")))?
        .map_err(|error| AcError::LaunchFailed {
            stage: "capture",
            hint: stream,
            source: Box::new(error),
        })
}
