pub mod config;
pub mod target;

use std::path::PathBuf;
use crate::app_config;
use crate::core::utility::{resolve_dir_from_app_root};

pub fn output_directory() -> PathBuf {
  resolve_dir_from_app_root(app_config().output_directory.as_ref(), "output")
}

pub fn website_output_directory() -> PathBuf {
  resolve_dir_from_app_root(app_config().website_output_directory.as_ref(), "output/web")
}

pub fn website_source_directory() -> PathBuf {
  resolve_dir_from_app_root(app_config().website_source_directory.as_ref(), "web/source")
}
