use crate::file::FilePermissions;
use crate::Result;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

use super::SessionStore;
use super::DEFAULT_FILE_PERMISSIONS;


#[derive(PartialEq, Debug)]
pub struct SessionGroup {
    path: PathBuf,
    sessions: IndexMap<String, SessionStore>,
    file_permissions: FilePermissions
}

impl SessionGroup {
    pub fn new(name: &str, root: &Path, file_permissions: Option<FilePermissions>) -> Result<Self> {
        let mut sg = Self {
            path: {
                let mut r = root.to_path_buf();
                r.push(name);
                r
            },
            sessions: IndexMap::new(),
            file_permissions: file_permissions.unwrap_or_else(|| DEFAULT_FILE_PERMISSIONS.clone())
        };
        sg.populate()?;
        Ok(sg)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.path.file_stem().unwrap().to_os_string().into_string()?)
    }

    pub fn get(&self, name: &str) -> Option<&SessionStore> {
        self.sessions.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut SessionStore> {
        self.sessions.get_mut(name)
    }

    // TODO use entry? Better performance?
    pub fn get_or_add(&mut self, name: &str) -> Result<&SessionStore> {
        Ok(match self.sessions.contains_key(name) {
            true => self.sessions.get(name).unwrap(),
            false => {
                let s = self.add_session(name)?;
                s
            }
        })
    }

    pub fn ensure(&mut self, name: &str) -> Result<bool> {
        Ok(if self.sessions.contains_key(name) {
            false
        } else {
            self.add_session(name)?;
            true
        })
    }

    pub fn require(&self, name: &str) -> Result<&SessionStore> {
        match self.sessions.get(name) {
            Some(s) => Ok(s),
            None => bail!("Session {} has not been added to group {} yet!", name, self.name()?)
        }
    }

    pub fn require_mut(&mut self, name: &str) -> Result<&mut SessionStore> {
        let n = self.name()?;
        match self.sessions.get_mut(name) {
            Some(s) => Ok(s),
            None => bail!("Session {} has not been added to group {} yet!", name, n)
        }
    }

    pub fn add_session(&mut self, name: &str) -> Result<&SessionStore> {
        self.sessions.insert(name.to_string(), SessionStore::new(name, &self.path, Some(self.name()?), Some(self.file_permissions.clone()))?);
        Ok(self.sessions.get(name).unwrap())
    }

    pub fn sessions(&self) -> &IndexMap<String, SessionStore> {
        &self.sessions
    }

    pub fn delete_session(&mut self, name: &str) -> Result<bool> {
        match self.sessions.remove(name) {
            Some(s) => {
                s.remove_file()?;
                Ok(true)
            },
            None => Ok(false)
        }
    }

    // TESTS_NEEDED
    pub fn clean(&mut self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_dir_all(&self.path)?;
        }
        self.sessions = IndexMap::new();
        Ok(())
    }

    // TESTS_NEEDED
    // TODO add some auto-populate?
    pub fn refresh(&self) -> Result<()> {
        for (_n, s) in self.sessions.iter() {
            s.refresh()?;
        }
        Ok(())
    }

    fn populate(&mut self) -> Result<()> {
        if self.path.exists() {
            match self.path.read_dir() {
                Ok(iter) => {
                    for f in iter {
                        self.add_session(&f?.file_name().into_string()?)?;
                    }
                },
                Err(e) => {
                    bail!(&format!(
                        "Unable to populate session group '{}' from existing path: '{}'. Encountered Error: {}",
                        self.name()?,
                        self.path.display(),
                        e
                    ))
                }
            }
        }
        Ok(())
    }

    pub fn file_permissions(&self) -> &FilePermissions {
        &self.file_permissions
    }
}
