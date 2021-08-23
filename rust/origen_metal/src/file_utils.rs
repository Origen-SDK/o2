use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

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
    env::set_current_dir(&dir).context(format!("When cd'ing to '{}'", dir.display()))?;
    Ok(())
}

/// Create a symlink, works on Linux or Windows.
/// The dst path will be a symbolic link pointing to the src path.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use origen_metal::file_utils::symlink;
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