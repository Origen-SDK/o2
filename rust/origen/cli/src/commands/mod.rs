pub mod fmt;
pub mod interactive;
pub mod mode;
pub mod proj;
pub mod setup;
pub mod target;
pub mod update;

use crate::python;
use indexmap::map::IndexMap;
use origen::{clean_mode, LOGGER};

/// Launch the given command in Python
pub fn launch(
    command: &str,
    targets: Option<Vec<&str>>,
    mode: &Option<&str>,
    files: Option<Vec<&str>>,
    cmd_args: Option<IndexMap<&str, String>>,
) {
    let mut cmd = format!(
        "from origen.boot import __origen__; __origen__('{}'",
        command
    );

    if let Some(t) = targets {
        // added r prefix to the string to force python to interpret as a string literal
        let _t: Vec<String> = t.iter().map(|__t| format!("r'{}'", __t)).collect();
        cmd += &format!(", targets=[{}]", &_t.join(",")).to_string();
    }

    if mode.is_some() {
        let c = clean_mode(mode.unwrap());
        cmd += &format!(", mode='{}'", c).to_string();
    }

    if files.is_some() {
        // added r prefix to the string to force python to interpret as a string literal
        let f: Vec<String> = files.unwrap().iter().map(|f| format!("r'{}'", f)).collect();
        cmd += &format!(", files=[{}]", f.join(",")).to_string();
    }

    if let Some(args) = cmd_args {
        cmd += ", args={";
        cmd += &args.iter().map(|(arg, val)| format!("'{}': {}", arg, val)).collect::<Vec<String>>().join(",");
        cmd += "}";
    }

    cmd += &format!(", verbosity={}", LOGGER.verbosity());

    cmd += ");";

    python::run(&cmd);
}
