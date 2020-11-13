use origen::app;
use std::fs;

pub fn run(targets: Option<Vec<&str>>, mode: &Option<&str>) {
    let dot_origen_dir = app().unwrap().root.join(".origen");
    if !dot_origen_dir.exists() {
        let _ = fs::create_dir(&dot_origen_dir);
    }
    let history_file = dot_origen_dir.join("console_history");
    if !history_file.exists() {
        let _ = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&history_file);
    }

    super::launch("interactive", targets, mode, None, None, None, None);
}
