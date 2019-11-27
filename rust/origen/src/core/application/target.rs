use crate::STATUS;
/// Responsible for managing the target and environment selection and loading.
///
/// A default target/env for an application can be set by config/application.toml, this can
/// then be overridden by the user by setting a temporary workspace default via the "origen t"
/// and "origen e" commands.
/// These commands stores the user's selection in .origen/application.toml and this file should
/// not be checked into revision control.
///
/// Finally, the target can be further overridden for a particular origen command invocation via
/// the -t and -e options, or programmatically within the application code.
use std::path::PathBuf;
use walkdir::WalkDir;
// Can be used to turn a relative path in an absolute
//use path_clean::{PathClean};
use pathdiff::diff_paths;
use regex::{escape, Regex};
use std::fs;
//use std::sync::Mutex;
use crate::APPLICATION_CONFIG;

lazy_static! {
    /// Stores the current target/environment selection and load state. Note that similar
    /// values from APPLICATION_CONFIG differ in the following respect:
    /// The values in APPLICATION_CONFIG refer only to the values derived from
    /// config/application.toml and .origen/application.toml (set by origen t and e commands).
    /// The value here also takes account of further overriding by -t or -e options or changes
    /// to the target that have been applied programmatically during application execution.
    /// In short, this is the authority on what the current target actually is at any given time.
    // The Mutex here is required to make this mutable, i.e. to change targets at runtime
    // See: https://github.com/rust-lang-nursery/lazy-static.rs/issues/39
    //pub static ref CURRENT_TARGET: Mutex<Target> = Mutex::new(Target::default());
    pub static ref CURRENT_TARGET: Target = Target::default();
}

#[derive(Debug)]
pub struct Target {
    pub target_name: Option<String>,
    pub target_file: Option<PathBuf>,
    pub env_name: Option<String>,
    pub env_file: Option<PathBuf>,
    pub is_loaded: bool,
}

impl Default for Target {
    fn default() -> Target {
        let mut t = Target {
            target_name: None,
            target_file: None,
            env_name: None,
            env_file: None,
            is_loaded: false,
        };
        if APPLICATION_CONFIG.target.is_some() {
            t.target_name = APPLICATION_CONFIG.target.clone();
            let name = APPLICATION_CONFIG.target.as_ref().unwrap();
            let files = matches(name, "targets");
            if files.len() == 0 {
                println!(
                    "Something has gone wrong, the application has requested a target \
                     named {}, but none can be found with that name.",
                    name
                );
                println!(
                    "Please review any -t option given to the current command, or change \
                     the target via the 'origen t' command as required."
                );
                std::process::exit(1);
            } else if files.len() > 1 {
                println!(
                    "Something has gone wrong, the application has requested a target \
                     named {}, but multiple targets matching that name have been found.",
                    name
                );
                println!(
                    "Please review any -t option given to the current command, or change \
                     the target via the 'origen t' command as required."
                );
                std::process::exit(1);
            } else {
                t.target_file = Some(files[0].clone());
            }
        }
        if APPLICATION_CONFIG.environment.is_some() {
            t.env_name = APPLICATION_CONFIG.environment.clone();
            let name = APPLICATION_CONFIG.environment.as_ref().unwrap();
            let files = matches(name, "environments");
            if files.len() == 0 {
                println!(
                    "Something has gone wrong, the application has requested an environment \
                     named {}, but none can be found with that name.",
                    name
                );
                println!(
                    "Please review any -e option given to the current command, or change \
                     the environment via the 'origen e' command as required."
                );
                std::process::exit(1);
            } else if files.len() > 1 {
                println!(
                    "Something has gone wrong, the application has requested an environment \
                     named {}, but multiple environments matching that name have been found.",
                    name
                );
                println!(
                    "Please review any -e option given to the current command, or change \
                     the environment via the 'origen e' command as required."
                );
                std::process::exit(1);
            } else {
                t.env_file = Some(files[0].clone());
            }
        }
        t
    }
}

/// Returns an array of possible target/environment files that match the given name/snippet
pub fn matches(name: &str, dir: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    for file in WalkDir::new(format!("{}", STATUS.root.join(dir).display())) {
        let path = file.unwrap().into_path();
        if path.is_file() {
            let path_str = format!(
                "{}",
                diff_paths(&path, &STATUS.root.join(dir)).unwrap().display()
            );
            if path_str.contains(name) {
                files.push(path);
            // Try again without the leading dir in case the user has supplied a path
            } else {
                let re = Regex::new(format!(r#".*{}(\\|/)"#, dir).as_str()).unwrap();
                let new_name: String = re.replace_all(&name, "").into();
                if path_str.contains(&new_name) {
                    files.push(path);
                }
            }
        }
    }
    files
}

/// Returns all files from the given directory
pub fn all(dir: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    for file in WalkDir::new(format!("{}", STATUS.root.join(dir).display())) {
        let path = file.unwrap().into_path();
        if path.is_file() {
            files.push(path);
        }
    }
    files
}

/// Sets the given key and value (currently only a string is supported) in
/// .origen/application.toml
pub fn set_workspace(key: &str, val: &str) {
    ensure_app_dot_toml();
    delete_val(key);
    add_val(key, val);
}

/// Deletes the given key (and its val) from .origen/application.toml if it exists
pub fn delete_val(key: &str) {
    let path = STATUS.root.join(".origen").join("application.toml");
    let data = fs::read_to_string(path).expect("Unable to read file .origen/application.toml");
    let re = Regex::new(format!(r#"{}\s?=.*(\r\n|\n)?"#, escape(key)).as_str()).unwrap();
    let new_data: String = re.replace_all(&data, "").into();
    fs::write(
        STATUS.root.join(".origen").join("application.toml"),
        new_data,
    )
    .expect("Unable to write file .origen/application.toml!");
}

/// Appends the given key/val pair to the end of .origen/application.toml
fn add_val(key: &str, val: &str) {
    let path = STATUS.root.join(".origen").join("application.toml");
    let data = fs::read_to_string(path).expect("Unable to read file .origen/application.toml");
    let new_data = format!("{}\n{} = \"{}\"", data.trim(), key, val);
    fs::write(
        STATUS.root.join(".origen").join("application.toml"),
        new_data,
    )
    .expect("Unable to write file .origen/application.toml!");
}

/// Verifies that .origen/application.toml exists and if not creates one
fn ensure_app_dot_toml() {
    let path = STATUS.root.join(".origen");
    if !path.exists() {
        fs::create_dir(&path).expect("Unable to create directory .origen!");
    }
    let path = path.join("application.toml");
    if !path.exists() {
        let data =
            "# This file is generated by Origen and should not be checked into revision control";
        fs::write(STATUS.root.join(".origen").join("application.toml"), data)
            .expect("Unable to write file .origen/application.toml!");
    }
}
