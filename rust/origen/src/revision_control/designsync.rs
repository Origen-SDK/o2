use super::{Credentials, Progress, RevisionControlAPI};
use crate::utility::with_dir;
use crate::{Error, Result, USER};
use std::{fs};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use regex::Regex;

const DEFAULT_VERSION: &str = "Trunk";

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
    fn populate(
        &self,
        version: Option<String>,
        callback: Option<&mut dyn FnMut(&Progress)>,
    ) -> Result<Progress> {
        log_info!("Started populating {}...", &self.remote);
        fs::create_dir_all(&self.local)?;
        self.set_vault()?;
        Ok(self.pop(version, callback)?)
    }
}

fn log_stdout_and_stderr(process: &mut std::process::Child,
      stdout_callback: Option<&mut dyn FnMut(&str)>,
      stderr_callback: Option<&mut dyn FnMut(&str)>
  ) {
    log_stdout(process, stdout_callback);
    log_stderr(process, stderr_callback);
}

fn log_stdout(process: &mut std::process::Child,
              mut callback: Option<&mut dyn FnMut(&str)>
              ) {
    let stdout = process.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| { 
            log_debug!("{}", line);
            if let Some(f) = &mut callback {
                f(&line);
            }
        });
}

fn log_stderr(process: &mut std::process::Child,
              mut callback: Option<&mut dyn FnMut(&str)>
              ) {
    let stdout = process.stderr.take().unwrap();
    let reader = BufReader::new(stdout);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            log_error!("{}", line);
            if let Some(f) = &mut callback {
                f(&line);
            }
        });
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
                Err(Error::new(&format!("Something went wrong setting the vault in '{}', see log for details", self.local.display())))
            }
        })
    }

    fn pop(&self,
          version: Option<String>,
          mut callback: Option<&mut dyn FnMut(&Progress)>,
          ) -> Result<Progress> {
        let mut progress = Progress::default();
        with_dir(&self.local, || {
            let mut args = vec!["pop", "-rec", "-uni", "-force", "-version"];
            match &version {
                Some(x) => args.push(x),
                None => args.push(DEFAULT_VERSION),
            }
            let mut process = Command::new("dssc")
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            lazy_static! {
                static ref POP_REGEX : Regex = Regex::new(
                    r": Success - (Fetched|Created symbolic)"
                ).unwrap();
            }

            log_stdout_and_stderr(&mut process, Some(&mut |line: &str| {
                if POP_REGEX.is_match(&line) {
                    progress.received_objects += 1;
                    progress.completed_objects += 1;
                    if let Some(f) = &mut callback {
                        f(&progress);
                    }
                }
            }), None);

            if process.wait()?.success() {
                Ok(progress.clone())
            } else {
                Err(Error::new(&format!("Something went wrong populating '{}', see log for details", self.remote)))
            }
        })
    }
}
