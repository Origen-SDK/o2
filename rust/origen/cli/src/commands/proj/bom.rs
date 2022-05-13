use super::group::{Group, TempGroup};
use super::package::Package;
use super::{error_and_exit, BOM_FILE};
use indexmap::IndexMap;
use origen::Result;
use origen_metal::utils::file::symlink;
use origen_metal::utils::file::with_dir;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[derive(Debug, Deserialize)]
// This is a temporary structure to make the BOM file syntax nicer for users.
// It will be quickly converted to a BOM which is structured better for us.
pub struct TempBOM {
    meta: Option<Meta>,
    package: Option<Vec<Package>>,
    links: Option<IndexMap<String, String>>,
    group: Option<Vec<TempGroup>>,
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
            groups: IndexMap::new(),
        };
        if let Some(packages) = &self.package {
            for p in packages.iter() {
                if bom.packages.contains_key(&p.id) {
                    return error!("Duplicate package definition found: '{}'", p.id);
                }
                bom.packages.insert(p.id.clone(), p.clone());
            }
        }
        // Add a default group called "all" to all BOMs
        let all = Group {
            id: "all".to_string(),
            packages: bom.packages.iter().map(|(id, _p)| id.clone()).collect(),
            version: None,
        };
        bom.groups.insert("all".to_string(), all);
        if let Some(groups) = &self.group {
            for g in groups.iter() {
                if bom.groups.contains_key(&g.id) {
                    return error!("Duplicate group definition found: '{}'", g.id);
                }
                bom.groups.insert(g.id.clone(), g.to_group());
            }
        }
        if let Some(links) = &self.links {
            for (k, v) in links.iter() {
                if bom.links.contains_key(k) {
                    return error!("Duplicate link definition found: '{}'", k);
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
    pub groups: IndexMap<String, Group>,
}

impl fmt::Display for BOM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = "".to_string();
        if self.links.len() > 0 {
            s += "[links]\n";
            for (k, v) in self.links.iter() {
                s += &format!("\"{}\" = \"{}\"\n", k, v);
            }
            s += "\n";
        }
        for (id, g) in self.groups.iter() {
            if id != "all" {
                s += &g.to_string(0);
            }
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
            groups: IndexMap::new(),
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
                    error_and_exit(
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
                    error_and_exit(
                        &format!("Malformed BOM file '{}':\n{}", f.display(), e),
                        Some(1),
                    );
                    unreachable!()
                }
            };
            match new_bom.to_bom() {
                Ok(mut x) => {
                    x.copy_group_packages(&bom);
                    x.apply_group_versions();
                    bom.merge(x, &f);
                }
                Err(e) => {
                    error_and_exit(
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

    /// Returns the package IDs matching the given package or group IDs.
    /// An error will be returned if one of the given IDs does not
    /// match a known package or group.
    pub fn resolve_ids(&self, ids: Vec<&str>) -> Result<Vec<String>> {
        let mut packages: Vec<String> = vec![];

        for id in ids {
            let id = id.to_string();
            if self.packages.contains_key(&id) {
                if !packages.contains(&id) {
                    packages.push(id);
                }
            } else if self.groups.contains_key(&id) {
                for pid in &self.groups[&id].packages {
                    if !packages.contains(pid) {
                        packages.push(pid.to_string());
                    }
                }
            } else {
                return error!("No package or group was found matching ID '{}'", id);
            }
        }
        Ok(packages)
    }

    //pub fn packages_from_path(&self, path: &Path) -> Result<Vec<&Package>> {
    //    if let Some(id) = pkg_ref.to_str() {
    //        if let Some(pkg) = self.packages.get(id) {
    //            return Ok(vec![pkg]);
    //        }
    //    }
    //    let pkg_ref = pkg_ref.canonicalize()?;
    //    Ok(self
    //        .packages
    //        .iter()
    //        .filter_map(|(_id, pkg)| {
    //            let pkg_root = pkg.path(self.root());
    //            if pkg_ref.strip_prefix(&pkg_root).is_ok()
    //                || pkg_root.strip_prefix(&pkg_ref).is_ok()
    //            {
    //                Some(pkg)
    //            } else {
    //                None
    //            }
    //        })
    //        .collect())
    //}

    fn merge(&mut self, bom: BOM, source: &Path) {
        self.files.push(source.to_path_buf());
        for (id, p) in bom.packages.iter() {
            if self.packages.contains_key(id) {
                self.packages.get_mut(id).unwrap().merge(p);
            } else {
                self.packages.insert(id.clone(), p.clone());
            }
        }
        for (id, g) in bom.groups.iter() {
            if self.groups.contains_key(id) {
                self.groups.get_mut(id).unwrap().merge(g);
            } else {
                self.groups.insert(id.clone(), g.clone());
            }
        }
        for (k, v) in bom.links.iter() {
            let _ = self.links.insert(k.clone(), v.clone());
        }
        self.meta.merge(&bom.meta);
    }

    /// Applies the version from a group to any of its child packages that doesn't have one
    fn apply_group_versions(&mut self) {
        for (_id, group) in &self.groups {
            if let Some(group_version) = &group.version {
                for pid in &group.packages {
                    if let Some(package) = self.packages.get_mut(pid) {
                        if package.version.is_none() {
                            package.version = Some(group_version.clone());
                        }
                    }
                }
            }
        }
    }

    /// Child BOMs are not allowed to redefine what packages are in a group, this is used
    /// to copy the existing definitions from a parent BOM.
    /// It is done separately from merging to support the version resolution flow.
    fn copy_group_packages(&mut self, bom: &BOM) {
        for (gid, parent_group) in &bom.groups {
            if let Some(group) = self.groups.get_mut(gid) {
                group.packages = parent_group.packages.clone();
            }
        }
    }

    fn validate(&self) {
        for (_id, p) in self.packages.iter() {
            p.validate();
        }
        for (_id, g) in self.groups.iter() {
            g.validate();
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

    pub fn create_links(&self, force: bool) -> Result<bool> {
        let mut force_required = false;
        Ok(with_dir(self.root(), || {
            for (dest, source) in self.links.iter() {
                let dest = Path::new(dest);
                let source = Path::new(source);
                if dest.exists() {
                    match dest.read_link() {
                        // Means it is not a symlink
                        Err(_) => {
                            if force {
                                if dest.is_file() {
                                    std::fs::remove_file(dest)?;
                                } else {
                                    std::fs::remove_dir_all(dest)?;
                                }
                            } else {
                                display_redln!("ERROR");
                                log_error!(
                                    "Could not create link '{}' as something already exists at that location, run again with --force to replace it",
                                    dest.display()
                                );
                                force_required = true;
                            }
                        }
                        Ok(_target) => {
                            // Just delete any existing symlink and re-create it, don't need to worry about checking if it current
                            // points to a different location - links are cheap and no risk of losing data
                            std::fs::remove_file(dest)?;
                        }
                    }
                }
                if !dest.exists() {
                    if source.exists() {
                        symlink(source, dest)?;
                    } else {
                        origen_metal::bail!(
                            "The target of link '{}' does not exist - '{}'",
                            dest.display(),
                            source.display()
                        );
                    }
                }
            }
            Ok(force_required)
        })?)
    }
}
