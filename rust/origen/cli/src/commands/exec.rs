use crate::python::{add_origen_env, PYTHON_CONFIG};
use std::process::Command;

pub fn run(cmd_name: &str, args: Vec<&str>) {
    let mut cmd = Command::new(&PYTHON_CONFIG.poetry_command);
    cmd.arg("run");
    cmd.arg(cmd_name);
    cmd.args(&args);

    add_origen_env(&mut cmd);

    let res = cmd
        .status()
        .expect("Something went wrong executing the command");

    if res.success() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
