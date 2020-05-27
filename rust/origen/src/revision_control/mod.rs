mod designsync;
mod git;

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

    /// Returns a vector of files which have local modifications. Optionally a path to a directory
    /// within the local workspace can be given and in that case only mods within that directory
    /// will be returned.
    fn local_mods(&self, path: Option<&Path>) -> Result<Vec<PathBuf>>;

    /// Returns true if the local workspace contains modifications
    fn has_local_mods(&self) -> Result<bool> {
        Ok(!self.local_mods(None)?.is_empty())
    }
}

impl RevisionControlAPI for RevisionControl {
    fn populate(&self, version: &str) -> Result<()> {
        self.driver.populate(version)
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<()> {
        self.driver.checkout(force, path, version)
    }

    fn local_mods(&self, path: Option<&Path>) -> Result<Vec<PathBuf>> {
        self.driver.local_mods(path)
    }
}
