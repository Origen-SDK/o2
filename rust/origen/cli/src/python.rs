// Responsible for managing Python execution
use crate::built_info;
use origen::{Result, STATUS};
use origen::core::status::DependencySrc;
use origen_metal::utils::file::search_backwards_for_first;
use origen_metal::new_cmd;
use semver::Version;
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use crate::_generated::python::PYTHONS;
pub use crate::_generated::python::MIN_PYTHON_VERSION;

#[macro_export]
macro_rules! strs_to_cli_arr {
    ($name:expr, $strs:expr) => {{
        format!(
            "{}=[{}]",
            $name,
            $strs.map(|t| format!("r'{}'", t)).collect::<Vec<String>>().join(", ")
        )
    }}
}

lazy_static! {
    pub static ref PYTHON_CONFIG: Config = Config::default();
}

macro_rules! pyproject_str {
    () => { "pyproject.toml" }
}
macro_rules! user_env_str {
    () => { "ORIGEN_PYPROJECT" }
}

const PYPROJECT: &'static str = pyproject_str!();
const USER_ENV: &'static str = user_env_str!();

lazy_static! {
    pub static ref NO_ORIGEN_BOOT_MODULE_ERROR: &'static str = "ModuleNotFoundError: No module named 'origen.boot'";
}

pub struct Config {
    pub available: bool,
    pub command: String,
    pub version: Version,
    pub error: String,
}

pub fn resolve_pyproject() -> Result<DependencySrc> {
    // TODO allow a .origen offset when looking for pyprojects?
    // let offset = ".origen";
    if let Some(app) = STATUS.app.as_ref() {
        let path = app.root.join(PYPROJECT);
        log_trace!("Found app pyproject: {}", path.display());
        return Ok(DependencySrc::App(path));
    }

    if let Some(p) = search_backwards_for_first(env::current_dir()?, |p| {
        let f = p.join(PYPROJECT);
        log_trace!("Searching for workspace project from {}", p.display());
        if f.exists() {
            log_trace!("Found workspace pyproject: {}", f.display());
            Ok(Some(f))
        } else {
            Ok(None)
        }
    })? {
        return Ok(DependencySrc::Workspace(p.to_path_buf()))
    }

    if let Some(p) = env::var_os(USER_ENV) {
        log_trace!("Attempting to find user-given pyproject: {}", p.to_string_lossy());
        let mut f = PathBuf::from(p);
        if f.exists() {
            f = f.join(PYPROJECT);
            if f.exists() {
                log_trace!("Found user-given pyproject: {}", f.display());
                return Ok(DependencySrc::UserGlobal(f));
            } else {
                bail!(concat!("Could not locate ", pyproject_str!(), " from ", user_env_str!(), " {}"), f.display());
            }
        } else {
            bail!(concat!(user_env_str!(), " '{}' does not exists!"), f.display());
        }
    }

    // Try the python package installation directory
    let path = std::env::current_exe()?;
    log_trace!("Searching CLI installation directories for pyproject: {}", path.display());
    if let Some(p) = search_backwards_for_first(path, |p| {
        let f = p.join(PYPROJECT);
        log_trace!("Searching for workspace project from {}", p.display());
        if f.exists() {
            log_trace!("Found workspace pyproject: {}", f.display());
            Ok(Some(f))
        } else {
            Ok(None)
        }
    })? {
        return Ok(DependencySrc::Global(p.to_path_buf()))
    }

    log_trace!("No pyproject found. Skipping Poetry invocations...");
    Ok(DependencySrc::NoneFound)
}

impl Config {
    pub fn base_cmd(&self) -> Command {
        let dep_src = STATUS.dependency_src();
        let mut c = new_cmd!(&self.command);

        if let Some(dep_src) = dep_src.as_ref() {
            match dep_src {
                DependencySrc::App(_path) | DependencySrc::Workspace(_path) => {
                    c.arg("-m");
                    c.arg("poetry");
                },
                DependencySrc::UserGlobal(path) | DependencySrc::Global(path) => {
                    c.arg("-m");
                    c.arg("poetry");
                    c.arg("-C");
                    c.arg(path);
                }
                DependencySrc::NoneFound => {}
            }
        } else {
            log_error!("Dependency source has not been set - defaulting to global Python installation");
        }
        c
    }

    pub fn run_cmd(&self, code: &str) -> Command {
        let mut c = self.base_cmd();
        if let Some(d) = STATUS.dependency_src().as_ref() {
            if d.src_available() {
                c.arg("run");
                c.arg(&self.command);
            }
        }
        c.arg("-c");
        c.arg(code);
        if let Some(d) = STATUS.dependency_src().as_ref() {
            c.arg(format!("invocation={}", d));
            if let Some(path) = d.src_file() {
                c.arg(format!("pyproject_src={}", path.display()));
            }
        }
        c
    }

    pub fn poetry_command(&self) -> Command {
        let mut c = Command::new(&self.command);
        c.arg("-m");
        c.arg("poetry");
        if let Some(d) = STATUS.dependency_src().as_ref() {
            if let Some(path) = d.src_file() {
                c.arg("-C");
                c.arg(path);
            }
        }
        c
    }

    // TODO Invocation see if these are needed or can be cleaned up
    // fn get_origen_pkg_path(&self) -> Result<PathBuf> {
    //     let mut c = Command::new("pip");
    //     c.arg("show");
    //     c.arg("origen");
    //     let output = exec_and_capture_cmd(c)?;
    //     if let Some(loc) = output.1.iter().find_map( |line| line.strip_prefix("Location: ")) {
    //         Ok(PathBuf::from(loc))
    //     } else {
    //         bail!(
    //             "Error locating origen package information from pip.\nReceived stdout:\n{}\nReceived stderr:\n{}",
    //             output.1.join("\n"),
    //             output.2.join("\n")
    //         );
    //     }
    // }

    // fn in_workspace(&self) -> Result<bool> {
    //     let mut c = Command::new(&self.command);
    //     c.arg("-m");
    //     c.arg("poetry");
    //     c.arg("env");
    //     c.arg("info");
    //     let output = exec_and_capture_cmd(c)?;
    //     if !output.0.success() {
    //         if let Some(l) = output.2.last() {
    //             if l.starts_with("Poetry could not find a pyproject.toml file in ") {
    //                 return Ok(false);
    //             }
    //         }
    //         bail!(
    //             "Unexpected response when querying poetry environment:\nReceived stdout:\n{}Received stderr:\n{}",
    //             output.1.join("\n"),
    //             output.2.join("\n")
    //         );
    //     }
    //     Ok(true)
    // }
}

impl Default for Config {
    fn default() -> Config {
        let mut available = false;
        match resolve_pyproject() {
            Ok(deps) => {
                STATUS.set_dependency_src(Some(deps))
            },
            Err(e) => log_error!("Errors encountered resolving pyproject: {}", e)
        }
        for cmd in PYTHONS.iter() {
            log_trace!("Searching for installed python at '{}'", cmd);
            match get_version(cmd) {
                Some(version) => {
                    available = true;
                    if version >= Version::parse(MIN_PYTHON_VERSION).unwrap() {
                        log_trace!("Found python version '{}'", cmd);
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
            command: String::from("python"),
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
    bail!("Could not read the path info from Poetry's output, run with full verbosity to see what happened")
}

/// Get the Python version from the given command
fn get_version(command: &str) -> Option<Version> {
    match new_cmd!(command).arg("--version").output() {
        Ok(output) => return extract_version(std::str::from_utf8(&output.stdout).unwrap()),
        Err(_e) => return None,
    }
}

/// Returns the version of poetry (obtained from running "poetry --version")
pub fn poetry_version() -> Option<Version> {
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
    let mut cmd = PYTHON_CONFIG.run_cmd(code);
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

/// Macro to create a Python command
#[macro_export]
macro_rules! python_cmd {
    ($code:expr) => {{
        let mut cmd = PYTHON_CONFIG.run_cmd($code);
        if let Ok(p) = std::env::current_exe() {
            cmd.arg(&format!("origen_cli={}", p.display()));
        };
        cmd.arg(&format!("origen_cli_version={}", built_info::PKG_VERSION));
        add_origen_env(&mut cmd);
        log_trace!("Running Python command: '{:?}'", cmd);
        cmd
    }};
}

/// Execute the given Python code and capture its stdout
pub fn run_and_capture_stdout(code: &str) -> Result<String> {
    let mut cmd = python_cmd!(code);
    cmd.stdout(Stdio::piped());

    let output = cmd.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        bail!(
            "Something went wrong running the operation '{}', the log may have more details",
            code
        )
    }
}

/// Run silently with all STDOUT and STDERR handled by the given callback functions
pub fn run_with_callbacks(
    code: &str,
    stdout_callback: Option<&mut dyn FnMut(&str)>,
    stderr_callback: Option<&mut dyn FnMut(&str)>,
) -> Result<()> {
    use origen::utility::command_helpers::log_stdout_and_stderr;

    let mut cmd = PYTHON_CONFIG.run_cmd(code);
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
        bail!(
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

pub fn is_backend_origen_mod_missing_err(err: &origen::Error) -> bool {
    err.to_string().contains("ModuleNotFoundError: No module named '_origen'")
}

/// Attempts to get the username and email, utilizing the full Origen environment (site config, etc.)
pub fn get_current_user_and_email() -> Result<(String, String, String)> {
    let out = run_and_capture_stdout("import origen; print('--user start--'); print(origen.current_user.first_name); print(origen.current_user.last_name); print(origen.current_user.email)")?;
    let split = out.split_once("--user start--");
    if let Some(s) = split {
        let mut info = s.1.trim().split("\n");
        let (fnm, lnm, email): (String, String, String);
        if let Some(i) = info.next() {
            fnm = i.trim().to_string();
        } else {
            bail!("Expected a first name when retrieving user info");
        }
        if let Some(i) = info.next() {
            lnm = i.trim().to_string();
        } else {
            bail!("Expected a last name when retrieving user info");
        }
        if let Some(i) = info.next() {
            email = i.trim().to_string();
        } else {
            bail!("Expected an email name when retrieving user info");
        }
        Ok((fnm, lnm, email))
    } else {
        bail!("Unable to find '--user start--' when retrieving user information");
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
