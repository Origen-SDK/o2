pub mod config;
pub mod target;

use super::application::config::Config;
use crate::Result;
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Represents the current application, an instance of this is returned by
/// origen::app().
pub struct Application {
    /// The full file system path to the application root (when applicable)
    pub root: PathBuf,
    config: RwLock<Config>,
}

#[derive(Debug, Deserialize)]
struct AppVersion {
    major: u32,
    minor: u32,
    patch: u32,
    pre: Option<u32>,
}

impl Application {
    pub fn new(root: PathBuf) -> Application {
        log_trace!("Building new Application");
        let config = Config::build(&root);
        Application {
            root: root,
            config: RwLock::new(config),
        }
    }

    pub fn version(&self) -> Result<Version> {
        let version_file = self.root.join("config").join("version.toml");
        log_trace!("Reading app version");
        if !version_file.exists() {
            return error!(
                "App version file does not exist at '{}'",
                version_file.display()
            );
        }
        let content = match fs::read_to_string(&version_file) {
            Ok(x) => x,
            Err(e) => return error!("There was a problem reading the app version file: {}", e),
        };
        let app_ver: AppVersion = match toml::from_str(&content) {
            Ok(x) => x,
            Err(e) => return error!("Malformed app version file: {}", e),
        };
        if let Some(pre) = app_ver.pre {
            Ok(Version::parse(&format!(
                "{}.{}.{}-pre{}",
                app_ver.major, app_ver.minor, app_ver.patch, pre
            ))?)
        } else {
            Ok(Version::parse(&format!(
                "{}.{}.{}",
                app_ver.major, app_ver.minor, app_ver.patch
            ))?)
        }
    }

    /// Execute the given function with a reference to the application config.
    pub fn with_config<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&Config) -> Result<T>,
    {
        let cfg = self.config.read().unwrap();
        func(&cfg)
    }

    /// Execute the given function with a reference to the application config.
    pub fn with_config_mut<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&mut Config) -> Result<T>,
    {
        let mut cfg = self.config.write().unwrap();
        func(&mut cfg)
    }

    /// Returns the application name
    pub fn name(&self) -> String {
        self.with_config(|cfg| Ok(cfg.name.to_string())).unwrap()
    }

    /// Returns a path to the current application's 'app' dir which is the root + app name.
    pub fn app_dir(&self) -> PathBuf {
        self.root.join(self.name())
    }

    /// Resolves a directory/file path relative to the application's root.
    /// Accepts an optional 'user_val' and a default. The resulting directory will be resolved from:
    /// 1. If a user value is given, and its absolute, this is the final path.
    /// 2. If a user value is given but its not absolute, then the final path is the user path relative to the application root.
    /// 3. If no user value is given, the final path is the default path relative to the root.
    /// Notes:
    ///   A default is required, but an empty default will point to the application root.
    ///   The default is assumed to be relative. Absolute defaults are not supported.
    pub fn resolve_path(&self, user_val: Option<&String>, default: &str) -> PathBuf {
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
        let mut dir = self.root.clone();
        dir.push(offset);
        dir
    }

    /// Don't use this unless you know what you're doing, use origen::STATUS::output_dir() instead, since
    /// that accounts for the output directory being overridden by the current command
    pub fn _output_directory(&self) -> PathBuf {
        self.with_config(|config| Ok(self.resolve_path(config.output_directory.as_ref(), "output")))
            .unwrap()
    }

    /// Don't use this unless you know what you're doing, use origen::STATUS::reference_dir() instead, since
    /// that accounts for the reference directory being overridden by the current command
    pub fn _reference_directory(&self) -> PathBuf {
        self.with_config(
            |config| Ok(self.resolve_path(config.reference_directory.as_ref(), ".ref")),
        )
        .unwrap()
    }

    pub fn website_output_directory(&self) -> PathBuf {
        self.with_config(|config| {
            Ok(self.resolve_path(config.website_output_directory.as_ref(), "output/web"))
        })
        .unwrap()
    }

    pub fn website_source_directory(&self) -> PathBuf {
        self.with_config(|config| {
            Ok(self.resolve_path(config.website_source_directory.as_ref(), "web/source"))
        })
        .unwrap()
    }
}
