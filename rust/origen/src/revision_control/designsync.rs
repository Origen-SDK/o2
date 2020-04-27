use super::{Credentials, Progress, RevisionControlAPI};
use crate::utility::with_dir;
use crate::{Error, Result, USER};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use regex::Regex;

pub struct Designsync {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote vault
    pub remote: String,
    credentials: Option<Credentials>,
}

impl Designsync {
    pub fn new(local: &Path, remote: &str, credentials: Option<Credentials>) -> Designsync {
        Designsync {
            local: local.to_path_buf(),
            remote: remote.to_string(),
            credentials: credentials,
        }
    }
}

impl RevisionControlAPI for Designsync {
    fn populate(&self, version: Option<String>) -> Result<()> {
        log_info!("Started populating {}...", &self.remote);
        fs::create_dir_all(&self.local)?;
        self.set_vault()?;
        self.pop(None)?;
        Ok(())
    }

    fn populate_with_progress(
        &self,
        version: Option<String>,
        callback: &mut dyn FnMut(&Progress),
    ) -> Result<Progress> {
        log_info!("Started populating {}...", &self.remote);
        fs::create_dir_all(&self.local)?;
        self.set_vault()?;
        Ok(self.pop(Some(callback))?)
    }
}

impl Designsync {
    fn set_vault(&self) -> Result<()> {
        with_dir(&self.local, || {
            let mut child = Command::new("dssc")
                .args(&["setvault", &self.remote, "."])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let stdout = child.stdout.take().unwrap();

            let reader = BufReader::new(stdout);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| log_debug!("{}", line));

            let stderr = child.stderr.take().unwrap();

            let reader2 = BufReader::new(stderr);
            reader2
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| log_error!("{}", line));

            if child.wait()?.success() {
                Ok(())
            } else {
                Err(Error::new(&format!("Something went wrong setting the vault in '{}', see log for details", self.local.display())))
            }
        })
    }

    fn pop(&self,
          mut callback: Option<&mut dyn FnMut(&Progress)>,
          ) -> Result<Progress> {
        let mut progress = Progress::default();
        with_dir(&self.local, || {
            let mut child = Command::new("dssc")
                .args(&["pop", "-rec", "-uni", "-force"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let stdout = child.stdout.take().unwrap();

            let reader = BufReader::new(stdout);
            lazy_static! {
                static ref POP_REGEX : Regex = Regex::new(
                    r": Success - (Fetched|Created symbolic)"
                ).unwrap();
            }
            // Sorry about the duplication here, couldn't work out how to unwrap
            // the optional callback without it :-(
            match &mut callback {
                Some(f) => {
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            log_debug!("{}", line);
                            if POP_REGEX.is_match(&line) {
                                progress.received_objects += 1;
                                progress.completed_objects += 1;
                                f(&progress);
                            }
                        });
                },
                None => {
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            log_debug!("{}", line);
                            if POP_REGEX.is_match(&line) {
                                progress.received_objects += 1;
                                progress.completed_objects += 1;
                            }
                        });
                }
            }

            let stderr = child.stderr.take().unwrap();

            let reader2 = BufReader::new(stderr);
            reader2
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| log_error!("{}", line));

            if child.wait()?.success() {
                Ok(progress.clone())
            } else {
                Err(Error::new(&format!("Something went wrong populating '{}', see log for details", self.remote)))
            }
        })
    }
}
