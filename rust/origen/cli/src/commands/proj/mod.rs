use clap::ArgMatches;
use origen::core::file_handler::File;
use origen::core::term;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};

static PROJECT_BOM: &str = ".project_bom.toml";
static WORKSPACE_BOM: &str = ".workspace_bom.toml";

pub fn run(matches: &ArgMatches) {
    match matches.subcommand_name() {
        Some("init") => {
            let m = matches.subcommand_matches("init").unwrap();

            let dir = match m.value_of("path") {
                Some(x) => PathBuf::from(x),
                None => env::current_dir().expect("Something has gone wrong trying to resolve the PWD, is it stale or do you not have read access to it?"),
            };
            validate_path(&dir, true, true);
            let mut f = dir.clone();
            f.push(PROJECT_BOM);
            if f.exists() {
                error(
                    &format!("Found an existing '{}' in '{}'", PROJECT_BOM, dir.display()),
                    Some(1),
                );
            }
            File::create(f).write(include_str!("templates/project_bom.toml"));
        }
        Some("create") => {
            println!("create");
        }
        Some("update") => {
            println!("update");
        }
        Some("bom") => {
            println!("bom");
        }
        _ => unreachable!(),
    }
}

fn error(msg: &str, exit_code: Option<i32>) {
    term::red("error: ");
    println!("{}", msg);
    if let Some(c) = exit_code {
        exit(c);
    }
}

fn success(msg: &str, exit_code: Option<i32>) {
    term::green("success: ");
    println!("{}", msg);
    if let Some(c) = exit_code {
        exit(c);
    }
}

// Will exit and print an error message if the given path reference is invalid.
// Caller must specify whether they want the path to exist or not and whether they expect it
// to be a file or a dir (if applicable).
fn validate_path(path: &Path, is_present: bool, is_dir: bool) {
    if is_present {
        let t = if is_dir { "directory" } else { "file" }.to_string();
        if !path.exists() {
            error(
                &format!("The {} '{}' does not exist", t, path.display()),
                Some(1),
            );
        }
        let md = fs::metadata(path).unwrap();
        if is_dir && md.is_file() {
            error(
                &format!(
                    "Expected '{}' to be a directory, but it is a file",
                    path.display()
                ),
                Some(1),
            );
        } else if !is_dir && md.is_dir() {
            error(
                &format!(
                    "Expected '{}' to be a file, but it is a directory",
                    path.display()
                ),
                Some(1),
            );
        }
    } else if path.exists() {
        error(
            &format!("The path '{}' already exists", path.display()),
            Some(1),
        );
    }
}

fn is_project_dir() {}

fn is_workspace_dir() {}

fn package_name() {}
