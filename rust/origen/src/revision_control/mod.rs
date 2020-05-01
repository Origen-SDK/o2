mod designsync;
mod git;

use crate::Result;
use designsync::Designsync;
use git::Git;
use std::path::Path;

pub struct RevisionControl {
    driver: Box<dyn RevisionControlAPI>,
}

#[derive(Clone, Default)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
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
}

impl RevisionControlAPI for RevisionControl {
    fn populate(&self, version: &str) -> Result<()> {
        self.driver.populate(version)
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<()> {
        self.driver.checkout(force, path, version)
    }
}
