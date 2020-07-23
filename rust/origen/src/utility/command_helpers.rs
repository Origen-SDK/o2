use crate::Result;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Executes the given command/args, returning all captured stdout and stderr lines and
/// the exit code of the process.
pub fn exec_and_capture(
    cmd: &str,
    args: Option<Vec<&str>>,
) -> Result<(std::process::ExitStatus, Vec<String>, Vec<String>)> {
    let mut command = Command::new(cmd);
    if let Some(args) = args {
        for arg in args {
            command.arg(arg);
        }
    }
    let mut process = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdout_lines: Vec<String> = vec![];
    let mut stderr_lines: Vec<String> = vec![];

    log_stdout_and_stderr(
        &mut process,
        Some(&mut |line: &str| {
            stdout_lines.push(line.to_owned());
        }),
        Some(&mut |line: &str| {
            stderr_lines.push(line.to_owned());
        }),
    );
    let exit_code = process.wait()?;
    Ok((exit_code, stdout_lines, stderr_lines))
}

/// Log both stdout and stderr to the debug and error logs respectively, optionally
/// calling a callback function for each line captured.
/// If no callbacks are given then any captures lines will be sent to the debug and
/// error logs respectively.
/// This currently logs stderr after stdout, in future this will hopefully support concurrent
/// logging of both streams.
pub fn log_stdout_and_stderr(
    process: &mut std::process::Child,
    stdout_callback: Option<&mut dyn FnMut(&str)>,
    stderr_callback: Option<&mut dyn FnMut(&str)>,
) {
    log_stdout(process, stdout_callback);
    log_stderr(process, stderr_callback);
}

/// Log stdout to the debug log, optionally calling a callback function for every line captured.
/// If no callback is given then the default behavior is to output any STDOUT lines to the
/// debug log.
pub fn log_stdout(process: &mut std::process::Child, mut callback: Option<&mut dyn FnMut(&str)>) {
    let stdout = process.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            if let Some(f) = &mut callback {
                f(&line);
            } else {
                log_debug!("{}", line);
            }
        });
}

/// Log stderr to the error log, optionally calling a callback function for every line captured.
/// If no callback is given then the default behavior is to output any STDERR lines to the
/// error log.
pub fn log_stderr(process: &mut std::process::Child, mut callback: Option<&mut dyn FnMut(&str)>) {
    let stdout = process.stderr.take().unwrap();
    let reader = BufReader::new(stdout);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            if let Some(f) = &mut callback {
                f(&line);
            } else {
                log_error!("{}", line);
            }
        });
}
