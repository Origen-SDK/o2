use crate::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

cfg_if! {
    if #[cfg(unix)] {
        use std::fs::File;
        use std::os::unix::fs::PermissionsExt;
    }
}

const MAX_PERMISSIONS: u16 = 0x1FF;

/// See search_backwards_for()
pub fn search_backwards_for_from_pwd(files: Vec<&str>) -> (bool, PathBuf) {
    let base = env::current_dir();
    let base = match base {
        Ok(p) => p,
        Err(_e) => {
            return (false, PathBuf::new());
        }
    };
    search_backwards_for(files, &base)
}

/// Searches backwards (i.e. up towards the file system root) from the given base path for any of the
/// given file(s).
///
/// The directory containing the first one to be found will be returned like (true, dir).
///
/// If they haven't been found by the time the root of the file system is reached then (false, dir) will
/// be returned, where dir is an empty PathBuf.
pub fn search_backwards_for(files: Vec<&str>, base: &Path) -> (bool, PathBuf) {
    let mut aborted = false;
    let mut base = base.to_path_buf();

    log_debug!(
        "Searching backwards from '{}' for '{:?}'",
        base.display(),
        &files
    );

    while !files
        .iter()
        .fold(base.clone(), |acc, p| acc.join(p))
        .is_file()
        && !aborted
    {
        if !base.pop() {
            aborted = true;
        }
    }

    if aborted {
        log_debug!("Not found");
        (false, PathBuf::new())
    } else {
        log_debug!("Found at '{}'", base.display());
        (true, base)
    }
}

/// Change the current directory to the given one
pub fn cd(dir: &Path) -> Result<()> {
    env::set_current_dir(&dir).context(&format!("When cd'ing to '{}'", dir.display()))?;
    Ok(())
}

/// Create a symlink, works on Linux or Windows.
/// The dst path will be a symbolic link pointing to the src path.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use origen_metal::utils::file::symlink;
///
/// // Create a symlink at my_file.rs pointing to proj/files/my_file.rs
/// symlink(Path::new("proj/files/my_file.rs"), Path::new("my_file.rs"));
/// ```
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    #[cfg(windows)]
    {
        if src.as_ref().is_dir() {
            Ok(std::os::windows::fs::symlink_dir(src, dst)?)
        } else {
            Ok(std::os::windows::fs::symlink_file(src, dst)?)
        }
    }
    #[cfg(unix)]
    {
        Ok(std::os::unix::fs::symlink(src, dst)?)
    }
}

/// Temporarily sets the current dir to the given dir for the duration of the given
/// function and then restores it at the end.
/// An error will be returned if there is a problem switching to the given directory,
/// e.g. if it doesn't exist, otherwise the result from the given function is returned.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use origen_metal::utils::file::with_dir;
///
/// let result = with_dir(Path::new("path/to/some/dir"), || {
///   // Do something in that dir
///   Ok(())
/// });
/// ```
pub fn with_dir<T, F>(path: &Path, mut f: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    log_trace!("Changing directory to '{}'", path.display());
    let orig = env::current_dir()?;
    env::set_current_dir(path)?;
    let result = f();
    log_trace!("Restoring directory to '{}'", orig.display());
    env::set_current_dir(&orig)?;
    result
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilePermissions {
    Private,
    Group,
    GroupWritable,
    PublicWithGroupWritable,
    Public,
    WorldWritable,
    Custom(u16),
}

impl FilePermissions {
    pub fn to_str(&self) -> String {
        match self {
            Self::Private => "private".to_string(),
            Self::Group => "group".to_string(),
            Self::GroupWritable => "group_writable".to_string(),
            Self::PublicWithGroupWritable => "public_with_group_writable".to_string(),
            Self::Public => "public".to_string(),
            Self::WorldWritable => "world_writable".to_string(),
            Self::Custom(perms) => format!("custom({})", perms),
        }
    }

    pub fn to_i(&self) -> u16 {
        match self {
            Self::Private => 0700,
            Self::Group => 0750,
            Self::GroupWritable => 0770,
            Self::PublicWithGroupWritable => 0775,
            Self::Public => 0755,
            Self::WorldWritable => 0777,
            Self::Custom(perms) => *perms,
        }
    }

    pub fn from_str(perms: &str) -> Result<Self> {
        match perms {
            "private" | "Private" | "007" => Ok(Self::Private),
            "group" | "Group" | "057" => Ok(Self::Group),
            "group_writable" | "GroupWritable" | "0077" => Ok(Self::GroupWritable),
            "public_with_group_writable" | "PublicWithGroupWritable" | "0577" => {
                Ok(Self::PublicWithGroupWritable)
            }
            "public" | "Public" | "557" => Ok(Self::Public),
            "world_writable" | "WorldWritable" | "777" => Ok(Self::WorldWritable),
            _ => Err(error!("Cannot infer permissions from {}", perms)),
        }
    }

    pub fn from_i(perms: u16) -> Result<Self> {
        match perms {
            0700 => Ok(Self::Private),
            0750 => Ok(Self::Group),
            0770 => Ok(Self::GroupWritable),
            0775 => Ok(Self::PublicWithGroupWritable),
            0755 => Ok(Self::Public),
            0777 => Ok(Self::WorldWritable),
            _ => {
                if perms > MAX_PERMISSIONS {
                    // given value exceeds max Unix permissions. Very likely this is a mistake
                    bail!(
                        "Given permissions {:#o} exceeds maximum supported Unix permissions {:#o}",
                        perms, MAX_PERMISSIONS
                    )
                } else {
                    Ok(Self::Custom(perms))
                }
            }
        }
    }

    #[allow(unused_variables)]
    pub fn apply_to(&self, path: &Path, warn_when_unsupported: bool) -> Result<()> {
        cfg_if! {
            if #[cfg(unix)] {
                let f = File::open(path)?;
                let m = f.metadata()?;
                let mut permissions = m.permissions();
                permissions.set_mode(self.to_i().into());
                Ok(())
            } else {
                let message = format!(
                    "Changing file permissions to {} is not supported on OS {}",
                    self.to_str(),
                    std::env::consts::OS
                );
                if warn_when_unsupported {
                    // Generate a warning instead of throwing an error
                    crate::LOGGER.warning(&message);
                    Ok(())
                } else {
                    bail!("{}", message)
                }
            }
        }
    }
}
