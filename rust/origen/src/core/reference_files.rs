//! This manages how generated files are saved as references.
//! When new or changed files are generated a save ref is saved on disk which will allow a future
//! execution of the 'origen save_ref' command to determine the source and destination of the file
//! to be saved as a the new reference version.
//! Save refs are stored as single files per reference instead of a single file containing all
//! references or the log as was used in O1. A single file is not used to avoid contentions when
//! multiple generation jobs are running in parallel on the LSF.

use crate::core::file_handler::File;
use crate::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
enum SaveRefType {
    New,
    Changed,
}

/// A 'save_ref' refers to a record that is created during pattern generation which records
/// the location of the new file (the source) and where it should be saved if the user wishes
/// to make it the new reference (the dest);
#[derive(Debug, Serialize, Deserialize)]
struct SaveRef {
    kind: SaveRefType,
    source: PathBuf,
    dest: PathBuf,
}

impl SaveRef {
    fn load(key: Option<&Path>, path_to_file: Option<&Path>) -> Result<SaveRef> {
        let file;
        if let Some(f) = path_to_file {
            file = f.to_owned();
        } else {
            file = save_ref_dir().join(&format!("{}.toml", key.unwrap().display()));
        }
        if !file.exists() {
            return error!("No save reference exists at '{}'", file.display());
        }
        let content = match fs::read_to_string(&file) {
            Ok(x) => x,
            Err(e) => return error!("There was a problem reading the save reference: {}", e),
        };
        let save_ref: SaveRef = match toml::from_str(&content) {
            Ok(x) => x,
            Err(e) => return error!("Malformed save reference file: {}", e),
        };
        Ok(save_ref)
    }

    fn save(&self, key: &Path) -> Result<()> {
        let file = save_ref_dir().join(&format!("{}.toml", key.display()));
        let serialized = toml::to_string(&self).unwrap();
        File::create(file).write(&serialized);
        Ok(())
    }

    fn apply(&self) -> Result<()> {
        std::fs::create_dir_all(&self.dest.parent().unwrap())?;
        std::fs::copy(&self.source, &self.dest)?;
        Ok(())
    }
}

pub fn apply_ref(key: &Path) -> Result<()> {
    match SaveRef::load(Some(key), None) {
        Err(e) => Err(e),
        Ok(save_ref) => Ok(save_ref.apply()?),
    }
}

pub fn apply_all_new_refs() -> Result<()> {
    log_debug!("Saving all new references");
    apply_all_refs(&save_ref_dir(), SaveRefType::New)
}

pub fn apply_all_changed_refs() -> Result<()> {
    log_debug!("Updating all changed references");
    apply_all_refs(&save_ref_dir(), SaveRefType::Changed)
}

fn apply_all_refs(dir: &Path, kind: SaveRefType) -> Result<()> {
    log_trace!("Looking for save refs in '{}'", dir.display());
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                apply_all_refs(&path, kind)?;
            } else {
                let save_ref = SaveRef::load(None, Some(&path))?;
                if save_ref.kind == kind {
                    save_ref.apply()?;
                }
            }
        }
    }
    Ok(())
}

pub fn create_changed_ref(key: &Path, source: &Path, dest: &Path) -> Result<()> {
    let s = SaveRef {
        kind: SaveRefType::Changed,
        source: source.to_owned(),
        dest: dest.to_owned(),
    };
    s.save(key)?;
    Ok(())
}

pub fn create_new_ref(key: &Path, source: &Path, dest: &Path) -> Result<()> {
    let s = SaveRef {
        kind: SaveRefType::New,
        source: source.to_owned(),
        dest: dest.to_owned(),
    };
    s.save(key)?;
    Ok(())
}

pub fn clear_save_refs() -> Result<()> {
    // Remove all existing save references
    let save_dir = save_ref_dir();
    if save_dir.exists() {
        std::fs::remove_dir_all(&save_dir)?;
    }
    std::fs::create_dir_all(&save_dir)?;
    Ok(())
}

/// Returns a path to the directory where save references will be stored, callers are responsible
/// for ensuring this is only called when an app is present
fn save_ref_dir() -> PathBuf {
    crate::app().unwrap().root.join(".origen").join("save_refs")
}
