use crate::python::PYTHON_CONFIG;
use origen::core::os;

pub fn run() {
    let _ = os::cmd(&PYTHON_CONFIG.poetry_command)
        .arg("update")
        .status();

    // Don't think we need to do anything here, if something goes wrong Poetry will give a better
    // error message than we could
}
