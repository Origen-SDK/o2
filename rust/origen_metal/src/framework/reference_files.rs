//! This manages how generated files are saved as references.
//! When new or changed files are generated a save ref is saved on disk which will allow a future
//! execution of the 'origen save_ref' command to determine the source and destination of the file
//! to be saved as a the new reference version.
//! Save refs are stored as single files per reference instead of a single file containing all
//! references or the log as was used in O1. A single file is not used to avoid contentions when
//! multiple generation jobs are running in parallel on the LSF.

use crate::Result;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

lazy_static! {
    static ref SAVE_REF_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
}

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

/// This must be called before all other APIs to set the directory where temporary files can
/// be stored to keep track of which files have changed, are new etc.
/// The given location should not be added to revision control.
pub fn set_save_ref_dir(dir: PathBuf) {
    let mut save_ref_dir = SAVE_REF_DIR.write().unwrap();
    *save_ref_dir = Some(dir);
}

fn save_ref_dir() -> Result<PathBuf> {
    let dir = SAVE_REF_DIR.read().unwrap();
    if let Some(d) = &*dir {
        Ok(d.to_owned())
    } else {
        bail!("origen_metal::framework::reference_files::set_save_ref_dir must be called first before using the reference file APIs")
    }
}

impl SaveRef {
    fn load_from_file(file: &Path) -> Result<SaveRef> {
        if !file.exists() {
            bail!("No save reference exists at '{}'", file.display());
        }
        let content = match fs::read_to_string(&file) {
            Ok(x) => x,
            Err(e) => bail!("There was a problem reading the save reference: {}", e),
        };
        let save_ref: SaveRef = match toml::from_str(&content) {
            Ok(x) => x,
            Err(e) => bail!("Malformed save reference file: {}", e),
        };
        Ok(save_ref)
    }

    fn load_from_key(key: &Path) -> Result<SaveRef> {
        let file = save_ref_dir()?.join(&format!("{}.toml", key.display()));
        SaveRef::load_from_file(&file)
    }

    fn save(&self, key: &Path) -> Result<()> {
        let file = save_ref_dir()?.join(&format!("{}.toml", key.display()));
        let serialized = toml::to_string(&self).unwrap();
        if !file.parent().unwrap().exists() {
            create_dir_all(file.parent().unwrap())?;
        }
        std::fs::write(file, &serialized)?;
        Ok(())
    }

    fn apply(&self) -> Result<()> {
        std::fs::create_dir_all(&self.dest.parent().unwrap())?;
        std::fs::copy(&self.source, &self.dest)?;
        Ok(())
    }
}

/// Apply a previously created changed/new file reference (copys the new version of the file to
/// its reference file counterpart)
pub fn apply_ref(key: &Path) -> Result<()> {
    match SaveRef::load_from_key(key) {
        Err(e) => Err(e),
        Ok(save_ref) => Ok(save_ref.apply()?),
    }
}

/// Apply all previously created new file references (copys the new versions of the files to
/// their reference file counterparts)
pub fn apply_all_new_refs() -> Result<()> {
    log_debug!("Saving all new references");
    apply_all_refs(&save_ref_dir()?, SaveRefType::New)
}

/// Apply all previously created changed file references (copys the new versions of the files to
/// their reference file counterparts)
pub fn apply_all_changed_refs() -> Result<()> {
    log_debug!("Updating all changed references");
    apply_all_refs(&save_ref_dir()?, SaveRefType::Changed)
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
                let save_ref = SaveRef::load_from_file(&path)?;
                if save_ref.kind == kind {
                    save_ref.apply()?;
                }
            }
        }
    }
    Ok(())
}

/// Create a new record of a changed file, providing a key with which to reference it by in future,
/// a path to the new file (the source) and a path to the reference file (the dest).
///
/// The reference file can be updated to the new file in future by either calling apply_all_changed_refs()
/// or by calling apply_ref(key), where key should match the value given here.
pub fn create_changed_ref(key: &Path, source: &Path, dest: &Path) -> Result<()> {
    let s = SaveRef {
        kind: SaveRefType::Changed,
        source: source.to_owned(),
        dest: dest.to_owned(),
    };
    s.save(key)?;
    Ok(())
}

/// Create a new record of a new file, providing a key with which to reference it by in future,
/// a path to the new file (the source) and a path to the reference file (the dest).
///
/// The reference file can be updated to the new file in future by either calling apply_all_new_refs()
/// or by calling apply_ref(key), where key should match the value given here.
pub fn create_new_ref(key: &Path, source: &Path, dest: &Path) -> Result<()> {
    let s = SaveRef {
        kind: SaveRefType::New,
        source: source.to_owned(),
        dest: dest.to_owned(),
    };
    s.save(key)?;
    Ok(())
}

/// Clear (delete) all existing changed and new file references that have not been applied yet.
pub fn clear_save_refs() -> Result<()> {
    let save_dir = save_ref_dir()?;
    // Remove all existing save references
    if save_dir.exists() {
        std::fs::remove_dir_all(&save_dir)?;
    }
    std::fs::create_dir_all(&save_dir)?;
    Ok(())
}
