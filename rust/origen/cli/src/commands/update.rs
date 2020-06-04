use crate::python::PYTHON_CONFIG;
use std::process::Command;

pub fn run() {
    let _ = Command::new(&PYTHON_CONFIG.poetry_command)
        .arg("update")
        .status();

    // Don't think we need to do anything here, if something goes wrong Poetry will give a better
    // error message than we could
}
