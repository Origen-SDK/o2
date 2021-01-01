use crate::Result;
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
    /// Returns the file at the root of the job or None if the job has no file association.
    pub fn source_file(&self) -> Option<&PathBuf> {
        self.files.first()
    }

    /// Returns the current file being processed by the job or None if the job has no file association.
    /// This can be the same as source_file() or it can be different, for example if a flow job has
    /// included a sub-flow which is currently being processed.
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.files.last()
    }

    /// Rerturns the origen command that would be run to replicate the job, e.g. "origen g some_file.py"
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

    /// Tries to resolve a reference to a file by the following rules:
    ///   * Its an absolute file reference
    ///   * Its a relative reference from the job's latest file
    ///   * Its a relative reference from the job's original source file
    ///   * Its a relative reference from the app's app dir
    ///   * Its a relative reference from the app's root
    ///
    /// Optionally a list of possible extensions can be given and the above will be retried
    /// with the given extension(s) appended if required.
    pub fn resolve_file_reference(
        &self,
        file: &Path,
        extensions: Option<Vec<&str>>,
    ) -> Result<PathBuf> {
        let f = self._resolve_file_reference(file);
        if let Some(f) = f {
            return Ok(f);
        }
        if let Some(exts) = extensions {
            for ext in exts {
                let f = self._resolve_file_reference(&file.with_extension(ext));
                if let Some(f) = f {
                    return Ok(f);
                }
            }
        }
        dbg!(file);
        dbg!(&self.files);
        error!("Could not find '{}'", file.display())
    }

    fn _resolve_file_reference(&self, file: &Path) -> Option<PathBuf> {
        if file.is_absolute() {
            if file.exists() {
                return Some(file.to_path_buf());
            }
        } else {
            if let Some(root) = self.files.last() {
                dbg!(root.parent());
                if let Some(dir) = root.parent() {
                    let f = dir.join(file);
                    dbg!(&f);
                    if f.exists() {
                        dbg!("Does exist!");
                        return Some(f.to_path_buf());
                    }
                    match f.canonicalize() {
                        Ok(f) => {
                            dbg!(&f);
                            if f.exists() {
                                dbg!("Does exist!");
                                return Some(f.to_path_buf());
                            }
                        }
                        Err(e) => { dbg!(e); } 
                    }
                    dbg!("Does not exist!");
                }
            }
            if let Some(root) = self.files.first() {
                if let Some(dir) = root.parent() {
                    let f = dir.join(file);
                    if f.exists() {
                        return Some(f.to_path_buf());
                    }
                }
            }
            if let Some(app) = crate::app() {
                let dir = app.app_dir();
                let f = dir.join(file);
                if f.exists() {
                    return Some(f.to_path_buf());
                }
                let dir = app.root.clone();
                let f = dir.join(file);
                if f.exists() {
                    return Some(f.to_path_buf());
                }
            }
        }
        None
    }
}
