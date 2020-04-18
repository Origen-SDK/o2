mod designsync;
mod git;

use crate::Result;
use designsync::Designsync;
use git::Git;
use std::env;
use std::path::{Path, PathBuf};

pub struct RevisionControl {
    driver: Box<dyn RevisionControlAPI>,
}

impl RevisionControl {
    pub fn new(local: &Path, remote: &str) -> RevisionControl {
        if remote.ends_with(".git") {
            RevisionControl {
                driver: Box::new(RevisionControl::git(local, remote)),
            }
        } else {
            RevisionControl {
                driver: Box::new(RevisionControl::designsync(local, remote)),
            }
        }
    }

    pub fn git(local: &Path, remote: &str) -> Git {
        Git::new(local, remote)
    }

    pub fn designsync(local: &Path, remote: &str) -> Designsync {
        Designsync::new(local, remote)
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
