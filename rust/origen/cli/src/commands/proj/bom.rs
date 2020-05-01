use super::{error, BOM_FILE};
use indexmap::IndexMap;
use origen::revision_control::{Credentials, RevisionControl, RevisionControlAPI};
use origen::{Error, Result};
use origen::utility::with_dir;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[derive(Debug, Deserialize)]
// This is a temporary structure to make the BOM file syntax nicer for users.
// It will be quickly converted to a BOM which is structured better for us.
pub struct TempBOM {
    meta: Option<Meta>,
    package: Option<Vec<Package>>,
    links: Option<IndexMap<String, String>>,
}

impl TempBOM {
    fn to_bom(&self) -> Result<BOM> {
        let mut bom = BOM {
            meta: match &self.meta {
                Some(x) => x.clone(),
                None => Meta::default(),
            },
            files: vec![],
            packages: IndexMap::new(),
            links: IndexMap::new(),
        };
        if let Some(packages) = &self.package {
            for p in packages.iter() {
                if bom.packages.contains_key(&p.id) {
                    return Err(Error::new(&format!(
                        "Duplicate package definition found: '{}'",
                        p.id
                    )));
                }
                bom.packages.insert(p.id.clone(), p.clone());
            }
        }
        if let Some(links) = &self.links {
            for (k,v) in links.iter() {
                if bom.links.contains_key(k) {
                    return Err(Error::new(&format!(
                        "Duplicate link definition found: '{}'",
                        k
                    )));
                }
                bom.links.insert(k.clone(), v.clone());
            }
        }
        Ok(bom)
    }
}

#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct Meta {
    pub workspace: bool,
}

impl Meta {
    fn merge(&mut self, meta: &Meta) {
        if meta.workspace {
            self.workspace = true;
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BOM {
    pub meta: Meta,
    pub files: Vec<PathBuf>,
    pub packages: IndexMap<String, Package>,
    pub links: IndexMap<String, String>,
}

impl fmt::Display for BOM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = "".to_string();
        if self.links.len() > 0 { 
            s += "[links]\n";
            for (k,v) in self.links.iter() {
                s += &format!("\"{}\" = \"{}\"\n", k, v);
            }
            s+= "\n";
        }
        for (_id, p) in self.packages.iter() {
            s += &p.to_string(0);
        }
        write!(f, "{}", s)
    }
}

impl BOM {
    pub fn for_dir(dir: &Path) -> BOM {
        let mut reached_root = false;
        let mut path = dir.to_path_buf();
        let mut bom = BOM {
            meta: Meta::default(),
            files: vec![],
            packages: IndexMap::new(),
            links: IndexMap::new(),
        };

        let mut bom_files: Vec<PathBuf> = vec![];
        while !reached_root {
            let p = path.join(BOM_FILE);
            if p.exists() {
                bom_files.push(p.clone());
            }
            if !path.pop() {
                reached_root = true;
            }
        }

        for f in bom_files.iter().rev() {
            let content = match fs::read_to_string(&f) {
                Ok(x) => x,
                Err(e) => {
                    error(
                        &format!(
                            "There was a problem reading BOM file '{}':\n{}",
                            f.display(),
                            e
                        ),
                        Some(1),
                    );
                    unreachable!()
                }
            };
            let new_bom: TempBOM = match toml::from_str(&content) {
                Ok(x) => x,
                Err(e) => {
                    error(
                        &format!("Malformed BOM file '{}':\n{}", f.display(), e),
                        Some(1),
                    );
                    unreachable!()
                }
            };
            match new_bom.to_bom() {
                Ok(x) => bom.merge(x, &f),
                Err(e) => {
                    error(
                        &format!("Malformed BOM file '{}': \n{}", f.display(), e.msg),
                        Some(1),
                    );
                    unreachable!()
                }
            }
        }
        bom.validate();
        bom
    }

    fn merge(&mut self, bom: BOM, source: &Path) {
        self.files.push(source.to_path_buf());
        for (id, p) in bom.packages.iter() {
            if self.packages.contains_key(id) {
                self.packages.get_mut(id).unwrap().merge(p);
            } else {
                self.packages.insert(id.clone(), p.clone());
            }
        }
        for (k, v) in bom.links.iter() {
            let _ = self.links.insert(k.clone(), v.clone());
        }
        self.meta.merge(&bom.meta);
    }

    fn validate(&self) {
        for (_id, p) in self.packages.iter() {
            p.validate();
        }
    }

    /// Returns true if the BOM belongs to a workspace
    pub fn is_workspace(&self) -> bool {
        self.meta.workspace
    }

    /// Returns an absolute path to the workspace top-level directory
    pub fn root(&self) -> &Path {
        self.files.last().unwrap().parent().unwrap()
    }

    pub fn create_links(&self) -> Result<()> {
        with_dir(self.root(), || {
            for (dest, source) in self.links.iter() {
                let dest = Path::new(dest);
                let source = Path::new(source);
                if !dest.exists() {
                    if source.exists() {
                        //if cfg!(target_os = "windows") {
                        //    if source.is_dir() {
                        //       std::os::windows::fs::symlink_dir(source, dest)?;
                        //    } else {
                        //       std::os::windows::fs::symlink_file(source, dest)?;
                        //    }
                        //} else {
                           std::os::unix::fs::symlink(source, dest)?;
                        //}

                    } else {
                        return Err(Error::new(&format!("The target of link '{}' does not exist - '{}'",
                                                      dest.display(), source.display())));
                    }
                }
            }
            Ok(())
        })
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Package {
    id: String,
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
        let mut path = workspace_dir.to_path_buf();
        match &self.path {
            None => path.push(self.id.clone()),
            Some(x) => path.push(x),
        }
        match fs::create_dir_all(&path) {
            Ok(()) => {}
            Err(_e) => {
                return Err(Error::new(&format!(
                    "Couldn't create '{}', do you have the required permissions?",
                    path.display()
                )));
            }
        }
        let rc = RevisionControl::new(&path, self.repo.as_ref().unwrap(), self.credentials());
        rc.populate(self.version.as_ref().unwrap())?;
        log_success!("Successfully created package '{}'", self.id);
        Ok(())
    }

    /// Updates the package in the given workspace directory, will return an error
    /// if something goes wrong with the underlying revision control checkout operation,
    /// or if the package dir resolved from the BOM does not exist
    pub fn update(&self, workspace_dir: &Path) -> Result<()> {
        let mut path = workspace_dir.to_path_buf();
        match &self.path {
            None => path.push(self.id.clone()),
            Some(x) => path.push(x),
        }
        if !path.exists() {
            return Err(Error::new(&format!(
                "Expected to find package '{}' at '{}', but it doesn't exist",
                self.id,
                path.display()
            )));
        }
        let rc = RevisionControl::new(&path, self.repo.as_ref().unwrap(), self.credentials());
        rc.checkout(false, None, self.version.as_ref().unwrap())?;
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

    fn to_string(&self, indent: usize) -> String {
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

    fn merge(&mut self, p: &Package) {
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

    fn validate(&self) {
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
