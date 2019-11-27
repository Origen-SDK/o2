use crate::python;
use origen::STATUS;
use std::fs;

pub fn main() {
    let dot_origen_dir = STATUS.root.join(".origen");
    if !dot_origen_dir.exists() {
        let _ = fs::create_dir(&dot_origen_dir);
    }
    let history_file = dot_origen_dir.join("console_history");
    if !history_file.exists() {
        let _ = fs::OpenOptions::new().create(true).write(true).open(&history_file);
    }

    let cmd = format!("historyPath = \"{}\"\n{}", history_file.display(), include_str!("interactive.py"));

    python::run(&cmd);
}
