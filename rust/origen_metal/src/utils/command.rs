use crate::Result;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub fn exec_and_capture_cmd(mut command: Command) -> Result<(std::process::ExitStatus, Vec<String>, Vec<String>)> {
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
    exec_and_capture_cmd(command)
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

#[macro_export]
macro_rules! new_cmd {
    ($base_cmd:expr) => {{
        if cfg!(windows) {
            let mut c = std::process::Command::new("cmd");
            c.arg(r"/c").arg($base_cmd);
            c
        } else {
            std::process::Command::new($base_cmd)
        }
    }};
}

pub fn exec<S: Into<String> + Clone>(
    cmd: Vec<S>,
    capture: bool,
    timeout: Option<Duration>,
    cd: Option<PathBuf>,
    add_env: Option<HashMap<String, String>>,
    remove_env: Option<Vec<String>>,
    clear_env: bool,
) -> Result<ExecResult> {
    let mut command;
    if cfg!(windows) {
        command = Command::new("cmd");
        command.arg(r"/c");
        for c in cmd {
            command.arg(Into::<String>::into(c));
        }
    } else {
        command = Command::new(cmd[0].to_owned().into());
        for c in cmd[1..].iter() {
            command.arg(Into::<String>::into(c.to_owned()));
        }
    }
    if let Some(d) = cd {
        command.current_dir(d);
    }
    if clear_env {
        if remove_env.is_some() {
            bail!("Options 'clear_env' and 'remove_env' cannot be used simultaneously");
        } else {
            command.env_clear();
        }
    } else {
        if let Some(envs) = remove_env {
            for e in envs {
                command.env_remove(e);
            }
        }
    }
    if let Some(envs) = add_env {
        command.envs(envs);
    }

    log_debug!("Running cmd: {:?}", command);
    if capture {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    }

    let mut process = command.spawn()?;

    drop(command);
    let mut stdout_lines: Vec<String> = vec![];
    let mut stderr_lines: Vec<String> = vec![];
    if capture {
        log_stdout_and_stderr(
            &mut process,
            Some(&mut |line: &str| {
                log_info!("{}", line);
                stdout_lines.push(line.to_owned());
            }),
            Some(&mut |line: &str| {
                log_error!("{}", line);
                stderr_lines.push(line.to_owned());
            }),
        );
    }
    let exit_code;

    if let Some(t) = timeout {
        log_debug!("Timeout of {:?} set", t);
        exit_code = match process.wait_timeout(t)? {
            Some(ec) => ec,
            None => {
                log_error!("Timeout of {:?} reached. Killing command...", t);
                process.kill()?;
                process.wait()?
            }
        }
    } else {
        exit_code = process.wait()?;
    }
    Ok(ExecResult {
        exit_code: if let Some(code) = exit_code.code() {
            code
        } else {
            -1
        },
        stdout: if capture { Some(stdout_lines) } else { None },
        stderr: if capture { Some(stderr_lines) } else { None },
    })
}

pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: Option<Vec<String>>,
    pub stderr: Option<Vec<String>>,
}

impl ExecResult {
    pub fn succeeded(&self) -> bool {
        self.exit_code == 0
    }

    pub fn failed(&self) -> bool {
        !self.succeeded()
    }
}
