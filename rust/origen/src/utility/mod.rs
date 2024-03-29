pub mod big_uint_helpers;
pub mod file_actions;
pub mod file_utils;
pub mod location;
pub mod command_helpers;
pub mod github;
pub mod num_helpers;
pub mod release_scribe;
pub mod sessions;

use crate::{Result, STATUS};
use std::path::{Path, PathBuf};

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
    let mut dir = STATUS.origen_wksp_root.clone();
    dir.push(offset);
    dir
}

pub fn status_to_bool(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "pass" | "success" | "true" => Ok(true),
        "fail" | "error" | "false" => Ok(false),
        _ => bail!("Could not convert '{}' to boolean value", s),
    }
}
