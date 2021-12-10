pub mod frontend;
pub mod supported;

use crate::{Outcome, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use supported::{Designsync, Git};

#[derive(Clone, Default, Debug)]
pub struct Status {
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
    /// Files with merge conflicts
    pub conflicted: Vec<PathBuf>,
    pub revision: String,
}

impl Status {
    /// Returns true if the workspace status is modified in any way
    pub fn is_modified(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.changed.is_empty()
            || !self.conflicted.is_empty()
            || !self.added.is_empty()
    }

    pub fn summarize(&self) {
        displayln!("Workspace Status");
        if !self.added.is_empty() {
            displayln!("  ADDED: {} ITEMS", self.added.len());
            for file in &self.added {
                displayln!("    {}", file.display());
            }
        }
        if !self.removed.is_empty() {
            displayln!("  DELETED: {} ITEMS", self.removed.len());
            for file in &self.removed {
                displayln!("    {}", file.display());
            }
        }
        if !self.changed.is_empty() {
            displayln!("  CHANGED: {} ITEMS", self.changed.len());
            for file in &self.changed {
                displayln!("    {}", file.display());
            }
        }
        if !self.conflicted.is_empty() {
            displayln!("  CONFLICTED: {} ITEMS", self.conflicted.len());
            for file in &self.conflicted {
                display_redln!("    {}", file.display());
            }
        }
    }
}

pub enum SupportedSystems {
    Git,
    Designsync,
}

impl SupportedSystems {
    pub fn from_str(system: &str) -> Result<Self> {
        let s = system.to_lowercase();
        match s.as_str() {
            "git" => Ok(Self::Git),
            "design_sync" | "designsync" => Ok(Self::Designsync),
            _ => bail!("Unsupported revision control system '{}'", system),
        }
    }
}

#[derive(Clone, Default)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl std::fmt::Debug for Credentials {
    // Purposefully leave off the password
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"<-- Plaintext Password Withheld -->")
            .finish()
    }
}

#[derive(Debug)]
pub struct RevisionControl {
    driver: Box<dyn RevisionControlAPI>,
}

impl RevisionControl {
    /// Returns a generic revision control driver which implements the RevisionControlAPI, it will work with any
    /// supported revision control tool and will work out which one to target from the remote argument.
    /// If you want to use some of the tool-specific APIs, then you should instantiate the relevant driver
    /// directly.
    /// Multiple remotes can be accepted, for example for Git the ssh and https urls can be given, then it is up
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

    pub fn from_config(config: &HashMap<String, String>) -> Result<Self> {
        let driver: Box<dyn RevisionControlAPI>;
        if let Some(c) = config.get("system") {
            let _c = c.to_lowercase();
            match _c.as_str() {
                "git" => driver = Box::new(Self::git_from_config(config)?),
                "designsync" | "design_sync" => {
                    driver = Box::new(Self::designsync_from_config(config)?)
                }
                _ => bail!("Unknown RC system '{}'", _c),
            }
        } else {
            // Check for some specific parameters to discern the system
            if config.contains_key("vault") {
                if config.contains_key("remote") {
                    bail!("Both 'vault' and 'remote' cannot be used without specifying the 'system' parameter");
                } else {
                    driver = Box::new(Self::designsync_from_config(config)?);
                }
            } else if config.contains_key("remote") {
                driver = Box::new(Self::git_from_config(config)?);
            } else {
                bail!("Could not discern revision control system. None of 'remote', 'vault', or 'system' were given");
            }
        }
        Ok(Self { driver: driver })
    }

    pub fn git(local: &Path, remotes: Vec<&str>, credentials: Option<Credentials>) -> Git {
        Git::new(local, remotes, credentials)
    }

    pub fn git_from_config(config: &HashMap<String, String>) -> Result<Git> {
        Ok(Self::git(
            match config.get("local") {
                Some(l) => &Path::new(l),
                None => bail!("Git driver must be given a 'local' parameter"),
            },
            match config.get("remote") {
                Some(r) => vec![r],
                None => bail!("Git driver must be given a 'remote' parameter"),
            },
            None,
        ))
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

    pub fn designsync_from_config(config: &HashMap<String, String>) -> Result<Designsync> {
        Ok(Self::designsync(
            &Path::new(config.get("local").unwrap()),
            match config.get("vault") {
                Some(v) => vec![v],
                None => bail!("DesignSync driver muust be given a 'vault' parameter"),
            },
            None,
        ))
    }
}

/// Defines a common minimum API that all revision control system drivers should support
pub trait RevisionControlAPI: std::fmt::Debug {
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

    /// Initialize a new local workspace at path, pointing to the given location
    /// Returns true if a new workspace was created, false if the workspace already
    /// existed (no action), or an error.
    fn init(&self) -> Result<Outcome>;

    /// Indicate if the path of the RC driver is initialized
    fn is_initialized(&self) -> Result<bool>;

    fn checkin(
        &self,
        files_or_dirs: Option<Vec<&Path>>,
        msg: &str,
        dry_run: bool,
    ) -> Result<Outcome>;

    fn system(&self) -> &str;
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

    fn init(&self) -> Result<Outcome> {
        self.driver.init()
    }

    fn is_initialized(&self) -> Result<bool> {
        self.driver.is_initialized()
    }

    fn checkin(
        &self,
        files_or_dirs: Option<Vec<&Path>>,
        msg: &str,
        dry_run: bool,
    ) -> Result<Outcome> {
        self.driver.checkin(files_or_dirs, msg, dry_run)
    }

    fn system(&self) -> &str {
        self.driver.system()
    }
}
