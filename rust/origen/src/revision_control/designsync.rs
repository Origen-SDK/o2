use super::{Credentials, RevisionControlAPI, Status};
use crate::utility::command_helpers::log_stdout_and_stderr;
use crate::utility::file_utils::with_dir;
use crate::{Error, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use chrono::offset::Utc;

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

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<bool> {
        self.pop(force, path, version)?;
        Ok(false)
    }

    fn revert(&self, path: Option<&Path>) -> Result<()> {
        // It seems like there is no simple way to checkout the current view without
        // potentially pulling in additional updates from the server.
        // This works by generating a throwaway tag to record the current versions of all
        // files, then forcing a re-pop of that tag.
        let timestamp = format!("ts{}", Utc::now().timestamp_millis());
        self._tag(&timestamp, false, None, true)?;
        self.pop(true, None, &timestamp)?;
        Ok(())
    }

    fn status(&self, _path: Option<&Path>) -> Result<Status> {
        with_dir(&self.local, || {
            let mut status = Status::default();
            let mut process = Command::new("dssc")
                .args(&["compare", "-rec", "-path", "-report", "silent", "."])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            // We need to parse the following data from the output, ignore everything else
            //
            // Unmanaged                        First only         some_new.file
            // 1.7 (Reference)        1.7       Different states   some_deleted.file
            // 1.32 (Locally Modified) 1.32     Different states   some_modified.file

            lazy_static! {
                static ref NEW_FILE_REGEX: Regex =
                    Regex::new(r"\s*Unmanaged\s+First only\s+(\S+)").unwrap();
                static ref DELETED_FILE_REGEX: Regex = Regex::new(
                    r"\s*\d+\.*\d*\s*\(Reference\)\s+\d+\.*\d*\s*Different \w+\s*(\S+)"
                )
                .unwrap();
                static ref MODIFIED_FILE_REGEX: Regex = Regex::new(
                    r"\s*\d+\.*\d*\s*\(Locally Modified\)\s+\d+\.*\d*\s*Different \w+\s*(\S+)"
                )
                .unwrap();
            }

            log_stdout_and_stderr(
                &mut process,
                Some(&mut |line: &str| {
                    log_debug!("{}", line);
                    if let Some(captures) = NEW_FILE_REGEX.captures(&line) {
                        status.added.push(self.local.join(&captures[1]));
                    } else if let Some(captures) = DELETED_FILE_REGEX.captures(&line) {
                        status.removed.push(self.local.join(&captures[1]));
                    } else if let Some(captures) = MODIFIED_FILE_REGEX.captures(&line) {
                        status.changed.push(self.local.join(&captures[1]));
                    }
                }),
                None,
            );

            if process.wait()?.success() {
                Ok(status)
            } else {
                error!(
                    "Something went wrong reporting the dssc status for '{}', see log for details",
                    self.local.display()
                )
            }
        })
    }

    fn tag(&self, tagname: &str, force: bool, _message: Option<&str>) -> Result<()> {
        self._tag(tagname, force, _message, true)
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

    fn _tag(&self, tagname: &str, force: bool, _message: Option<&str>, modified: bool) -> Result<()> {
        with_dir(&self.local, || {
            let replace = match force {
                false => "",
                true => "-replace",
            };
            let modified = match modified {
                false => "",
                true => "-modified",
            };
            let mut process = Command::new("dssc")
                .args(&["tag", tagname, replace, modified, "-rec", "."])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let mut err_msg: Option<String> = None;

            lazy_static! {
                static ref TAG_EXISTS_REGEX: Regex =
                    Regex::new(r"This tag is already in use").unwrap();
            }

            log_stdout_and_stderr(&mut process, 
                Some(&mut |line: &str| {
                    log_debug!("{}", line);
                    if TAG_EXISTS_REGEX.is_match(&line) {
                        if err_msg.is_none() {
                            err_msg = Some("tag already exists".to_string());
                        }
                    }
                }),
                None
            );

            let p = process.wait()?;

            if let Some(msg) = err_msg {
                error!("{}", msg)
            } else if p.success() {
                Ok(())
            } else {
                error!(
                    "Something went wrong tagging '{}', see log for details",
                    self.local.display()
                )
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
