mod bom;
mod package;

use bom::BOM;
use clap::ArgMatches;
use origen::core::file_handler::File;
use origen::core::term;
use package::Package;
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
                error_and_exit(
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
                error_and_exit(
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
                        error_and_exit(
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
                        error_and_exit(
                            &format!("The workspace '{}' already exists, did you mean to run the 'update' command instead?", path.display()),
                            Some(1),
                        );
                    } else {
                        error_and_exit(
                            &format!("The directory '{}' already exists, though it does not appear to be a workspace, did you give the correct path?", path.display()),
                            Some(1),
                        );
                    }
                }
            } else {
                validate_path(&path, false, true);
            }
            if bom.is_workspace() {
                error_and_exit(
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
            // Converting this to a vector here as the template was printing out the package list
            // in reverse order when given the index map
            let packages: Vec<&Package> = bom.packages.iter().map(|(_id, pkg)| pkg).collect();
            context.insert("packages", &packages);
            let contents = tera
                .render_str(include_str!("templates/workspace_bom.toml"), &context)
                .unwrap();
            File::create(path.join(BOM_FILE)).write(&contents);
            // Now populate the packages
            log_info!("Fetching {} packages", bom.packages.len());
            let mut errors = false;
            for (id, package) in &bom.packages {
                display!("Populating '{}' ... ", id);
                match package.create(&path) {
                    Ok(()) => display_greenln!("OK"),
                    Err(e) => {
                        log_error!("{}", e);
                        log_error!("Failed to create package '{}'", id);
                        errors = true;
                    }
                }
            }
            display!("Creating links ... ");
            // Create a new BOM instance for the new workspace so that create_links runs on the right root dir
            let bom = BOM::for_dir(&path);
            match bom.create_links(false) {
                Ok(_) => display_greenln!("OK"),
                Err(e) => {
                    log_error!("There was a problem creating the workspace's links:");
                    log_error!("{}", e);
                    errors = true;
                }
            }
            if errors {
                exit_error!();
            } else {
                exit(0);
            }
        }
        Some("update") => {
            let matches = matches.subcommand_matches("update").unwrap();
            let force = matches.is_present("force");
            let links_only = matches.is_present("links");
            let package_args: Vec<PathBuf> = match matches.values_of("packages") {
                Some(pkgs) => pkgs.map(|p| PathBuf::from(p)).collect(),
                None => vec![],
            };
            let exclude_args: Vec<PathBuf> = match matches.values_of("exclude") {
                Some(pkgs) => pkgs.map(|p| PathBuf::from(p)).collect(),
                None => vec![],
            };
            let bom = BOM::for_dir(&pwd());
            if !bom.is_workspace() {
                error_and_exit("The update command must be run from within an existing workspace, please cd to your target workspace and try again", Some(1));
            }
            // Clean the package args to remove any overly broad references, such as a reference to "." (meaning
            // the current workspace).
            // References will override a package's exclude state from the BOM, however a user entering
            // `origen proj update .` would probably expect it to behave like `origen proj update` and apply
            // the default excludes. So we throwaway any references to the workspace and above to play it safe.
            let package_args: Vec<&PathBuf> = package_args
                .iter()
                .filter(|pkg_ref| {
                    let root = bom.root();
                    match pkg_ref.canonicalize() {
                        Err(_e) => true,
                        Ok(p) => {
                            let is_root = match p.strip_prefix(&root) {
                                Err(_) => false,
                                Ok(res) => res.to_str() == Some(""),
                            };
                            let above_workspace = root.strip_prefix(&p).is_ok();
                            !is_root && !above_workspace
                        }
                    }
                })
                .collect();
            // Turn any exclude args into package references
            let mut exclude_packages: Vec<&Package> = vec![];
            for pkg_ref in exclude_args {
                match bom.packages_from_ref(&pkg_ref) {
                    Err(_e) => log_warning!("Exclude reference '{}' did not resolve to any packages in the current workspace", pkg_ref.display()),
                    Ok(packages) => {
                        for pkg in packages {
                            if !exclude_packages.contains(&pkg) {
                                exclude_packages.push(&pkg);
                            }
                        }
                    }
                }
            }
            let mut errors = false;
            let mut packages_requiring_force: Vec<&str> = vec![];
            if package_args.is_empty() {
                log_info!("Updating {} packages...", &bom.packages.len());
                for (id, package) in &bom.packages {
                    if package.is_excluded() || exclude_packages.contains(&package) || links_only {
                        displayln!("Skipping '{}'", id);
                    } else {
                        display!("Updating '{}' ... ", id);
                        match package.update(bom.root(), force) {
                            Ok(force_required) => {
                                if !force_required {
                                    display_greenln!("OK");
                                } else {
                                    packages_requiring_force.push(id);
                                }
                            }
                            Err(e) => {
                                log_error!("{}", e);
                                log_error!("Failed to update package '{}'", id);
                                errors = true;
                            }
                        }
                    }
                }
            } else {
                log_info!("Updating {} packages...", package_args.len());
                for pkg_ref in package_args {
                    match bom.packages_from_ref(&pkg_ref) {
                        Err(e) => {
                            log_error!("{}", e);
                            log_error!("The package referece '{}' is invalid", pkg_ref.display());
                            errors = true;
                        }
                        Ok(packages) => {
                            if packages.is_empty() {
                                log_error!(
                                    "No package was found corresponding to '{}'",
                                    pkg_ref.display()
                                );
                                errors = true;
                            } else {
                                for package in packages {
                                    if exclude_packages.contains(&package) || links_only {
                                        displayln!("Skipping '{}'", package.id);
                                    } else {
                                        display!("Updating '{}' ... ", package.id);
                                        match package.update(bom.root(), force) {
                                            Ok(force_required) => {
                                                if !force_required {
                                                    display_greenln!("OK");
                                                } else {
                                                    packages_requiring_force.push(&package.id);
                                                }
                                            }
                                            Err(e) => {
                                                log_error!("{}", e);
                                                log_error!(
                                                    "Failed to update package '{}'",
                                                    package.id
                                                );
                                                errors = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let mut links_force_required = false;
            display!("Updating links ... ");
            match bom.create_links(force) {
                Ok(force_required) => {
                    if !force_required {
                        display_greenln!("OK");
                    } else {
                        links_force_required = true;
                    }
                }
                Err(e) => {
                    log_error!("There was a problem creating the workspace's links:");
                    log_error!("{}", e);
                    errors = true;
                }
            }
            if errors {
                exit_error!();
            } else {
                if links_force_required || !packages_requiring_force.is_empty() {
                    displayln!("");
                    display_redln!("The update was not completed successfully due to the possibility of losing local work, you can run the following command to force alignment with the current BOM:");
                    displayln!("");
                    let mut command = "  origen proj update --force".to_string();
                    if packages_requiring_force.is_empty() {
                        command += " --links";
                    } else {
                        command += " ";
                        command += &packages_requiring_force.join(" ");
                    }
                    displayln!("{}", command);
                    exit(1);
                } else {
                    exit(0);
                }
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
            error_and_exit("Something has gone wrong trying to resolve the PWD, is it stale or do you not have read access to it?", Some(1));
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

fn error_and_exit(msg: &str, exit_code: Option<i32>) {
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
            error_and_exit(
                &format!("The {} '{}' does not exist", t, path.display()),
                Some(1),
            );
        }
        if is_dir && path.is_file() {
            error_and_exit(
                &format!(
                    "Expected '{}' to be a directory, but it is a file",
                    path.display()
                ),
                Some(1),
            );
        } else if !is_dir && path.is_dir() {
            error_and_exit(
                &format!(
                    "Expected '{}' to be a file, but it is a directory",
                    path.display()
                ),
                Some(1),
            );
        }
    } else if path.exists() {
        error_and_exit(
            &format!("The path '{}' already exists", path.display()),
            Some(1),
        );
    }
}
