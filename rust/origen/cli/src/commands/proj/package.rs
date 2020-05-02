use super::error;
use origen::revision_control::{Credentials, RevisionControl, RevisionControlAPI};
use origen::utility::{copy_dir, symlink, with_dir};
use origen::{Error, Result};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Package {
    pub id: String,
    path: Option<PathBuf>,
    version: Option<String>,
    repo: Option<String>,
    copy: Option<PathBuf>,
    link: Option<PathBuf>,
    exclude: Option<bool>,
    username: Option<String>,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl Package {
    /// Creates the package in the given workspace directory, will return an error
    /// if something goes wrong with the revision control populate operation
    pub fn create(&self, workspace_dir: &Path) -> Result<()> {
        let path = self.path(workspace_dir);
        match fs::create_dir_all(&path) {
            Ok(()) => {}
            Err(_e) => {
                return Err(Error::new(&format!(
                    "Couldn't create '{}', do you have the required permissions?",
                    path.display()
                )));
            }
        }
        if self.link.is_some() {
            self.create_from_link(&path)?;
        } else if self.copy.is_some() {
            self.create_from_copy(&path, false)?;
        } else {
            let rc = RevisionControl::new(&path, self.repo.as_ref().unwrap(), self.credentials());
            rc.populate(self.version.as_ref().unwrap())?;
        }
        log_success!("Successfully created package '{}'", self.id);
        Ok(())
    }

    /// Updates the package in the given workspace directory, will return an error
    /// if something goes wrong with the underlying revision control checkout operation,
    /// or if the package dir resolved from the BOM does not exist
    pub fn update(&self, workspace_dir: &Path) -> Result<()> {
        let path = self.path(workspace_dir);
        if !path.exists() {
            return Err(Error::new(&format!(
                "Expected to find package '{}' at '{}', but it doesn't exist",
                self.id,
                path.display()
            )));
        }
        if self.link.is_some() {
            self.create_from_link(&path)?;
        } else if self.copy.is_some() {
            self.create_from_copy(&path, false)?;
        } else {
            let rc = RevisionControl::new(&path, self.repo.as_ref().unwrap(), self.credentials());
            rc.checkout(false, None, self.version.as_ref().unwrap())?;
        }
        Ok(())
    }

    /// Returns a path to the package dir within the given workspace
    pub fn path(&self, workspace_dir: &Path) -> PathBuf {
        let mut path = workspace_dir.to_path_buf();
        match &self.path {
            None => path.push(self.id.clone()),
            Some(x) => path.push(x),
        }
        path
    }

    fn create_from_link(&self, dest: &Path) -> Result<()> {
        let source = self.link.as_ref().unwrap();
        // If the package is currently implemented in some other way, then delete it
        if dest.exists() && dest.read_link().is_err() {
            if dest.is_dir() {
                fs::remove_dir_all(dest)?;
            } else {
                fs::remove_file(dest)?;
            }
        }
        if !dest.exists() {
            if source.exists() {
                symlink(source, dest)?;
            } else {
                return Err(Error::new(&format!(
                    "The target of link '{}' for package '{}' does not exist - '{}'",
                    dest.display(),
                    self.id,
                    source.display()
                )));
            }
        }
        Ok(())
    }

    fn create_from_copy(&self, dest: &Path, recopy: bool) -> Result<()> {
        let source = self.copy.as_ref().unwrap();
        // If the package is currently implemented as a symlink, then delete it
        if dest.exists() && (dest.read_link().is_ok() || dest.is_file()) {
            fs::remove_file(dest)?;
        }
        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }
        let is_empty = dest.read_dir()?.next().is_none();
        if is_empty {
            if source.exists() {
                copy_dir(source, dest)?;
            } else {
                return Err(Error::new(&format!(
                    "The copy target for package '{}' does not exist - '{}'",
                    self.id,
                    source.display()
                )));
            }
        } else {
            // What to do if it exists and not empty?
        }
        Ok(())
    }

    fn credentials(&self) -> Option<Credentials> {
        match &self.username {
            None => None,
            Some(x) => Some(Credentials {
                username: Some(x.clone()),
                password: None,
            }),
        }
    }

    pub fn to_string(&self, indent: usize) -> String {
        let i = " ".repeat(indent);
        let mut s = format!("{}[[package]]\n", i);
        s += &format!("{}id = \"{}\"\n", i, self.id);
        if let Some(x) = &self.path {
            s += &format!("{}path = \"{}\"\n", i, x.display());
        }
        if let Some(x) = &self.version {
            s += &format!("{}version = \"{}\"\n", i, x);
        }
        if let Some(x) = &self.repo {
            s += &format!("{}repo = \"{}\"\n", i, x);
        }
        if let Some(x) = &self.username {
            s += &format!("{}username = \"{}\"\n", i, x);
        }
        if let Some(x) = &self.copy {
            s += &format!("{}copy = \"{}\"\n", i, x.display());
        }
        if let Some(x) = &self.link {
            s += &format!("{}link = \"{}\"\n", i, x.display());
        }
        s += "\n";
        s
    }

    pub fn merge(&mut self, p: &Package) {
        match &p.path {
            Some(x) => {
                self.path = Some(x.clone());
            }
            None => {}
        }
        match &p.version {
            Some(x) => {
                self.version = Some(x.clone());
            }
            None => {}
        }
        match &p.exclude {
            Some(x) => {
                self.exclude = Some(x.clone());
            }
            None => {}
        }
        match &p.username {
            Some(x) => {
                self.username = Some(x.clone());
            }
            None => {}
        }
        match &p.repo {
            Some(x) => {
                self.repo = Some(x.clone());
                self.copy = None;
                self.link = None;
            }
            None => {}
        }
        match &p.copy {
            Some(x) => {
                self.copy = Some(x.clone());
                self.repo = None;
                self.link = None;
                self.version = None;
            }
            None => {}
        }
        match &p.link {
            Some(x) => {
                self.link = Some(x.clone());
                self.repo = None;
                self.copy = None;
                self.version = None;
            }
            None => {}
        }
    }

    pub fn validate(&self) {
        if self.has_no_source() {
            error(
                &format!(
                    "Malformed BOM, package '{}' has no source defined:\n{}",
                    self.id, self
                ),
                Some(1),
            );
        }
        if self.has_missing_version() {
            error(
                &format!(
                    "Malformed BOM, package '{}' has a repository defined, but no version:\n{}",
                    self.id, self
                ),
                Some(1),
            );
        }
        if self.has_multiple_sources() {
            error(
                &format!(
                    "Malformed BOM, package '{}' has multiple sources defined:\n{}",
                    self.id, self
                ),
                Some(1),
            );
        }
    }

    fn has_no_source(&self) -> bool {
        self.repo.is_none() && self.copy.is_none() && self.link.is_none()
    }

    fn has_missing_version(&self) -> bool {
        self.repo.is_some() && self.version.is_none()
    }

    fn has_multiple_sources(&self) -> bool {
        let mut sources = 0;
        if self.repo.is_some() {
            sources += 1;
        }
        if self.copy.is_some() {
            sources += 1;
        }
        if self.link.is_some() {
            sources += 1;
        }
        sources > 1
    }
}
