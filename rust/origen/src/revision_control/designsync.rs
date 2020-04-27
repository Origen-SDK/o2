use super::{Credentials, Progress, RevisionControlAPI};
use crate::utility::with_dir;
use crate::{Error, Result, USER};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Designsync {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote vault
    pub remote: String,
    credentials: Option<Credentials>,
}

struct Output {
    passed: bool,
    stdout: String,
    stderr: String,
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
        self.pop()?;
        Ok(())
    }

    fn populate_with_progress(
        &self,
        version: Option<String>,
        callback: &mut dyn FnMut(&Progress),
    ) -> Result<Progress> {
        Ok(Progress::default())
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
            //child.try_wait();

            println!("Exit status: {:?}", child.wait()?.success());

            Ok(())
        })
    }

    fn pop(&self) -> Result<()> {
        with_dir(&self.local, || {
            let mut child = Command::new("dssc")
                .args(&["pop", "-rec", "-uni", "-force"])
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
            //child.try_wait();

            println!("Exit status: {:?}", child.wait()?.success());

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
