use std::process::Command;

pub fn main() {
    let mut python = Command::new("python");
    python.arg("--version");

    python.status().expect("Python missing!");
}
