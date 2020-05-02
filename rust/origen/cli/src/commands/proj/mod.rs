mod bom;
mod package;

use bom::BOM;
use clap::ArgMatches;
use origen::core::file_handler::File;
use origen::core::term;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};
use tera::{Context, Tera};

static BOM_FILE: &str = "bom.toml";
static README_FILE: &str = "README.md";

pub fn run(matches: &ArgMatches) {
    match matches.subcommand_name() {
        Some("init") => {
            let dir = get_dir_or_pwd(matches.subcommand_matches("init").unwrap());
            let f = dir.join(BOM_FILE);
            if f.exists() {
                error(
                    &format!("Found an existing '{}' in '{}'", BOM_FILE, dir.display()),
                    Some(1),
                );
            }
            File::create(f).write(include_str!("templates/project_bom.toml"));
            let f = dir.join(README_FILE);
            if !f.exists() {
                File::create(f).write(include_str!("templates/project_README.md"));
            }
        }
        Some("create") => {
            // First convert the new workspace location to an absolute path
            let path = matches
                .subcommand_matches("create")
                .unwrap()
                .value_of("path")
                .unwrap();
            let mut path = PathBuf::from(path);
            if !path.is_absolute() {
                path = pwd().join(path);
            }
            // Now find the last dir in the path that exists, that will be used to collect the BOM
            let mut dir = path.clone();
            while !dir.exists() {
                dir.pop();
            }
            let bom = BOM::for_dir(&dir);
            if bom.files.len() == 0 {
                error(
                    &format!(
                        "No BOM files were found within the parent directories of '{}'",
                        path.display()
                    ),
                    Some(1),
                );
            }
            // Allow an existing directory to be specified for the new workspace, as long as it is empty
            if path.exists() && path.is_dir() {
                let is_empty: bool = match path.read_dir() {
                    Ok(mut x) => x.next().is_none(),
                    Err(_e) => {
                        error(
                            &format!(
                                "There was a problem reading directory '{}', do you have permission to read it?:",
                                path.display(),
                            ),
                            Some(1),
                        );
                        unreachable!()
                    }
                };
                if !is_empty {
                    let b = BOM::for_dir(&path);
                    if b.is_workspace() {
                        error(
                            &format!("The workspace '{}' already exists, did you mean to run the 'update' command instead?", path.display()),
                            Some(1),
                        );
                    } else {
                        error(
                            &format!("The directory '{}' already exists, though it does not appear to be a workspace, did you give the correct path?", path.display()),
                            Some(1),
                        );
                    }
                }
            } else {
                validate_path(&path, false, true);
            }
            if bom.is_workspace() {
                error(
                    &format!("A workspace can't be created within a workspace.\nCan't create a workspace at '{}' because '{}' is a workspace", path.display(),
                    bom.files.last().unwrap().parent().unwrap().display()
                ),
                    Some(1),
                );
            }
            fs::create_dir_all(&path).expect(&format!(
                "Couldn't create '{}', do you have the required permissions?",
                path.display()
            ));
            // Write out a BOM file for the new workspace
            let mut tera = Tera::default();
            let mut context = Context::new();
            context.insert("bom", &bom);
            let contents = tera
                .render_str(include_str!("templates/workspace_bom.toml"), &context)
                .unwrap();
            File::create(path.join(BOM_FILE)).write(&contents);
            // Now populate the packages
            log_info!("Fetching {} packages", bom.packages.len());
            let mut errors = false;
            for (id, package) in &bom.packages {
                match package.create(&path) {
                    Ok(()) => display_green!("OK"),
                    Err(e) => {
                        log_error!("{}", e);
                        log_error!("Failed to create package '{}'", id);
                        errors = true;
                    }
                }
            }
            if bom.create_links().is_err() {
                errors = true;
            };
            if errors {
                exit_error!();
            } else {
                exit_success!();
            }
        }
        Some("update") => {
            let mut packages: Vec<PathBuf> = match matches.values_of("packages") {
                Some(pkgs) => pkgs.map(|p| PathBuf::from(p)).collect(),
                None => vec![],
            };
            let bom = BOM::for_dir(&pwd());
            if !bom.is_workspace() {
                error("The update command must be run from within an existing workspace, please cd to your target workspace and try again", Some(1));
            }
            let mut errors = false;
            if packages.is_empty() {
                log_info!("Updating {} packages...", &bom.packages.len());
                for (id, package) in &bom.packages {
                    match package.update(bom.root()) {
                        Ok(()) => display_green!("OK"),
                        Err(e) => {
                            log_error!("{}", e);
                            log_error!("Failed to update package '{}'", id);
                            errors = true;
                        }
                    }
                }
            } else {
                log_info!("Updating {} packages...", packages.len());
            }
            if bom.create_links().is_err() {
                errors = true;
            };
            if errors {
                exit_error!();
            } else {
                exit_success!();
            }
        }
        Some("bom") => {
            let dir = get_dir_or_pwd(matches.subcommand_matches("bom").unwrap());
            let bom = BOM::for_dir(&dir);
            println!("{}", bom);
        }
        None => unreachable!(),
        _ => unreachable!(),
    }
}

fn pwd() -> PathBuf {
    let dir = match env::current_dir() {
        Err(_e) => {
            error("Something has gone wrong trying to resolve the PWD, is it stale or do you not have read access to it?", Some(1));
            unreachable!();
        }
        Ok(d) => d,
    };
    dir
}

fn get_dir_or_pwd(matches: &ArgMatches) -> PathBuf {
    let dir = match matches.value_of("dir") {
        Some(x) => PathBuf::from(x),
        None => pwd(),
    };
    validate_path(&dir, true, true);
    dir.canonicalize().unwrap()
}

fn error(msg: &str, exit_code: Option<i32>) {
    term::red("error: ");
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
        if is_dir && path.is_file() {
            error(
                &format!(
                    "Expected '{}' to be a directory, but it is a file",
                    path.display()
                ),
                Some(1),
            );
        } else if !is_dir && path.is_dir() {
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
