//! Some helper methods for working with the current application

pub mod config;
pub mod target;

use crate::app_config;
use crate::utility::file_utils::resolve_dir_from_app_root;
use std::path::{Path, PathBuf};

pub fn output_directory() -> PathBuf {
    resolve_dir_from_app_root(app_config().output_directory.as_ref(), "output")
}

pub fn website_output_directory() -> PathBuf {
    resolve_dir_from_app_root(app_config().website_output_directory.as_ref(), "output/web")
}

pub fn website_source_directory() -> PathBuf {
    resolve_dir_from_app_root(app_config().website_source_directory.as_ref(), "web/source")
}

/// Returns the current application root dir or None if called when Origen is running
/// with no app
pub fn root() -> Option<&'static Path> {
    if crate::STATUS.is_app_present {
        Some(&crate::STATUS.root)
    } else {
        None
    }
}

/// Returns the current application 'app' dir which is the root + app name.
/// Returns None if called when Origen is running with no app.
pub fn app_dir() -> Option<PathBuf> {
    if crate::STATUS.is_app_present {
        let name = name().unwrap();
        Some(crate::STATUS.root.join(name))
    } else {
        None
    }
}

/// Returns the application name, or None if there is no application
pub fn name() -> Option<String> {
    if crate::STATUS.is_app_present {
        Some(crate::with_app_config(|cfg| Ok(cfg.name.to_string())).unwrap())
    } else {
        None
    }
}
