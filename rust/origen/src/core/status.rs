extern crate time;
use crate::core::application::Application;
use crate::utility::file_utils::with_dir;
use crate::{built_info, Result};
use semver::Version;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;

/// Exposes some status information about the runtime environment, e.g. whether an
/// application workspace is present
//
// If you add an attribute to this you must also update:
// * pyapi/src/lib.rs to convert it to Python
// * default function below to define the default value (no nils in Rust)
pub struct Status {
    /// When true, Origen is executing within an application workspace
    pub is_app_present: bool,
    /// When Origen is executing with the context of an application, this represents it
    pub app: Option<Application>,
    /// The Origen version in a Semver object
    pub origen_version: Version,
    pub start_time: time::Tm,
    /// The full file system path to the user's home directory
    pub home: PathBuf,
    pub log_level: u8,
    unhandled_error_count: RwLock<usize>,
    /// This must remain private, forcing it to be accessed by a function. That ensures
    /// that it will always be created if it doesn't exist and all other code can forget about
    /// checking for that.
    output_dir: RwLock<Option<PathBuf>>,
    /// This must remain private, forcing it to be accessed by a function. That ensures
    /// that it will always be created if it doesn't exist and all other code can forget about
    /// checking for that.
    reference_dir: RwLock<Option<PathBuf>>,
}

impl Default for Status {
    fn default() -> Status {
        log_trace!("Building STATUS");
        let (p, r) = search_for_app_root();
        let version = match Version::parse(built_info::PKG_VERSION) {
            Ok(v) => v,
            Err(_e) => Version::parse("0.0.0").unwrap(),
        };
        let s = Status {
            is_app_present: p,
            app: match p {
                true => Some(Application::new(r)),
                false => None,
            },
            origen_version: version,
            start_time: time::now(),
            home: get_home_dir(),
            log_level: 1,
            unhandled_error_count: RwLock::new(0),
            output_dir: RwLock::new(None),
            reference_dir: RwLock::new(None),
        };
        log_trace!("Status built successfully");
        s
    }
}

impl Status {
    /// Returns the number of unhandled errors that have been encountered since this thread started.
    /// An example of a unhandled error is a pattern that failed to generate.
    /// If an error occurs on the Python side then Origen will most likely crash, however on the
    /// rust side it is possible to keep going (e.g. to the next pattern), and this keeps track
    /// of how many big problems there were for reporting to the user at the end.
    pub fn unhandled_error_count(&self) -> usize {
        *self.unhandled_error_count.read().unwrap()
    }

    pub fn inc_unhandled_error_count(&self) {
        let mut cnt = self.unhandled_error_count.write().unwrap();
        *cnt += 1;
    }

    /// Set the base output dir to the given path, it is <APP ROOT>/output by default
    pub fn set_output_dir(&self, path: &Path) {
        let mut dir = self.output_dir.write().unwrap();
        *dir = Some(path.to_path_buf());
    }

    /// Set the base reference dir to the given path, it is <APP ROOT>/.ref by default
    pub fn set_reference_dir(&self, path: &Path) {
        let mut dir = self.reference_dir.write().unwrap();
        *dir = Some(path.to_path_buf());
    }

    /// This is the main method to get the current output directory, accounting for all
    /// possible ways to set it, from current command, the app, default, etc.
    /// If nothing has been set (only possible when running globally), then it will default
    /// to the PWD.
    /// It will ensure that the directory exists before returning it.
    pub fn output_dir(&self) -> PathBuf {
        let mut dir = self.output_dir.read().unwrap().to_owned();
        // If it hasn't been set by the current command
        if dir.is_none() {
            if let Some(app) = &self.app {
                dir = Some(app._output_directory());
            } else {
                dir = Some(env::current_dir().expect(
                    "Can't read the current directory when trying to set the output directory",
                ));
            }
        }
        let dir = dir.unwrap();
        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect(&format!(
                "Couldn't create the output directory '{}'",
                dir.display()
            ));
        }
        dir
    }

    /// Execute the given function with a reference to the current output directory (<APP ROOT>/output by default).
    /// Optionally, the current working directory can be switched to the output dir before executing
    /// the function and then restored at the end by setting change_to to True.
    /// If this is called when Origen is executing outside of an application workspace then it will
    /// return None unless it has been previously set by calling set_output_dir().
    pub fn with_output_dir<T, F>(&self, change_dir: bool, mut f: F) -> Result<T>
    where
        F: FnMut(&Path) -> Result<T>,
    {
        let dir = self.output_dir();
        if change_dir {
            with_dir(&dir, || f(&dir))
        } else {
            f(&dir)
        }
    }

    /// This is the main method to get the current reference directory, accounting for all
    /// possible ways to set it, from current command, the app, default, etc.
    /// If nothing has been set (only possible when running globally), then it will return None.
    /// It will ensure that the directory exists before returning it.
    pub fn reference_dir(&self) -> Option<PathBuf> {
        let mut dir = self.reference_dir.read().unwrap().to_owned();
        // If it hasn't been set by the current command
        if dir.is_none() {
            if let Some(app) = &self.app {
                dir = Some(app._reference_directory());
            }
        }
        if let Some(dir) = dir {
            if !dir.exists() {
                std::fs::create_dir_all(&dir).expect(&format!(
                    "Couldn't create the reference directory '{}'",
                    dir.display()
                ));
            }
            Some(dir)
        } else {
            None
        }
    }

    /// Execute the given function with a reference to the current reference directory (<APP ROOT>/.ref by default).
    /// Optionally, the current working directory can be switched to the reference dir before executing
    /// the function and then restored at the end by setting change_to to True.
    /// If this is called when Origen is executing outside of an application workspace then it will
    /// return None unless it has been previously set by calling set_reference_dir().
    pub fn with_reference_dir<T, F>(&self, change_dir: bool, mut f: F) -> Result<T>
    where
        F: FnMut(Option<&PathBuf>) -> Result<T>,
    {
        let dir = self.reference_dir();
        if change_dir && dir.is_some() {
            let dir = dir.unwrap();
            with_dir(&dir, || f(Some(&dir)))
        } else {
            f(dir.as_ref())
        }
    }
}

fn search_for_app_root() -> (bool, PathBuf) {
    log_trace!("Searching for app");
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
        log_debug!("No app found");
        (false, PathBuf::new())
    } else {
        log_debug!("App found at '{}'", path.display());
        (true, path)
    }
}

fn get_home_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(env::var("USERPROFILE").expect("Please set environment variable USERPROFILE to point to your home directory, then try again"))
    } else {
        PathBuf::from(env::var("HOME").expect(
            "Please set environment variable HOME to point to your home directory, then try again",
        ))
    }
}
