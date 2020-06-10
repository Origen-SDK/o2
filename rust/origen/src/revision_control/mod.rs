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
    /// Files with merge conflicts
    pub conflicted: Vec<PathBuf>,
}

impl Status {
    /// Returns true if the workspace status is modified in any way
    pub fn is_modified(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.changed.is_empty()
            || !self.conflicted.is_empty()
    }
}

impl RevisionControl {
    /// Returns a generic revision control driver which implements the RevisionControlAPI, it will work with any
    /// supported revision control tool and will work out which one to target from the remote argument.
    /// If you want to use some of the tool-specific APIs, then you should instantiate the relevant driver
    /// directly.
    /// Mutliple remotes can be accepted, for example for Git the ssh and https urls can be given, then it is up
    /// to the driver to select the first one that works for the current user at runtime.
    pub fn new(
        local: &Path,
        remotes: Vec<&str>,
        credentials: Option<Credentials>,
    ) -> RevisionControl {
        if remotes.iter().any(|r| r.ends_with(".git")) {
            RevisionControl {
                driver: Box::new(RevisionControl::git(local, remotes, credentials)),
            }
        } else {
            RevisionControl {
                driver: Box::new(RevisionControl::designsync(local, remotes, credentials)),
            }
        }
    }

    pub fn git(local: &Path, remotes: Vec<&str>, credentials: Option<Credentials>) -> Git {
        Git::new(local, remotes, credentials)
    }

    pub fn designsync(
        local: &Path,
        remotes: Vec<&str>,
        credentials: Option<Credentials>,
    ) -> Designsync {
        if remotes.len() > 1 {
            log_warning!("Multiple remotes were given to the DesignSync driver, but only the first one is currently used");
        }
        Designsync::new(local, remotes[0], credentials)
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

    /// Checkout the given version of the repository.
    /// If force is false then local modifications will be preserved and merged with any upstream changes.
    /// If merge conflicts are encountered then this method will return Ok(true)
    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<bool>;

    /// Reverts any local changes.
    /// Supplying a path to a directory may be supported to limit the results to files that fall withing
    /// the given directory.
    fn revert(&self, path: Option<&Path>) -> Result<()>;

    /// Returns a Status object which contains lists of all files which have local modifications.
    /// Supplying a path to a directory may be supported to limit the results to files that fall withing
    /// the given directory.
    fn status(&self, path: Option<&Path>) -> Result<Status>;

    /// Tag the view in the local workspace. A tag message can be supplied, but this may or may not be
    /// applied to the repo depending on whether the underlying system supports it.
    /// Supplying force: true will replace any existing tag with the same name.
    fn tag(&self, tagname: &str, force: bool, message: Option<&str>) -> Result<()>;
}

impl RevisionControlAPI for RevisionControl {
    fn populate(&self, version: &str) -> Result<()> {
        self.driver.populate(version)
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> Result<bool> {
        self.driver.checkout(force, path, version)
    }

    fn revert(&self, path: Option<&Path>) -> Result<()> {
        self.driver.revert(path)
    }

    fn status(&self, path: Option<&Path>) -> Result<Status> {
        self.driver.status(path)
    }

    fn tag(&self, tagname: &str, force: bool, message: Option<&str>) -> Result<()> {
        self.driver.tag(tagname, force, message)
    }
}
