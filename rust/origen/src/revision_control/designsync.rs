use super::RevisionControlAPI;
use crate::utility::with_dir;
use crate::{Error, Result, USER};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Designsync {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote vault
    pub remote: String,
}

struct Output {
    passed: bool,
    stdout: String,
    stderr: String,
}

impl Designsync {
    pub fn new(local: &Path, remote: &str) -> Designsync {
        Designsync {
            local: local.to_path_buf(),
            remote: remote.to_string(),
        }
    }
}

impl RevisionControlAPI for Designsync {
    fn populate(&self, version: Option<String>) -> Result<()> {
        log_info!("Started populating {}...", &self.remote);
        fs::create_dir_all(&self.local)?;
        self.set_vault()?;
        //self.set_vault()?;
        Ok(())
    }
}

impl Designsync {
    fn set_vault(&self) -> Result<()> {
        with_dir(&self.local, || {
            let output = dssc(&["setvault", &self.remote, "."])?;
            if !output.passed {
                return Err(Error::new(&format!(
                    "Something went wrong setting the vault in '{}'",
                    self.local.display()
                )));
            }
            for line in output.stdout.lines() {
                log_debug!("{}", line);
            }
            log_error!("STDERR");
            for line in output.stderr.lines() {
                log_error!("{}", line);
            }
            Ok(())
        })
    }
}

fn dssc<I, S>(args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = Command::new("dssc").args(args).output()?;
    Ok(Output {
        passed: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
