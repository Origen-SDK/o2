pub mod environment;
pub mod interactive;
pub mod setup;
pub mod target;
pub mod version;

use crate::python;

/// Launch the given command in Python
pub fn launch(command: &str) {
    let cmd = format!("
from origen.boot import __origen__;

__origen__('{}');

    ", command);

    python::run(&cmd);
}
