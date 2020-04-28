use super::{error, BOM_FILE};
use indexmap::IndexMap;
use origen::revision_control::{Credentials, Progress, RevisionControl, RevisionControlAPI};
use origen::{Error, Result};
use std::path::{Path, PathBuf};
use std::{env, fmt, fs};

#[derive(Debug, Deserialize)]
// This is a temporary structure to make the BOM file syntax nicer for users.
// It will be quickly converted to a BOM which is structured better for us.
pub struct TempBOM {
    meta: Option<Meta>,
    package: Option<Vec<Package>>,
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
}

impl fmt::Display for BOM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = "Packages:\n".to_string();
        for (_id, p) in self.packages.iter() {
            s += &p.to_string(2);
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
        self.meta.merge(&bom.meta);
    }

    fn validate(&self) {
        for (_id, p) in self.packages.iter() {
            p.validate();
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Package {
    id: String,
    path: Option<PathBuf>,
    group_id: Option<String>,
    version: Option<String>,
    repo: Option<String>,
    copy: Option<PathBuf>,
    link: Option<PathBuf>,
    update: Option<bool>,
    username: Option<String>,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl Package {
    // Creates the package view in the current workspace
    pub fn create(&mut self) -> Result<()> {
        let mut path = env::current_dir().expect("Something has gone wrong trying to resolve the PWD, is it stale or do you not have read access to it?");
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
        let credentials = match &self.username {
            None => None,
            Some(x) => Some(Credentials {
                username: Some(x.clone()),
                password: None,
            }),
        };
        let rc = RevisionControl::new(&path, self.repo.as_ref().unwrap(), credentials);
        let final_progress = rc.populate(self.version.clone(), Some(&mut |progress| {
            self.print_progress(progress, false);
        }))?;
        self.print_progress(&final_progress, true);
        log_success!("Successfully fetched package '{}'", self.id);
        Ok(())
    }

    fn print_progress(&self, progress: &Progress, last: bool) {
        let msg = match progress.total_objects {
            None => format!(
                "Populating package '{}', fetched {} objects", self.id,
                progress.completed_objects
            ),
            Some(n) => format!(
                "Populating package '{}', fetched {}/{} objects", self.id,
                progress.completed_objects, n
            ),
        };
        if last {
            print!("{} ", msg);
        } else {
            print!("{}\r", msg);
        }
    }

    fn to_string(&self, indent: usize) -> String {
        let i = " ".repeat(indent);
        let mut s = format!("{}'{}':\n", i, self.id);
        if let Some(x) = &self.path {
            s += &format!("{}  path:     {}\n", i, x.display());
        }
        if let Some(x) = &self.group_id {
            s += &format!("{}  proup_id: {}\n", i, x);
        }
        if let Some(x) = &self.version {
            s += &format!("{}  version:  {}\n", i, x);
        }
        if let Some(x) = &self.repo {
            s += &format!("{}  repo:     {}\n", i, x);
        }
        if let Some(x) = &self.username {
            s += &format!("{}  username: {}\n", i, x);
        }
        if let Some(x) = &self.copy {
            s += &format!("{}  copy:     {}\n", i, x.display());
        }
        if let Some(x) = &self.link {
            s += &format!("{}  link:     {}\n", i, x.display());
        }
        if let Some(x) = &self.update {
            s += &format!("{}  update:   {}\n", i, x);
        } else {
            s += &format!("{}  update:   true\n", i);
        }
        s
    }

    fn merge(&mut self, p: &Package) {
        match &p.path {
            Some(x) => {
                self.path = Some(x.clone());
            }
            None => {}
        }
        match &p.group_id {
            Some(x) => {
                self.group_id = Some(x.clone());
            }
            None => {}
        }
        match &p.version {
            Some(x) => {
                self.version = Some(x.clone());
            }
            None => {}
        }
        match &p.update {
            Some(x) => {
                self.update = Some(x.clone());
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
            }
            None => {}
        }
        match &p.link {
            Some(x) => {
                self.link = Some(x.clone());
                self.repo = None;
                self.copy = None;
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
