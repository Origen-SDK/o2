// Responsible for managing Python execution

use origen::STATUS;
use semver::Version;
use std::path::PathBuf;
use std::process::Command;

const PYTHONS: &[&str] = &[
    "python",
    "python3",
    "python3.8",
    "python3.7",
    "python3.6",
    "python3.5",
];
pub const MIN_PYTHON_VERSION: &str = "3.5.0";

lazy_static! {
    pub static ref PYTHON_CONFIG: Config = Config::default();
}

pub struct Config {
    pub available: bool,
    pub command: String,
    pub version: Version,
    pub error: String,
    pub poetry_command: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        let mut available = false;
        for cmd in PYTHONS.iter() {
            match get_version(cmd) {
                Some(version) => {
                    available = true;
                    let mut poetry_cmd = PathBuf::from(&STATUS.home);
                    for d in [".poetry", "bin", "poetry"].iter() {
                        poetry_cmd.push(d)
                    }
                    if cfg!(windows) {
                        poetry_cmd.set_extension("bat");
                    }
                    if version >= Version::parse(MIN_PYTHON_VERSION).unwrap() {
                        return Config {
                            available: true,
                            command: cmd.to_string(),
                            version: version,
                            error: "".to_string(),
                            poetry_command: poetry_cmd,
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
            poetry_command: PathBuf::new(),
        }
    }
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
    match Command::new(&PYTHON_CONFIG.poetry_command)
        .arg("--version")
        .output()
    {
        Ok(output) => return extract_version(std::str::from_utf8(&output.stdout).unwrap()),
        Err(_e) => return None,
    }
}

fn extract_version(text: &str) -> Option<Version> {
    let re = regex::Regex::new(r".*(\d+\.\d+\.\d+[\s]*)").unwrap();

    match re.captures(text) {
        Some(x) => {
            let c = x.get(1).unwrap().as_str();
            let v = Version::parse(c).unwrap();
            return Some(v);
        }
        None => {
            return None;
        }
    };
}

/// Execute the given Python code
pub fn run(code: &str) {
    let _status = Command::new(&PYTHON_CONFIG.poetry_command)
        .arg("run")
        .arg(&PYTHON_CONFIG.command)
        .arg("-c")
        .arg(&code)
        .arg("-")
        .arg(&format!("verbosity={}", origen::LOGGER.verbosity()))
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_version_works() {
        assert_eq!(
            Version::parse("2.7.15").unwrap(),
            extract_version("Python 2.7.15+a\n").unwrap()
        );
        assert_eq!(
            Version::parse("3.6.8").unwrap(),
            extract_version("Python 3.6.8 \n").unwrap()
        );
    }
}
