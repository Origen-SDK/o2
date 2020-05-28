pub mod designsync;
pub mod git;

use crate::Result;
use designsync::Designsync;
use git::Git;
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
pub struct Status {
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
}

impl Status {
    /// Returns true if the workspace status is modified in any way
    pub fn is_modified(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }
}

impl RevisionControl {
    /// Returns a generic revision control driver which implements the RevisionControlAPI, it will work with any
    /// supported revision control tool and will work out which one to target from the remote argument.
    /// If you want to use some of the tool-specific APIs, then you should instantiate the relevant driver
    /// directly.
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
}

/// Defines a common minimum API that all revision control system drivers should support
pub trait RevisionControlAPI {
    /// Initially populate the local directory with the remote, this is equivalent to a 'git clone'
    /// or a 'dssc pop' operation.
    /// A progress instance will be returned indicating how many objects were fetched.
    /// A callback can be given which will be called periodically if the caller wants to be updated
    /// on the progress during the operation.
    fn populate(&self, version: &str) -> Result<()>;

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<()>;

    /// Returns a Status object which contains lists of all files which have local modifications.
    /// Supplying a path to a directory may be supported to limit the results to files that fall withing
    /// the given directory.
    fn status(&self, path: Option<&Path>) -> Result<Status>;
}

impl RevisionControlAPI for RevisionControl {
    fn populate(&self, version: &str) -> Result<()> {
        self.driver.populate(version)
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<()> {
        self.driver.checkout(force, path, version)
    }

    fn status(&self, path: Option<&Path>) -> Result<Status> {
        self.driver.status(path)
    }
}
