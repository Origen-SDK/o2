use std::io::{BufRead, BufReader};

/// Log both stdout and stderr to the debug and error logs respectively, optionally
/// calling a callback function for each line captured.
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

/// Log stdout to the debug log, optionally calling a callback function for every line captured
pub fn log_stdout(process: &mut std::process::Child, mut callback: Option<&mut dyn FnMut(&str)>) {
    let stdout = process.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            log_debug!("{}", line);
            if let Some(f) = &mut callback {
                f(&line);
            }
        });
}

/// Log stderr to the error log, optionally calling a callback function for every line captured
pub fn log_stderr(process: &mut std::process::Child, mut callback: Option<&mut dyn FnMut(&str)>) {
    let stdout = process.stderr.take().unwrap();
    let reader = BufReader::new(stdout);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            log_error!("{}", line);
            if let Some(f) = &mut callback {
                f(&line);
            }
        });
}
