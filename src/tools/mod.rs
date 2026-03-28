pub mod adb;
pub mod apktool;
pub mod merger;
pub mod signer;
pub mod zipalign;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub line: String,
    pub stream: OutputStream,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub exit_code: i32,
    pub duration: Duration,
    pub output: Vec<ToolOutput>,
}

pub fn run_command_collect(mut command: Command) -> Result<ToolResult, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to spawn process: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| String::from("failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| String::from("failed to capture stderr"))?;

    let (tx, rx) = mpsc::channel::<ToolOutput>();

    let stdout_reader = spawn_reader_thread(stdout, OutputStream::Stdout, tx.clone());
    let stderr_reader = spawn_reader_thread(stderr, OutputStream::Stderr, tx);

    let status = child
        .wait()
        .map_err(|e| format!("failed to wait for process: {e}"))?;

    stdout_reader
        .join()
        .map_err(|_| String::from("stdout reader thread panicked"))?;
    stderr_reader
        .join()
        .map_err(|_| String::from("stderr reader thread panicked"))?;

    let mut output = Vec::new();
    while let Ok(item) = rx.recv() {
        output.push(item);
    }

    Ok(ToolResult {
        exit_code: status.code().unwrap_or(-1),
        duration: start.elapsed(),
        output,
    })
}

fn spawn_reader_thread<R: std::io::Read + Send + 'static>(
    stream: R,
    output_stream: OutputStream,
    sender: Sender<ToolOutput>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            let _ = sender.send(ToolOutput {
                line,
                stream: output_stream,
            });
        }
    })
}
