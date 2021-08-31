extern crate time;
use crate::built_info;
use crate::core::application::Application;
use crate::testers::SupportedTester;
use crate::utility::file_utils::with_dir;
use crate::utility::version::Version;
use crate::Result as OrigenResult;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::RwLock;

// Trait for extending std::path::PathBuf
use path_slash::PathBufExt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    None,
    Convert,
    Generate,
    GeneratePattern,
    GenerateFlow,
    Compile,
    Interactive,
    Web,
    App,
    AppCommand,
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept any case
        let s = s.to_lowercase();
        match s.trim() {
            "none" => Ok(Operation::None),
            "convert" => Ok(Operation::Convert),
            "generate" => Ok(Operation::Generate),
            "generatepattern" => Ok(Operation::GeneratePattern),
            "generateflow" => Ok(Operation::GenerateFlow),
            "compile" => Ok(Operation::Compile),
            "interactive" => Ok(Operation::Interactive),
            "web" => Ok(Operation::Web),
            "app" => Ok(Operation::App),
            "appcommand" => Ok(Operation::AppCommand),
            _ => Err(format!("Unknown Operation: '{}'", &s)),
        }
    }
}

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
    in_origen_core_app: RwLock<bool>,
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
    cli_version: RwLock<Option<Version>>,
    pub origen_core_support_version: Version,
    pub origen_metal_backend_version: Version,
    other_build_info: RwLock<HashMap<String, String>>,
    _custom_tester_ids: RwLock<Vec<String>>,
    testers_eq: RwLock<Vec<Vec<SupportedTester>>>,
    testers_neq: RwLock<Vec<Vec<SupportedTester>>>,
    unique_id: RwLock<usize>,
    debug_enabled: RwLock<bool>,
    _operation: RwLock<Operation>,
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

        let version = match Version::new_semver(built_info::PKG_VERSION) {
            Ok(v) => v,
            Err(_e) => Version::default(),
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
            cli_version: RwLock::new(None),
            origen_core_support_version: Version::new_semver({
                let mut v: &str = "";
                for d in built_info::DEPENDENCIES.iter() {
                    if d.0 == "origen-core-support" {
                        v = d.1;
                        break;
                    }
                }
                if v == "" {
                    panic!("Could not determine origen-core-support version")
                }
                v
            })
            .unwrap(),
            origen_metal_backend_version: Version::new_semver(&origen_metal::VERSION).unwrap(),
            other_build_info: RwLock::new(HashMap::new()),
            is_app_in_origen_dev_mode: origen_dev_mode,
            in_origen_core_app: RwLock::new(false),
            _custom_tester_ids: RwLock::new(vec![]),
            testers_eq: RwLock::new(vec![]),
            testers_neq: RwLock::new(vec![]),
            unique_id: RwLock::new(0),
            debug_enabled: RwLock::new(false),
            _operation: RwLock::new(Operation::None),
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
    pub fn push_testers_eq(&self, testers: Vec<SupportedTester>) -> OrigenResult<()> {
        if testers.is_empty() {
            return error!("No tester type(s) given");
        }
        let mut testers_eq = self.testers_eq.write().unwrap();
        // This may not be correct, makes it harder to call re-useable functions that make their own tester selections
        //let testers_neq = self.testers_neq.read().unwrap();
        //// Verify that the new selection does not violate any existing tester exclusion
        //if !testers_neq.is_empty() {
        //    for t_neqs in &*testers_neq {
        //        for t_neq in t_neqs {
        //            for t_eq in &testers {
        //                if t_neq.is_compatible_with(t_eq) {
        //                    return error!(
        //                        "Can't select tester '{}' within a block that already excludes it",
        //                        t_eq,
        //                    );
        //                }
        //            }
        //        }
        //    }
        //}
        // When some testers are already selected, the application is only allowed to select a subset of them,
        // so verify that all given testers are already contained in the last selection
        //if !testers_eq.is_empty() {
        //    let last = testers_eq.last().unwrap();
        //    for t in &testers {
        //        if !last
        //            .iter()
        //            .any(|current_tester| t.is_compatible_with(current_tester))
        //        {
        //            return error!(
        //                "Can't select tester '{}' within a block that selects '{}'",
        //                t,
        //                last.iter()
        //                    .map(|t| t.to_string())
        //                    .collect::<Vec<String>>()
        //                    .join(", ")
        //            );
        //        }
        //    }
        //}
        testers_eq.push(testers.clone());
        Ok(())
    }

    /// Corresponds to the start of a tester except (tester not equal to) block of code, returns an error if the given tester
    /// selection is not valid within any existing eq or neq blocks
    pub fn push_testers_neq(&self, testers: Vec<SupportedTester>) -> OrigenResult<()> {
        if testers.is_empty() {
            return error!("No tester type(s) given");
        }
        let mut testers_neq = self.testers_neq.write().unwrap();
        testers_neq.push(testers.clone());
        Ok(())
    }

    /// Corresponds to the end of a tester specific block, returns an error if no tester specific
    /// selection currently exists
    pub fn pop_testers_eq(&self) -> OrigenResult<()> {
        let mut current_testers = self.testers_eq.write().unwrap();
        if current_testers.is_empty() {
            return error!("There has been an attempt to close a tester-specific block, but none is currently open");
        }
        let _ = current_testers.pop();
        Ok(())
    }

    /// Corresponds to the end of a tester exclusion block, returns an error if no tester exlcusion
    /// selection currently exists
    pub fn pop_testers_neq(&self) -> OrigenResult<()> {
        let mut current_testers = self.testers_neq.write().unwrap();
        if current_testers.is_empty() {
            return error!("There has been an attempt to close a tester-exclusion block, but none is currently open");
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

    pub fn set_debug_enabled(&self, val: bool) {
        let mut debug_enabled = self.debug_enabled.write().unwrap();
        *debug_enabled = val;
    }

    pub fn is_debug_enabled(&self) -> bool {
        *self.debug_enabled.read().unwrap()
    }

    pub fn set_operation(&self, val: Operation) {
        let mut operation = self._operation.write().unwrap();
        *operation = val;
    }

    pub fn operation(&self) -> Operation {
        *self._operation.read().unwrap()
    }

    pub fn set_cli_location(&self, loc: Option<String>) {
        if let Some(loc) = loc {
            let mut cli_loc = self.cli_location.write().unwrap();
            *cli_loc = Some(PathBuf::from(loc));
        }
    }

    pub fn set_cli_version(&self, ver: Option<String>) {
        if let Some(v) = ver {
            let mut cli_ver = self.cli_version.write().unwrap();
            *cli_ver = Some(Version::new_semver(&v).unwrap());
        }
    }

    pub fn cli_location(&self) -> Option<PathBuf> {
        self.cli_location.read().unwrap().to_owned()
    }

    pub fn cli_version(&self) -> Option<Version> {
        self.cli_version.read().unwrap().to_owned()
    }

    pub fn update_other_build_info(&self, key: &str, item: &str) -> OrigenResult<()> {
        self.other_build_info
            .write()
            .unwrap()
            .insert(key.to_string(), item.to_string());
        Ok(())
    }

    pub fn other_build_info(&self) -> HashMap<String, String> {
        self.other_build_info.read().unwrap().to_owned()
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

    pub fn in_origen_core_app(&self) -> bool {
        *self.in_origen_core_app.read().unwrap()
    }

    pub fn set_in_origen_core_app(&self, stat: bool) {
        let mut s = self.in_origen_core_app.write().unwrap();
        *s = stat;
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
    pub fn with_output_dir<T, F>(&self, change_dir: bool, mut f: F) -> OrigenResult<T>
    where
        F: FnMut(&Path) -> OrigenResult<T>,
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
    pub fn with_reference_dir<T, F>(&self, change_dir: bool, mut f: F) -> OrigenResult<T>
    where
        F: FnMut(Option<&PathBuf>) -> OrigenResult<T>,
    {
        let dir = self.reference_dir();
        if change_dir && dir.is_some() {
            let dir = dir.unwrap();
            with_dir(&dir, || f(Some(&dir)))
        } else {
            f(dir.as_ref())
        }
    }

    // pub fn current_user(&self) -> &User {
    //     &self._current_user.read().unwrap()
    // }
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

pub fn get_home_dir() -> PathBuf {
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
