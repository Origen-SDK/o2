// Responsible for managing Python execution

use crate::built_info;
use origen::Result;
use semver::Version;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

const PYTHONS: &[&str] = &[
    "python",
    "python3",
    "python3.10",
    "python3.9",
    "python3.8",
    "python3.7",
    "python3.6",
];
pub const MIN_PYTHON_VERSION: &str = "3.6.0";

lazy_static! {
    pub static ref PYTHON_CONFIG: Config = Config::default();
}

pub struct Config {
    pub available: bool,
    pub command: String,
    pub version: Version,
    pub error: String,
}

impl Config {
    pub fn poetry_command(&self) -> Command {
        let mut c = Command::new(&self.command);
        c.arg("-m");
        c.arg("poetry");
        c
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut available = false;
        for cmd in PYTHONS.iter() {
            match get_version(cmd) {
                Some(version) => {
                    available = true;
                    if version >= Version::parse(MIN_PYTHON_VERSION).unwrap() {
                        return Config {
                            available: true,
                            command: cmd.to_string(),
                            version: version,
                            error: "".to_string(),
                        };
                    }
                }
                None => {}
            }
        }
        let mut msg = format!("Your environment does not have Python installed/available");
        if available {
            msg = format!("Your environment has Python available but it is too old, Origen needs a minimum of Python {}", MIN_PYTHON_VERSION);
        }
        Config {
            available: false,
            command: String::new(),
            version: Version::parse("0.0.0").unwrap(),
            error: msg,
        }
    }
}

/// Returns a path to the virtual env for the current (application) directory.
/// The caller is responsible for setting the current directory before calling this.
pub fn virtual_env() -> Result<PathBuf> {
    let (_code, stdout, stderr) =
        origen::utility::command_helpers::exec_and_capture("poetry", Some(vec!["env", "info"]))?;
    let r = regex::Regex::new(r"^Path:\s*(.*)").unwrap();
    for line in &stdout {
        log_trace!("{}", line);
        if let Some(captures) = r.captures(line) {
            return Ok(PathBuf::from(captures.get(1).unwrap().as_str()));
        }
    }
    for line in stdout {
        log_debug!("{}", line);
    }
    for line in stderr {
        log_debug!("[STDERR] {}", line);
    }
    error!("Could not read the path info from Poetry's output, run with full verbosity to see what happened")
}

/// Get the Python version from the given command
fn get_version(command: &str) -> Option<Version> {
    match Command::new(command).arg("--version").output() {
        Ok(output) => return extract_version(std::str::from_utf8(&output.stdout).unwrap()),
        Err(_e) => return None,
    }
}

/// Returns the version of poetry (obtained from running "poetry --version")
pub fn poetry_version() -> Option<Version> {
    //log_trace!("Executing command: {} --version", &PYTHON_CONFIG.poetry_command);
    //match Command::new(&PYTHON_CONFIG.poetry_command)
    match &PYTHON_CONFIG.poetry_command().arg("--version").output() {
        Ok(output) => {
            let text = std::str::from_utf8(&output.stdout).unwrap();
            log_trace!("{}", text);
            extract_version(text)
        }
        Err(e) => {
            log_debug!("{}", e);
            None
        }
    }
}

fn extract_version(text: &str) -> Option<Version> {
    let re = regex::Regex::new(r".*(\d+\.\d+\.\d+)([^\s\)]+)?").unwrap();

    match re.captures(text) {
        Some(x) => {
            let c = {
                let v = x.get(1).unwrap().as_str();
                if let Some(p) = x.get(2) {
                    let mut p = p.as_str();
                    if p.starts_with("-") {
                        format!("{}{}", v, p)
                    } else {
                        if p.starts_with(".") || p.starts_with("+") {
                            p = &p[1..];
                        }
                        format!("{}-{}", v, p)
                    }
                } else {
                    v.to_string()
                }
            };
            match Version::parse(&c) {
                Ok(v) => {
                    return Some(v);
                }
                Err(e) => {
                    panic!("Unable to parse version {}. Received Error:\n {}", c, e);
                }
            }
        }
        None => {
            return None;
        }
    };
}

/// Execute the given Python code
pub fn run(code: &str) -> Result<ExitStatus> {
    let mut cmd = PYTHON_CONFIG.poetry_command();
    cmd.arg("run");
    cmd.arg(&PYTHON_CONFIG.command);
    cmd.arg("-c");
    cmd.arg(&code);
    cmd.arg("-");
    cmd.arg(&format!("verbosity={}", origen::LOGGER.verbosity()));
    cmd.arg(&format!(
        "verbosity_keywords={}",
        origen::LOGGER.keywords_to_cmd()
    ));
    // current_exe returns the Python process once it gets underway, so pass in the CLI
    // location for Origen to use (used to resolve Origen config files)
    if let Ok(p) = std::env::current_exe() {
        cmd.arg(&format!("origen_cli={}", p.display()));
    };
    cmd.arg(&format!("origen_cli_version={}", built_info::PKG_VERSION));

    add_origen_env(&mut cmd);

    log_trace!("Running Python command: '{:?}'", cmd);

    Ok(cmd.status()?)
}

/// Run silently with all STDOUT and STDERR handled by the given callback functions
pub fn run_with_callbacks(
    code: &str,
    stdout_callback: Option<&mut dyn FnMut(&str)>,
    stderr_callback: Option<&mut dyn FnMut(&str)>,
) -> Result<()> {
    use origen::utility::command_helpers::log_stdout_and_stderr;

    let mut cmd = PYTHON_CONFIG.poetry_command();
    cmd.arg("run");
    cmd.arg(&PYTHON_CONFIG.command);
    cmd.arg("-c");
    cmd.arg(&code);
    cmd.arg("-");
    // Force logger to be silent, use case for this is parsing output data so keep
    // noise to a minimum
    cmd.arg("verbosity=0");
    // current_exe returns the Python process once it gets underway, so pass in the CLI
    // location for Origen to use (used to resolve Origen config files)
    if let Ok(p) = std::env::current_exe() {
        cmd.arg(&format!("origen_cli={}", p.display()));
    };
    cmd.arg(&format!("origen_cli_version={}", built_info::PKG_VERSION));
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    add_origen_env(&mut cmd);

    log_trace!("Running Python command: '{:?}'", cmd);

    let mut process = cmd.spawn()?;

    log_stdout_and_stderr(&mut process, stdout_callback, stderr_callback);

    if process.wait()?.success() {
        Ok(())
    } else {
        error!(
            "Something went wrong running the operation '{}', the log may have more details",
            code
        )
    }
}

/// Adds any Origen-related environment settings to a command
pub fn add_origen_env(cmd: &mut Command) {
    if origen::STATUS.is_origen_present || origen::STATUS.is_app_in_origen_dev_mode {
        cmd.env(
            "PYTHONPATH",
            format!(
                "{}",
                origen::STATUS
                    .origen_wksp_root
                    .join("rust")
                    .join("pyapi")
                    .join("target")
                    .display()
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_version_works() {
        assert_eq!(
            Version::parse("2.7.15-a").unwrap(),
            extract_version("Python 2.7.15+a\n").unwrap()
        );
        assert_eq!(
            Version::parse("3.6.8").unwrap(),
            extract_version("Python 3.6.8 \n").unwrap()
        );
        assert_eq!(
            Version::parse("1.1.0-rc1").unwrap(),
            extract_version("Poetry version 1.1.0rc1\n").unwrap()
        );
    }
}
