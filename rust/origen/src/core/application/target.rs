//! A default target/env for an application can be set by config/application.toml, this can
//! then be overridden by the user by setting a temporary workspace default via the "origen t"
//! and "origen e" commands.
//! These commands store the user's selections in .origen/application.toml and this file should
//! NOT be checked into revision control.
//!
//! The target can be further overridden for a particular origen command invocation via
//! the -t and -e options, or programmatically within the application code, however that is all
//! handled on the front end in Python code.
use crate::STATUS;
use std::path::PathBuf;
use walkdir::WalkDir;
// Can be used to turn a relative path in an absolute
//use path_clean::{PathClean};
use pathdiff::diff_paths;
use regex::{escape, Regex};
use std::fs;

/// Sanitizes the given target/env name and returns it, but will exit the process if it does
/// not uniquely identify a single target/env file.
/// Set the last arg to true to return the path to the matching target instead.
pub fn clean_name(name: &str, dir: &str, return_file: bool) -> String {
    let matches = matches(name, dir);
    let t = dir.trim_end_matches("s");

    if matches.len() == 0 {
        println!("No matching {} found, here are the available {}s:", t, t);
        for file in all(dir).iter() {
            println!(
                "    {}",
                diff_paths(&file, &STATUS.root.join(dir)).unwrap().display()
            );
        }
    } else if matches.len() > 1 {
        println!(
            "That {} name is ambiguous, please try again to narrow it down to one of these:",
            t
        );
        for file in matches.iter() {
            println!(
                "    {}",
                diff_paths(&file, &STATUS.root.join(dir)).unwrap().display()
            );
        }
    } else {
        if return_file {
            return format!("{}", matches[0].display());
        } else {
            let clean = format!(
                "{}",
                diff_paths(&matches[0], &STATUS.root.join(dir))
                    .unwrap()
                    .display()
            );
            return clean;
        }
    }
    std::process::exit(1);
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
