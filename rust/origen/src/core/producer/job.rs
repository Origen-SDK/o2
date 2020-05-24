use std::path::{Path, PathBuf};

/// A job represents the execution of an Origen application source file.
/// For example, if the user runs `origen g <pat1> <pat2>` then two jobs will be created,
/// one for each pattern source file.
pub struct Job {
    pub command: String,
    pub results: Option<String>,
    pub id: usize,
    /// A stack of source files executed by the job. The first one is typically the file
    /// supplied by the user, then if that file imports other files, e.g. a sub-flow, then
    /// that will be added to this stack and then popped off when completed.
    pub files: Vec<PathBuf>,
}

impl Job {
    pub fn source_file(&self) -> Option<&Path> {
        if self.files.is_empty() {
            None
        } else {
            Some(&self.files[0])
        }
    }

    pub fn command(&self) -> String {
        let mut cmd = "origen ".to_string();
        cmd += &self.command;
        if let Some(p) = self.files.first() {
            if let Some(f) = p.to_str() {
                cmd += " ";
                cmd += f;
            }
        }
        cmd
    }
}
