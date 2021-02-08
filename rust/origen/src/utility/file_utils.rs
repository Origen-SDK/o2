use super::command_helpers::log_stdout_and_stderr;
use crate::Result;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

cfg_if! {
    if #[cfg(unix)] {
        use std::fs::File;
        use std::os::unix::fs::PermissionsExt;
    }
}

const MAX_PERMISSIONS: u16 = 0x1FF;

/// Returns the given abs path as a relative path. By default it will be made relative to the
/// PWD, but an alternative path to make it relative to can be supplied.
/// Can return an error if
///  * There is a problem resolving the PWD
///  * If the given abs_path or relative_to is not absolute
pub fn to_relative_path(abs_path: &Path, relative_to: Option<&Path>) -> Result<PathBuf> {
    let base = match relative_to {
        None => env::current_dir()?,
        Some(p) => p.to_path_buf(),
    };
    if !abs_path.is_absolute() {
        return error!(
            "An absolute path must be given to to_relative_path, this is relative: '{}'",
            abs_path.display()
        );
    }
    if !base.is_absolute() {
        return error!(
            "An absolute path must be given to to_relative_path, this is relative: '{}'",
            base.display()
        );
    }

    // This code came from here: https://stackoverflow.com/a/39343127/220679

    use std::path::Component;

    let mut ita = abs_path.components();
    let mut itb = base.components();
    let mut comps: Vec<Component> = vec![];
    loop {
        match (ita.next(), itb.next()) {
            (None, None) => break,
            (Some(a), None) => {
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
            (None, _) => comps.push(Component::ParentDir),
            (Some(a), Some(b)) if comps.is_empty() && a == b => (),
            (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
            (Some(_), Some(b)) if b == Component::ParentDir => {
                return error!(
                    "Could not work out relative path from '{}' to '{}'",
                    base.display(),
                    abs_path.display()
                )
            }
            (Some(a), Some(_)) => {
                comps.push(Component::ParentDir);
                for _ in itb {
                    comps.push(Component::ParentDir);
                }
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
        }
    }
    Ok(comps.iter().map(|c| c.as_os_str()).collect())
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
/// use origen::utility::file_utils::with_dir;
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

/// Create a symlink, works on Linux or Windows.
/// The dst path will be a symbolic link pointing to the src path.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use origen::utility::file_utils::symlink;
///
/// // Create a symlink from my_file.rs to proj/files/my_file.rs
/// symlink(Path::new("my_file.rs"), Path::new("proj/files/my_file.rs"));
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

/// Move a file or directory
pub fn mv(source: &Path, dest: &Path) -> Result<()> {
    if cfg!(windows) {
        return error!(
            "origen::utility::file_utils::move function is not supported on Windows yet"
        );
    }
    if !source.exists() {
        return error!("The source file/dir {} does not exist", source.display());
    }
    log_debug!("Moving '{}' to '{}'", source.display(), dest.display());

    let mut process = Command::new("mv")
        .args(&vec![source.to_str().unwrap(), dest.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    log_stdout_and_stderr(&mut process, None, None);

    if process.wait()?.success() {
        Ok(())
    } else {
        error!(
            "Something went wrong when moving {}, see log for details",
            source.display()
        )
    }
}

/// Copy the given file or directory to the given directory, directories will be copied recursively.
pub fn copy(source: &Path, dest: &Path) -> Result<()> {
    log_debug!("Copying '{}' to '{}'", source.display(), dest.display());
    _copy(source, dest, false)
}

/// Like copy, however if the source is a directory then its contents will be copied to the target
/// directory, rather than copying the source folder itself
pub fn copy_contents(source: &Path, dest: &Path) -> Result<()> {
    log_debug!(
        "Copying contents of '{}' to '{}'",
        source.display(),
        dest.display()
    );
    _copy(source, dest, true)
}

pub fn _copy(source: &Path, dest: &Path, contents: bool) -> Result<()> {
    if !source.exists() {
        return error!("The source file/dir {} does not exist", source.display());
    }

    if cfg!(windows) {
        error!("origen::utility::file_utils copy functions are not supported on Windows yet")
    } else {
        let mut args = vec!["-r"];

        let s;
        if contents && source.is_dir() {
            s = format!("{}/.", source.to_str().unwrap());
            args.push(&s);
        } else {
            args.push(source.to_str().unwrap());
        };
        args.push(dest.to_str().unwrap());

        let mut process = Command::new("cp")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        log_stdout_and_stderr(&mut process, None, None);

        if process.wait()?.success() {
            Ok(())
        } else {
            error!(
                "Something went wrong when copying {}, see log for details",
                source.display()
            )
        }
    }
}

pub enum FilePermissions {
    Private,
    Group,
    GroupWritable,
    PublicWithGroupWritable,
    Public,
    WorldWritable,
    Custom(u16)
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
            Self::Custom(perms) => format!("custom({})", perms)
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
            Self::Custom(perms) => *perms
        }
    }

    pub fn from_str(perms: &str) -> Result<Self> {
        match perms {
            "private" | "Private" | "007" => Ok(Self::Private),
            "group" | "Group" | "057" => Ok(Self::Group),
            "group_writable" | "GroupWritable" | "0077" => Ok(Self::GroupWritable),
            "public_with_group_writable" | "PublicWithGroupWritable" | "0577" => Ok(Self::PublicWithGroupWritable),
            "public" | "Public" | "557" => Ok(Self::Public),
            "world_writable" | "WorldWritable" | "777" => Ok(Self::WorldWritable),
            _ => error!("Cannot infer permisions from {}", perms)
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
                    error!("Given permissions {:#o} exceeds maximum supported Unix permissions {:#o}", perms, MAX_PERMISSIONS)
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
                    "Changing file permissions to {} is not support on OS {}",
                    self.to_str(),
                    std::env::consts::OS
                );
                if warn_when_unsupported {
                    // Generate a warning instead of throwing an error
                    crate::LOGGER.warning(&message);
                    Ok(())
                } else {
                    error!("{}", message)
                }
            }
        }
    }
}