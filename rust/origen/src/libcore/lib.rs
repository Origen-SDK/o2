#[macro_use]
extern crate lazy_static;

pub mod python;
pub mod term;
pub mod config;

use semver::Version;
use std::env;
use std::path::PathBuf;

lazy_static! {
    pub static ref CONFIG: Config = Config::default();
}

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// Global configuration singleton available as _origen::CONFIG
pub struct Config {
    pub is_app_present: bool,
    pub root: PathBuf,
    pub origen_version: Version,
}

impl Default for Config {
    fn default() -> Config {
        let (p, r) = search_for_app_root();
        let version = match Version::parse(built_info::PKG_VERSION) {
            Ok(v) => v,
            Err(_e) => Version::parse("0.0.0").unwrap(),
        };
        Config {
            is_app_present: p,
            root: r,
            origen_version: version,
        }
    }
}

fn search_for_app_root() -> (bool, PathBuf) {
    let mut aborted = false;
    let path = env::current_dir();
    let mut path = match path {
        Ok(p) => p,
        Err(_e) => {
            return (false, PathBuf::new());
        }
    };

    while !path.join("config").join("application.toml").is_file() && !aborted {
        if !path.pop() {
            aborted = true;
        }
    }

    if aborted {
        (false, PathBuf::new())
    } else {
        (true, path)
    }
}
