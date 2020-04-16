mod git;

use crate::Result;
use std::path::{Path, PathBuf};
pub use git::Git;

pub struct RevisionControl {
    driver: Box<dyn RevisionControlAPI>,
}

impl RevisionControl {
    pub fn new(local: &Path, remote: &str) -> RevisionControl {
        RevisionControl {
            driver: Box::new(RevisionControl::git(local, remote)),
        }
    }

    pub fn git(local: &Path, remote: &str) -> Git {
        Git::new(local, remote)
    }
}

/// Defines a common minimum API that all revision control system drivers should support
pub trait RevisionControlAPI {
    /// Initially populate the local directory with the remote
    fn populate(&self, version: Option<String>) -> Result<()>;
}

impl RevisionControlAPI for RevisionControl {
    fn populate(&self, version: Option<String>) -> Result<()> {
        self.driver.populate(version)
    }
}