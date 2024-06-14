use super::Maillist;
use crate::Result;
use crate::_utility::file_utils::resolve_relative_paths_to_strings;
use indexmap::IndexMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MaillistsTOMLConfig {
    pub class: Option<String>,
    pub directories: Option<Vec<String>>,
    src_dir: Option<PathBuf>,
}

impl std::convert::From<MaillistsTOMLConfig> for config::ValueKind {
    fn from(_value: MaillistsTOMLConfig) -> Self {
        Self::Nil
    }
}

impl MaillistsTOMLConfig {
    pub fn set_source_dir(&mut self, src_dir: PathBuf) {
        self.src_dir = Some(src_dir);
    }

    pub fn resolve_dirs(&self) -> Vec<String> {
        if let Some(directories) = self.directories.as_ref() {
            if let Some(src) = self.src_dir.as_ref() {
                resolve_relative_paths_to_strings(directories, src)
            } else {
                directories.clone()
            }
        } else {
            vec![]
        }
    }

    pub fn empty() -> Self {
        Self {
            class: None,
            directories: None,
            src_dir: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Maillists {
    pub name: String,
    pub maillists: IndexMap<String, Maillist>,
    pub directories: Vec<PathBuf>,
}

impl Maillists {
    pub fn new(
        name: String,
        directories: Option<Vec<PathBuf>>,
        continue_on_error: bool,
    ) -> Result<Self> {
        let mut m = Self {
            name: name,
            directories: Vec::new(),
            maillists: IndexMap::new(),
        };

        if let Some(dirs) = directories {
            for d in dirs {
                if d.is_file() {
                    m.add_ml_from_file(d, continue_on_error)?;
                } else {
                    m.pop_maillists_from_dir(d, continue_on_error)?;
                }
            }
        }

        Ok(m)
    }

    pub fn get_maillist(&self, m: &str) -> Result<&Maillist> {
        if let Some(ml) = self.maillists.get(m) {
            Ok(ml)
        } else {
            bail!("No maillist named '{}' found!", m)
        }
    }

    pub fn maillists_for(&self, audience: &str) -> Result<IndexMap<&str, &Maillist>> {
        let mut retn: IndexMap<&str, &Maillist> = IndexMap::new();
        let aud = Maillist::map_audience(audience).unwrap_or(audience.to_string());
        for (name, mlist) in self.maillists.iter() {
            if let Some(a) = mlist.audience.as_ref() {
                if a == &aud {
                    retn.insert(name, mlist);
                }
            }
        }
        Ok(retn)
    }

    fn add_ml_from_file(&mut self, path: PathBuf, continue_on_error: bool) -> Result<()> {
        match Maillist::from_file(&path) {
            Ok(ml) => {
                if let Some(orig_ml) = self.maillists.get(&ml.name) {
                    log_info!(
                        "Replacing maillist at '{}' with maillist at '{}'",
                        orig_ml.name,
                        ml.name
                    )
                }
                self.maillists.insert(ml.name.clone(), ml);
            }
            Err(err) => {
                if continue_on_error {
                    log_error!("{}", err);
                } else {
                    bail!(&err.msg)
                }
            }
        }
        Ok(())
    }

    fn pop_maillists_from_dir(&mut self, path: PathBuf, continue_on_error: bool) -> Result<()> {
        if !path.exists() {
            let err_msg = format!("Cannot find maillist path '{}'", path.display());
            if continue_on_error {
                log_error!("{}", err_msg);
            } else {
                bail!(&err_msg);
            }
        }

        // The order of this loop matters as a ".maillists.toml" will overwrite a ".maillists"
        for ext in ["maillist", "maillist.toml"].iter() {
            match glob::glob(&format!("{}/*.{}", path.display(), ext)) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(e) => self.add_ml_from_file(e, continue_on_error)?,
                            Err(err) => {
                                let err_msg = format!(
                                    "Error accessing maillist at '{}': {}",
                                    err.path().display(),
                                    err
                                );
                                if continue_on_error {
                                    log_error!("{}", err_msg);
                                } else {
                                    bail!(&err_msg)
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    let err_msg = format!(
                        "Error processing glob for '{}': {}",
                        path.display(),
                        err.msg
                    );
                    if continue_on_error {
                        log_error!("{}", err_msg);
                    } else {
                        bail!(&err_msg)
                    }
                }
            }
        }
        self.directories.push(PathBuf::from(path));
        Ok(())
    }
}
