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

#[derive(Clone, Default)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct Progress {
    /// The total number of objects (generally meaning files), that are expected during a populate
    /// or update operation, it may not be possible to determine this with some revision control systems
    pub total_objects: Option<usize>,
    /// The number of objects (generally meaning files), that have been received during a populate
    /// or update operation
    pub received_objects: usize,
    /// The number of objects which have been received and finished processing.
    /// This will be the same as received objects for revision control tools which receive and process
    /// a file at a time vs. receiving as a batch and then locally processing.
    pub completed_objects: usize,
}

impl RevisionControl {
    pub fn new(local: &Path, remote: &str, credentials: Option<Credentials>) -> RevisionControl {
        if remote.ends_with(".git") {
            RevisionControl {
                driver: Box::new(RevisionControl::git(local, remote, credentials)),
            }
        } else {
            RevisionControl {
                driver: Box::new(RevisionControl::designsync(local, remote, credentials)),
            }
        }
    }

    pub fn git(local: &Path, remote: &str, credentials: Option<Credentials>) -> Git {
        Git::new(local, remote, credentials)
    }

    pub fn designsync(local: &Path, remote: &str, credentials: Option<Credentials>) -> Designsync {
        Designsync::new(local, remote, credentials)
    }

    //pub fn with_dir<T, F>(path: &Path, mut f: F) -> Result<T>
    //where
    //    F: FnMut() -> Result<T>,
    //{
    //    let orig = env::current_dir()?;
    //    env::set_current_dir(path)?;
    //    let result = f();
    //    env::set_current_dir(&orig)?;
    //    result
    //}
}

/// Defines a common minimum API that all revision control system drivers should support
pub trait RevisionControlAPI {
    /// Initially populate the local directory with the remote
    fn populate(&self, version: Option<String>) -> Result<()>;

    fn populate_with_progress(
        &self,
        version: Option<String>,
        callback: &mut dyn FnMut(&Progress),
    ) -> Result<Progress>;
}

impl RevisionControlAPI for RevisionControl {
    fn populate(&self, version: Option<String>) -> Result<()> {
        self.driver.populate(version)
    }

    fn populate_with_progress(
        &self,
        version: Option<String>,
        callback: &mut dyn FnMut(&Progress),
    ) -> Result<Progress> {
        self.driver.populate_with_progress(version, callback)
    }
}
