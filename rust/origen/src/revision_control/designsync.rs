use super::{Credentials, RevisionControlAPI};
use crate::utility::command_helpers::log_stdout_and_stderr;
use crate::utility::file_utils::with_dir;
use crate::{Error, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Designsync {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote vault
    pub remote: String,
}

impl Designsync {
    pub fn new(local: &Path, remote: &str, _credentials: Option<Credentials>) -> Designsync {
        Designsync {
            local: local.to_path_buf(),
            remote: remote.to_string(),
        }
    }
}

impl RevisionControlAPI for Designsync {
    fn populate(&self, version: &str) -> Result<()> {
        log_info!("Populating {}", &self.local.display());
        fs::create_dir_all(&self.local)?;
        self.set_vault()?;
        self.pop(true, Some(Path::new(".")), version)
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<()> {
        self.pop(force, path, version)
    }
}

impl Designsync {
    fn set_vault(&self) -> Result<()> {
        with_dir(&self.local, || {
            let mut process = Command::new("dssc")
                .args(&["setvault", &self.remote, "."])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            log_stdout_and_stderr(&mut process, None, None);

            if process.wait()?.success() {
                Ok(())
            } else {
                Err(Error::new(&format!(
                    "Something went wrong setting the vault in '{}', see log for details",
                    self.local.display()
                )))
            }
        })
    }

    fn pop(&self, force: bool, path: Option<&Path>, version: &str) -> Result<()> {
        let path = path.unwrap_or(Path::new("."));
        with_dir(&self.local, || {
            let mut args = vec!["pop", "-rec", "-uni", "-version"];
            args.push(version);
            if force {
                args.push("-force");
            }
            //    args.push("-merge");
            //}
            args.push(path.to_str().unwrap());
            log_debug!("Running DesignSync command: dssc {}", args.join(" "));
            let mut process = Command::new("dssc")
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            lazy_static! {
                static ref POP_REGEX: Regex =
                    Regex::new(r": Success - (Fetched|Created symbolic)").unwrap();
            }

            log_stdout_and_stderr(
                &mut process,
                None,
                // Example of a callback
                //Some(&mut |line: &str| {
                //    if POP_REGEX.is_match(&line) {
                //        progress.received_objects += 1;
                //        progress.completed_objects += 1;
                //    }
                //}),
                None,
            );

            if process.wait()?.success() {
                Ok(())
            } else {
                Err(Error::new(&format!(
                    "Something went wrong populating '{}', see log for details",
                    self.remote
                )))
            }
        })
    }
}
