//! A default target/env for an application can be set by config/application.toml, this can
//! then be overridden by the user by setting a temporary workspace default via the "origen t"
//! and "origen e" commands.
//! These commands store the user's selections in .origen/application.toml and this file should
//! NOT be checked into revision control.
//!
//! The target can be further overridden for a particular origen command invocation via
//! the -t and -e options, or programmatically within the application code, however that is all
//! handled on the front end in Python code.
use crate::app;
use std::path::PathBuf;
use walkdir::WalkDir;
// Can be used to turn a relative path in an absolute
//use path_clean::{PathClean};
use pathdiff::diff_paths;
use regex::{escape, Regex};
use std::fs;
use std::process::exit;
use std::collections::HashSet;

#[macro_export]
macro_rules! clean_target {
    ($name:expr, $dir:expr, $return_file:expr) => {{
        $crate::core::application::target::clean_name($name, $dir, $return_file, &$crate::app().unwrap().root)
    }}
}

macro_rules! toml {
    ($root:expr) => {{
        $root.join(".origen").join("application.toml")
    }}
}

/// Sanitizes the given target/env name and returns it, but will exit the process if it does
/// not uniquely identify a single target/env file.
/// Set the last arg to true to return the path to the matching target instead.
pub fn clean_name(name: &str, dir: &str, return_file: bool, root: &PathBuf) -> String {
    let matches = matches(name, dir, root);
    let t = dir.trim_end_matches("s");

    if matches.len() == 0 {
        println!(
            "No matching {} '{}' found, here are the available {}s:",
            t, name, t
        );
        for file in all(&root.join(dir)).iter() {
            println!(
                "    {}",
                diff_paths(&file, &root.join(dir))
                    .unwrap()
                    .display()
            );
        }
    } else if matches.len() > 1 {
        println!(
            "That {} name '{}' is ambiguous, please try again to narrow it down to one of these:",
            t, name
        );
        for file in matches.iter() {
            println!(
                "    {}",
                diff_paths(&file, &root.join(dir))
                    .unwrap()
                    .display()
            );
        }
    } else {
        if return_file {
            return format!("{}", matches[0].display());
        } else {
            let clean = format!(
                "{}",
                diff_paths(&matches[0], &root.join(dir))
                    .unwrap()
                    .display()
            );
            return clean;
        }
    }
    exit(1);
}

/// Returns an array of possible target/environment files that match the given name/snippet
// TODO: look into updating this to use the PathBuf PartialEq Trait to compare instead string compare which is prone to bugs due to OS differences
pub fn matches(name: &str, dir: &str, root: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    for file in WalkDir::new(format!("{}", root.join(dir).display())) {
        let path = file.unwrap().into_path();
        if path.is_file() {
            let mut path_str = format!(
                "{}",
                diff_paths(&path, &root.join(dir))
                    .unwrap()
                    .display()
            );
            // in case we're running on Windows normalize to linux style path separator character
            path_str = path_str.replace("\\", "/").replace("//", "/");

            if path_str.contains(name) {
                files.push(path);
            // Try again without the leading dir in case the user has supplied a path
            } else {
                let re = Regex::new(format!(r#".*{}(\\|/)"#, dir).as_str()).unwrap();
                let mut new_name: String = re.replace_all(&name, "").into();

                // in case we're running on Windows normalize to linux style path separator character
                new_name = new_name.replace("\\", "/").replace("//", "/");

                if path_str.contains(&new_name) {
                    files.push(path);
                }
            }
        }
    }
    // After collecting all the matches, if the size > 1 then filter again for exact matches
    if files.len() > 1 {
        files = files
            .into_iter()
            .filter(|path| path.file_name().unwrap().to_str().unwrap() == &format!("{}.py", name))
            .collect();
    }
    files
}

/// Gets the currently enabled targets
pub fn get(full_paths: bool) -> Option<Vec<String>> {
    app()
        .unwrap()
        .with_config_mut(|config| {
            config.refresh();
            match config.target.as_ref() {
                Some(targets) => Ok(Some(
                    targets
                        .iter()
                        .map(|t| clean_target!(t, "targets", full_paths))
                        .collect::<Vec<String>>()
                        .clone(),
                )),
                None => Ok(None),
            }
        })
        .unwrap()
}

/// Sets the targets, overriding any that may be present
pub fn set(targets: Vec<&str>) -> Vec<String> {
    set_at_root(targets, &app().unwrap().root)
}

pub fn set_at_root(targets: Vec<&str>, root: &PathBuf) -> Vec<String> {
    let mut to_set = vec!();
    for t in targets.iter() {
        let cn = clean_name(t, "targets", true, root);
        if to_set.contains(&cn) {
            log_error!("Target '{}' appears multiple times in the TARGETS list ({})", t, cn);
            exit(1);
        }
        to_set.push(cn.clone());
    }
    set_workspace_array_at_root("target", to_set.clone(), root);
    to_set
}

/// Resets (deletes) the target back to its default value
pub fn reset() {
    delete_val("target")
}

pub fn clear() {
    delete_val("target");
    set_workspace_array("target", vec!())
}

/// Enables additional targets in the workspace
pub fn add(targets: Vec<&str>) {
    let mut current: Vec<String> = app()
        .unwrap()
        .with_config(|config| {
            let c = match &config.target {
                Some(targets) => targets.clone(),
                None => vec![],
            }
            .iter()
            .map(|t| clean_target!(t, "targets", true))
            .collect();
            Ok(c)
        })
        .unwrap();

    let mut added = Vec::new();
    for t in targets.iter() {
        // Check that the targets to add are valid
        let clean_t = clean_target!(t, "targets", true);

        // If the target is already added, remove it from its current position and reapply it in the order
        // given here
        current.retain(|c| *c != clean_t);

        if added.contains(&clean_t) {
            log_error!("Target '{}' appears multiple times in the TARGETS list ({})", t, clean_t);
            exit(1);
        }
        added.push(clean_t);
    }
    current.extend(added);

    set_workspace_array("target", current);
}

/// Disables currently enables targets in the workspace
pub fn remove(targets: Vec<&str>) {
    let mut current: Vec<String> = app()
        .unwrap()
        .with_config(|config| {
            let c: Vec<String> = match &config.target {
                Some(targets) => targets.clone(),
                None => vec![],
            }
            .iter()
            .map(|t| clean_target!(t, "targets", true))
            .collect();
            Ok(c)
        })
        .unwrap();

    let mut removed = HashSet::new();
    let mut len = current.len();
    for t in targets.iter() {
        let clean_t = clean_target!(t, "targets", true);

        if removed.contains(&clean_t) {
            log_error!("Target '{}' appears multiple times in the TARGETS list ({})", t, clean_t);
            exit(1);
        }

        current.retain(|c| *c != clean_t);
        let new_len = current.len();
        if new_len == len {
            log_error!("Tried to remove non-activated target '{}' ({})", t, clean_t);
            exit(1);
        }
        len = new_len;
        removed.insert(clean_t);
    }

    if current.len() == 0 {
        println!("All targets were removed. Resetting to the default target.");
        reset();
    } else {
        set_workspace_array("target", current);
    }
}

/// Returns all files from the given directory
pub fn all(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    for file in WalkDir::new(dir) {
        let path = file.unwrap().into_path();
        if path.is_file() && path.extension().unwrap_or("".as_ref()) == "py" {
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

/// Sets an Array-of-Strings workspace variable
pub fn set_workspace_array(key: &str, vals: Vec<String>) {
    set_workspace_array_at_root(key, vals, &app().unwrap().root)
}

pub fn set_workspace_array_at_root(key: &str, vals: Vec<String>, root: &PathBuf) {
    ensure_app_dot_toml_at_root(root);
    delete_val_at_root(key, root);
    add_val_array_at_root(key, vals, root);
}

/// Deletes the given key (and its val) from .origen/application.toml if it exists
pub fn delete_val(key: &str) {
    delete_val_at_root(key, &app().unwrap().root);
}

pub fn delete_val_at_root(key: &str, root: &PathBuf) {
    let path = toml!(root);
    let data = fs::read_to_string(&path).expect(&format!("Unable to read file {}", &path.display()));
    let re = Regex::new(format!(r#"{}\s?=.*(\r\n|\n)?"#, escape(key)).as_str()).unwrap();
    let new_data: String = re.replace_all(&data, "").into();
    fs::write(&path, new_data).expect(&format!("Unable to write file {}", &path.display()));
}

/// Appends the given key/val pair to the end of .origen/application.toml
fn add_val(key: &str, val: &str) {
    let path = app().unwrap().root.join(".origen").join("application.toml");
    let data = fs::read_to_string(path).expect("Unable to read file .origen/application.toml");
    let new_data = format!("{}\n{} = \"{}\"", data.trim(), key, val);
    fs::write(
        app().unwrap().root.join(".origen").join("application.toml"),
        new_data,
    )
    .expect("Unable to write file .origen/application.toml!");
}

/// Appends the given key/val pair to the end of .origen/application.toml
// fn add_val_array(key: &str, vals: Vec<String>) {
//     add_val_array_at_root(key, vals, &app().unwrap().root);
// }

fn add_val_array_at_root(key: &str, vals: Vec<String>, root: &PathBuf) {
    let path = toml!(root);
    let data = fs::read_to_string(&path).expect(&format!("Unable to read file {}", &path.display()));

    // Note: use string literals here to account for Windows paths
    let new_data = format!(
        "{}\n{} = [{}]",
        data.trim(),
        key,
        vals.iter()
            .map(|v| format!("'{}'", v))
            .collect::<Vec<String>>()
            .join(", ")
    );
    fs::write(&path, new_data).expect(&format!("Unable to write file {}", &path.display()));
}

/// Verifies that .origen/application.toml exists and if not creates one
fn ensure_app_dot_toml() {
    ensure_app_dot_toml_at_root( &app().unwrap().root)
}

fn ensure_app_dot_toml_at_root(root: &PathBuf) {
    let path = root.join(".origen");
    if !path.exists() {
        fs::create_dir(&path).expect("Unable to create directory .origen!");
    }
    let path = path.join("application.toml");
    if !path.exists() {
        let data =
            "# This file is generated by Origen and should not be checked into revision control";
        fs::write(&path, data).expect(&format!("Unable to write file {}", &path.display()));
    }
}
