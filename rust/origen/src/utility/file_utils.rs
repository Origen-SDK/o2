use super::command_helpers::log_stdout_and_stderr;
use crate::{Result, STATUS};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Resolves a directory path from the current application root.
/// Accepts an optional 'user_val' and a default. The resulting directory will be resolved from:
/// 1. If a user value is given, and its absolute, this is the final path.
/// 2. If a user value is given but its not absolute, then the final path is the user path relative to the application root.
/// 3. If no user value is given, the final path is the default path relative to the root.
/// Notes:
///   A default is required, but an empty default will point to the application root.
///   The default is assumed to be relative. Absolute defaults are not supported.
pub fn resolve_dir_from_app_root(user_val: Option<&String>, default: &str) -> PathBuf {
    let offset;
    if let Some(user_str) = user_val {
        if Path::new(&user_str).is_absolute() {
            return PathBuf::from(user_str);
        } else {
            offset = user_str.to_string();
        }
    } else {
        offset = default.to_string();
    }
    let mut dir = STATUS.root.clone();
    dir.push(offset);
    dir
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
    log_debug!("Changing directory to '{}'", path.display());
    let orig = env::current_dir()?;
    env::set_current_dir(path)?;
    let result = f();
    log_debug!("Restoring directory to '{}'", orig.display());
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
        if source.is_dir() {
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
