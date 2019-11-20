use core::term::*;
use std::process::{Command, Stdio};

pub fn main() {
    print!("Is a suitable Python available? ... ");
    if core::python::CONFIG.available {
        greenln("YES");
    } else {
        redln("NO");
    }
    print!("Is Poetry installed?            ... ");
    let status = Command::new("poetry")
        .arg("--version")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status();

    if status.is_ok() && status.unwrap().success() {
        greenln("YES");
    } else {
        redln("NO");
    }
}
