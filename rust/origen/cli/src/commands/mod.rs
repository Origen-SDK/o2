pub mod environment;
pub mod interactive;
pub mod mode;
pub mod setup;
pub mod target;
pub mod version;

use crate::python;
use origen::clean_mode;

/// Launch the given command in Python
pub fn launch(
    command: &str,
    target: &Option<&str>,
    environment: &Option<&str>,
    mode: &Option<&str>,
    files: Option<Vec<&str>>,
) {
    let mut cmd = format!(
        "from origen.boot import __origen__; __origen__('{}'",
        command
    );

    if target.is_some() {
        cmd += &format!(", target='{}'", target.unwrap()).to_string();
    }

    if environment.is_some() {
        cmd += &format!(", environment='{}'", environment.unwrap()).to_string();
    }

    if mode.is_some() {
        let c = clean_mode(mode.unwrap());
        cmd += &format!(", mode='{}'", c).to_string();
    }

    if files.is_some() {
        let f: Vec<String> = files.unwrap().iter().map(|f| format!("'{}'", f)).collect();
        cmd += &format!(", files=[{}]", f.join(",")).to_string();
    }

    cmd += ");";

    python::run(&cmd);
}
