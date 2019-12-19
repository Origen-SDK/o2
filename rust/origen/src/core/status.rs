extern crate time;
use crate::built_info;
use semver::Version;
use std::env;
/// Exposes the some status information about the runtime environment, e.g. whether an
/// application workspace is present
use std::path::PathBuf;

// If you add an attribute to this you must also update:
// * pyapi/src/lib.rs to convert it to Python
// * default function below to define the default value (no nils in Rust)
pub struct Status {
    /// When true, Origen is executing within an application workspace
    pub is_app_present: bool,
    /// The full file system path to the application root (when applicable)
    pub root: PathBuf,
    /// The Origen version in a Semver object
    pub origen_version: Version,
    pub start_time: time::Tm,
    /// The full file system path to the user's home directory
    pub home: PathBuf,
}

impl Default for Status {
    fn default() -> Status {
        let (p, r) = search_for_app_root();
        let version = match Version::parse(built_info::PKG_VERSION) {
            Ok(v) => v,
            Err(_e) => Version::parse("0.0.0").unwrap(),
        };
        Status {
            is_app_present: p,
            root: r,
            origen_version: version,
            start_time: time::now(),
            home: get_home_dir(),
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

    while !path.join("config").join("origen.toml").is_file() && !aborted {
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

fn get_home_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(env::var("USERPROFILE").expect("Please set environment variable USERPROFILE to point to your home directory, then try again"))
    }
    else {
        PathBuf::from(env::var("HOME").expect("Please set environment variable HOME to point to your home directory, then try again"))
    }
}