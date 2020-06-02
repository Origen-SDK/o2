use super::error_and_exit;
use flate2::read::GzDecoder;
use origen::revision_control::{Credentials, RevisionControl, RevisionControlAPI};
use origen::utility::file_utils::{copy, copy_contents, mv, symlink};
use origen::Result;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fmt, fs};
use tar::Archive;
use tempfile::tempdir;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Package {
    pub id: String,
    pub path: Option<PathBuf>,
    version: Option<String>,
    repo: Option<String>,
    copy: Option<PathBuf>,
    link: Option<PathBuf>,
    username: Option<String>,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Package {
    /// Returns true if the pacakge is currently defined as a repo
    pub fn is_repo(&self) -> bool {
        self.repo.is_some()
    }

    /// Creates the package in the given workspace directory, will return an error
    /// if something goes wrong with the revision control populate operation
    pub fn create(&self, workspace_dir: &Path) -> Result<()> {
        let path = self.path(workspace_dir);
        match fs::create_dir_all(&path) {
            Ok(()) => {}
            Err(_e) => {
                return error!(
                    "Couldn't create '{}', do you have the required permissions?",
                    path.display()
                );
            }
        }
        if self.link.is_some() {
            self.create_from_link(&path, true)?;
        } else if self.copy.is_some() {
            self.create_from_copy(&path, true)?;
        } else {
            let rc = self.rc(workspace_dir).unwrap();
            rc.populate(self.version.as_ref().unwrap())?;
        }
        log_success!("Successfully created package '{}'", self.id);
        Ok(())
    }

    /// Updates the package in the given workspace directory, will return an error
    /// if something goes wrong with the underlying revision control checkout operation,
    /// or if the package dir resolved from the BOM does not exist
    pub fn update(&self, workspace_dir: &Path, force: bool) -> Result<(bool, bool)> {
        let mut force_required = false;
        let mut conflicts = false;
        let path = self.path(workspace_dir);
        // If the package is currently defined as a link
        if self.link.is_some() {
            log_trace!("Package '{}' is currently defined as a link", self.id);
            force_required = self.create_from_link(&path, force)?;
        // If the package is currently defined as copy
        } else if self.copy.is_some() {
            log_trace!("Package '{}' is currently defined as a copy", self.id);
            force_required = self.create_from_copy(&path, force)?;
        // If the package is currently defined as a revision control reference
        } else {
            // If currently defined as a symlink, then delete it
            if path.exists() && path.read_link().is_ok() {
                log_trace!(
                    "Package '{}' is currently defined as a symlink, deleting it",
                    self.id
                );
                fs::remove_file(&path)?;
                fs::create_dir_all(&path)?;
            }
            if !path.exists() {
                return error!(
                    "Expected to find package '{}' at '{}', but it doesn't exist",
                    self.id,
                    path.display()
                );
            }
            let rc = self.rc(workspace_dir).unwrap();
            let is_empty = path.read_dir()?.next().is_none();
            if is_empty {
                log_trace!("Populating package '{}' from revision control", self.id);
                rc.populate(self.version.as_ref().unwrap())?;
            } else {
                log_trace!("Checking out package '{}' from revision control", self.id);
                conflicts = rc.checkout(force, None, self.version.as_ref().unwrap())?;
            }
        }
        Ok((force_required, conflicts))
    }

    /// Returns a revision control driver for the package, if applicable
    pub fn rc(&self, workspace_dir: &Path) -> Option<RevisionControl> {
        if self.is_repo() {
            let path = self.path(workspace_dir);
            Some(RevisionControl::new(
                &path,
                self.repo.as_ref().unwrap(),
                self.credentials(),
            ))
        } else {
            None
        }
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

    fn create_from_link(&self, dest: &Path, force: bool) -> Result<bool> {
        let mut force_required = false;
        let source = self.link.as_ref().unwrap();
        if dest.exists() {
            match dest.read_link() {
                // If a symlink, just delete and re-create it to ensure it matches the latest target, don't
                // need to worry about losing data in this case so don't bother the user about forcing
                Ok(_) => {
                    fs::remove_file(dest)?;
                }
                Err(_) => {
                    if force {
                        if dest.is_dir() {
                            fs::remove_dir_all(dest)?;
                        } else {
                            fs::remove_file(dest)?;
                        }
                    } else {
                        display_redln!("ERROR");
                        log_error!(
                            "Package is currently defined as a link but a previous (non-linked) definition exists in the workspace, --force is required to replace it"
                        );
                        force_required = true;
                    }
                }
            }
        }
        if !dest.exists() {
            if source.exists() {
                symlink(source, dest)?;
            } else {
                return error!(
                    "The target of link '{}' for package '{}' does not exist - '{}'",
                    dest.display(),
                    self.id,
                    source.display()
                );
            }
        }
        Ok(force_required)
    }

    fn create_from_copy(&self, dest: &Path, _force: bool) -> Result<bool> {
        let source = self.copy.as_ref().unwrap();
        // If the package is currently implemented as a symlink then delete it, don't
        // need to worry about losing data in this case so don't bother the user about forcing
        if dest.exists() && dest.read_link().is_ok() {
            fs::remove_file(dest)?;
        }
        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }
        if dest.is_dir() {
            let is_empty = dest.read_dir()?.next().is_none();
            if is_empty {
                if source.exists() {
                    if source.is_file() {
                        fs::remove_dir_all(dest)?;
                        let temp = tempdir()?;
                        let temp_file = temp.path().join("temp_file");
                        copy(source, &temp_file)?;
                        let f = File::open(&temp_file)?;
                        let gz = GzDecoder::new(f.try_clone()?);
                        // If the file is g-zipped
                        let result;
                        let unpacked = temp.path().join("unpacked");
                        if gz.header().is_some() {
                            let mut archive = Archive::new(gz);
                            result = archive.unpack(&unpacked);
                        } else {
                            let mut archive = Archive::new(f);
                            result = archive.unpack(&unpacked);
                        }
                        if result.is_ok() && unpacked.exists() {
                            let paths = fs::read_dir(&unpacked).unwrap();
                            if paths.count() == 1 {
                                let paths = fs::read_dir(&unpacked).unwrap();
                                mv(&paths.last().unwrap()?.path(), dest)?;
                            } else {
                                mv(&unpacked, dest)?;
                            }
                        } else {
                            mv(&temp_file, dest)?;
                        }
                    } else {
                        copy_contents(source, dest)?;
                    }
                } else {
                    return error!(
                        "The copy target for package '{}' does not exist - '{}'",
                        self.id,
                        source.display()
                    );
                }
            } else {
                // What to do when updating a copied directory?
            }
        } else {
            // What to do when updating a copied file?
        }
        Ok(false)
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
            error_and_exit(
                &format!(
                    "Malformed BOM, package '{}' has no source defined:\n{}",
                    self.id, self
                ),
                Some(1),
            );
        }
        if self.has_missing_version() {
            error_and_exit(
                &format!(
                    "Malformed BOM, package '{}' has a repository defined, but no version:\n{}",
                    self.id, self
                ),
                Some(1),
            );
        }
        if self.has_multiple_sources() {
            error_and_exit(
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
