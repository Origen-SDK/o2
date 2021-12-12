use super::command_helpers::log_stdout_and_stderr;
use crate::Result;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// Given a relative path and a path its relative to, resolve the absolute path
///
/// In Windows, 'canonicalize' prepends "\\?\".
/// However, upon some google searching, it apparently is
/// dangerous to just remove as, in certain circumstances,
/// it does resolve to a different pattern.
/// But, other libraries, like glob, cannot handle it,
/// even when the ? is escaped.
/// So, instead just mashing the relative piece atop the
/// relative_to directory and users of these paths
/// can decide how to resolve.
/// For example, glob has no problem correctly resolving
/// this, without canonicalize
///
/// TL;DR - this needs some work in the future
pub fn to_abs_path(path: &PathBuf, relative_to: &PathBuf) -> Result<PathBuf> {
    if path.is_relative() {
        let mut resolved = PathBuf::from(relative_to);
        resolved.push(path);
        Ok(resolved)
    } else {
        Ok(path.to_path_buf())
    }
}
