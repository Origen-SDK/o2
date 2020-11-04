extern crate time;
use crate::core::application::Application;
use crate::testers::SupportedTester;
use crate::utility::file_utils::with_dir;
use crate::{built_info, Result};
use regex::Regex;
use semver::Version;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;

// Trait for extending std::path::PathBuf
use path_slash::PathBufExt;

/// Exposes some status information about the runtime environment, e.g. whether an
/// application workspace is present
//
// If you add an attribute to this you must also update:
// * pyapi/src/lib.rs to convert it to Python
// * default function below to define the default value (no nils in Rust)
#[derive(Debug)]
pub struct Status {
    /// When true, Origen is executing within an origen development workspace
    pub is_origen_present: bool,
    /// When true, Origen is executing within an application workspace
    pub is_app_present: bool,
    /// When Origen is executing with the context of an application, this represents it
    pub app: Option<Application>,
    /// When Origen is running within an Origen development workspace or when it is running within
    /// an app which is referencing a local copy of Origen, this will point to the root of the
    /// Origen workspace.
    /// i.e. it is only valid when either is_origen_present or is_app_in_origen_dev_mode is true.
    pub origen_wksp_root: PathBuf,
    /// The Origen version in a Semver object
    pub origen_version: Version,
    pub start_time: time::Tm,
    /// The full file system path to the user's home directory
    pub home: PathBuf,
    pub log_level: u8,
    /// When true it means that Origen is running within an app and that app is using a local
    /// development version of Origen
    pub is_app_in_origen_dev_mode: bool,
    unhandled_error_count: RwLock<usize>,
    /// This must remain private, forcing it to be accessed by a function. That ensures
    /// that it will always be created if it doesn't exist and all other code can forget about
    /// checking for that.
    output_dir: RwLock<Option<PathBuf>>,
    /// This must remain private, forcing it to be accessed by a function. That ensures
    /// that it will always be created if it doesn't exist and all other code can forget about
    /// checking for that.
    reference_dir: RwLock<Option<PathBuf>>,
    cli_location: RwLock<Option<PathBuf>>,
    _custom_tester_ids: RwLock<Vec<String>>,
    current_testers: RwLock<Vec<Vec<SupportedTester>>>,
    unique_id: RwLock<usize>,
}

impl Default for Status {
    fn default() -> Status {
        log_trace!("Building STATUS");
        log_trace!(
            "Current exe is {}",
            std::env::current_exe().unwrap().display()
        );
        let mut dev_mode_origen_root: Option<PathBuf> = None;
        let mut origen_dev_mode = false;
        let (app_present, app_root) = search_for_from_pwd(vec!["config", "origen.toml"], true);
        let (origen_present, origen_wksp_root) =
            search_for_from_pwd(vec![".origen_dev_workspace"], false);

        // If a standalone app is present check if it contains a reference to a local Origen
        if app_present && !origen_present {
            let pyproject = app_root.join("pyproject.toml");
            match std::fs::read_to_string(&pyproject) {
                Ok(contents) => {
                    // (?m) is how you set the multiline flag
                    let regex = Regex::new(
                        r#"(?m)^\s*origen\s*=\s*\{\s*path\s*=\s*['"](?P<path>[^'"]+)['"]"#,
                    )
                    .unwrap();
                    if let Some(captures) = regex.captures(&contents) {
                        let path = PathBuf::from(captures.name("path").unwrap().as_str());
                        let path = match path.is_relative() {
                            true => app_root.join(&path),
                            false => path,
                        };
                        let (found, path) = search_for(vec![".origen_dev_workspace"], false, &path);
                        if found {
                            dev_mode_origen_root = Some(path);
                            origen_dev_mode = true;
                        }
                    }
                }
                Err(e) => log_error!("{}", e),
            }
        }

        let version = match Version::parse(built_info::PKG_VERSION) {
            Ok(v) => v,
            Err(_e) => Version::parse("0.0.0").unwrap(),
        };
        let s = Status {
            is_app_present: app_present,
            app: match app_present {
                true => Some(Application::new(app_root.clone())),
                false => None,
            },
            is_origen_present: origen_present,
            origen_wksp_root: match dev_mode_origen_root {
                Some(x) => x,
                None => origen_wksp_root,
            },
            origen_version: version,
            start_time: time::now(),
            home: get_home_dir(),
            log_level: 1,
            unhandled_error_count: RwLock::new(0),
            output_dir: RwLock::new(None),
            reference_dir: RwLock::new(None),
            cli_location: RwLock::new(None),
            is_app_in_origen_dev_mode: origen_dev_mode,
            _custom_tester_ids: RwLock::new(vec![]),
            current_testers: RwLock::new(vec![]),
            unique_id: RwLock::new(0),
        };
        log_trace!("Status built successfully");
        s
    }
}

impl Status {
    pub fn register_custom_tester(&self, name: &str) {
        let mut t = self._custom_tester_ids.write().unwrap();
        let name = name.to_string();
        if !t.contains(&name) {
            t.push(name);
        }
    }

    pub fn custom_tester_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = vec![];
        for id in &*self._custom_tester_ids.read().unwrap() {
            ids.push(id.to_string());
        }
        ids
    }

    /// Returns an ID that it guaranteed unique across threads and within the lifetime of an Origen
    /// invocation
    pub fn generate_unique_id(&self) -> usize {
        let mut unique_id = self.unique_id.write().unwrap();
        *unique_id += 1;
        *unique_id
    }

    /// Corresponds to the start of a tester specific block of code, returns an error if the given tester
    /// selection is not a subset of any existing selection
    pub fn push_current_testers(&self, testers: Vec<SupportedTester>) -> Result<()> {
        if testers.is_empty() {
            return error!("No tester type(s) given");
        }
        let mut current_testers = self.current_testers.write().unwrap();
        // When some testers are already selected, the application is only allowed to select a subset of them,
        // so verify that all given testers are already contained in the last selection
        if !current_testers.is_empty() {
            let last = current_testers.last().unwrap();
            for t in &testers {
                if !last.contains(t) {
                    return error!(
                        "Can't select tester '{}' within a block that already selects '{:?}'",
                        t, last
                    );
                }
            }
        }
        current_testers.push(testers.clone());
        Ok(())
    }

    /// Corresponds to the end of a tester specific block, returns an error if no tester specific
    /// selection currently exists
    pub fn pop_current_testers(&self) -> Result<()> {
        let mut current_testers = self.current_testers.write().unwrap();
        if current_testers.is_empty() {
            return error!("There has been an attempt to close a tester-specific block, but none is currently open in the program generator");
        }
        let _ = current_testers.pop();
        Ok(())
    }

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

    pub fn set_cli_location(&self, loc: Option<String>) {
        if let Some(loc) = loc {
            let mut cli_loc = self.cli_location.write().unwrap();
            *cli_loc = Some(PathBuf::from(loc));
        }
    }

    pub fn cli_location(&self) -> Option<PathBuf> {
        self.cli_location.read().unwrap().to_owned()
    }

    /// Set the base output dir to the given path, it is <APP ROOT>/output by default
    pub fn set_output_dir(&self, path: &Path) {
        let mut dir = self.output_dir.write().unwrap();
        *dir = Some(clean_path(path));
    }

    /// Set the base reference dir to the given path, it is <APP ROOT>/.ref by default
    pub fn set_reference_dir(&self, path: &Path) {
        let mut dir = self.reference_dir.write().unwrap();
        *dir = Some(clean_path(path));
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

pub fn search_for_from_pwd(paths: Vec<&str>, searching_for_app: bool) -> (bool, PathBuf) {
    let base = env::current_dir();
    let base = match base {
        Ok(p) => p,
        Err(_e) => {
            return (false, PathBuf::new());
        }
    };
    search_for(paths, searching_for_app, &base)
}

pub fn search_for(paths: Vec<&str>, searching_for_app: bool, base: &Path) -> (bool, PathBuf) {
    if searching_for_app {
        log_trace!("Searching for app");
    }
    let mut aborted = false;
    let mut base = base.to_path_buf();

    while !paths
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
        if searching_for_app {
            log_debug!("No app found");
        }
        (false, PathBuf::new())
    } else {
        if searching_for_app {
            log_debug!("App found at '{}'", base.display());
        }
        (true, base)
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

// Convert any paths with / to \ on Windows
fn clean_path(path: &Path) -> PathBuf {
    let clean_path;
    if cfg!(target_os = "windows") {
        if let Some(p) = path.to_str() {
            let win_path = PathBuf::from_slash(p);
            clean_path = win_path;
        } else {
            clean_path = path.to_path_buf();
        }
    } else {
        clean_path = path.to_path_buf();
    }
    clean_path
}
