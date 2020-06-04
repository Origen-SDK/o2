mod bom;
mod group;
mod package;
#[cfg(test)]
mod tests;

use crate::origen::revision_control::RevisionControlAPI;
use bom::BOM;
use clap::ArgMatches;
use group::Group;
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
            let dir = get_dir_or_pwd(matches.subcommand_matches("init").unwrap(), false);
            if !dir.exists() {
                fs::create_dir_all(&dir).expect(&format!(
                    "Couldn't create '{}', do you have the required permissions?",
                    dir.display()
                ));
            }
            validate_path(&dir, true, true);
            let dir = dir.canonicalize().unwrap();
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
            let groups: Vec<&Group> = bom.groups.iter().map(|(_id, grp)| grp).collect();
            context.insert("groups", &groups);
            let contents = tera
                .render_str(include_str!("templates/workspace_bom.toml.tera"), &context)
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
            if !bom.links.is_empty() {
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
            let mut links = matches.is_present("links");
            if let Some(packages) = matches.values_of("packages") {
                if packages.map(|p| p).collect::<Vec<&str>>().contains(&"all") {
                    links = true;
                }
            }
            let package_ids = get_package_ids_from_args(matches, false);
            let bom = BOM::for_dir(&pwd());
            if !bom.is_workspace() {
                error_and_exit("The update command must be run from within an existing workspace, please cd to your target workspace and try again", Some(1));
            }
            let mut errors = false;
            let mut packages_requiring_force: Vec<&str> = vec![];
            let mut packages_with_conflicts: Vec<&str> = vec![];
            log_info!("Updating {} packages...", &package_ids.len());
            for id in package_ids {
                let package = &bom.packages[&id];
                display!("Updating '{}' ... ", package.id);
                match package.update(bom.root(), force) {
                    Ok((force_required, conflicts)) => {
                        if conflicts {
                            display_redln!("CONFLICTS");
                            packages_with_conflicts.push(&package.id);
                        } else if !force_required {
                            display_greenln!("OK");
                        } else {
                            packages_requiring_force.push(&package.id);
                        }
                    }
                    Err(e) => {
                        log_error!("{}", e);
                        log_error!("Failed to update package '{}'", package.id);
                        errors = true;
                    }
                }
            }
            let mut links_force_required = false;
            if links {
                if !bom.links.is_empty() {
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
                }
            }
            if errors {
                exit_error!();
            } else {
                if links_force_required
                    || !packages_requiring_force.is_empty()
                    || !packages_with_conflicts.is_empty()
                {
                    if links_force_required || !packages_requiring_force.is_empty() {
                        displayln!("");
                        display_redln!("The following packages were not updated successfully due to the possibility of losing local work, you can run the following command to force alignment with the current BOM:");
                        displayln!("");
                        let mut command = "  origen proj update --force".to_string();
                        if packages_requiring_force.is_empty() {
                            command += " --links";
                        } else {
                            command += " ";
                            command += &packages_requiring_force.join(" ");
                        }
                        displayln!("{}", command);
                    }
                    if !packages_with_conflicts.is_empty() {
                        displayln!("");
                        display_redln!("The following packages were not updated successfully due to conflicts when trying to merge local work, you can run the following command to force alignment with the current BOM:");
                        displayln!("");
                        let mut command = "  origen proj update --force".to_string();
                        command += " ";
                        command += &packages_with_conflicts.join(" ");
                        displayln!("{}", command);
                    }
                    exit(1);
                } else {
                    exit(0);
                }
            }
        }
        Some("mods") => {
            let matches = matches.subcommand_matches("mods").unwrap();
            let package_ids = get_package_ids_from_args(matches, true);
            let bom = BOM::for_dir(&pwd());
            if !bom.is_workspace() {
                error_and_exit("The mods command must be run from within an existing workspace, please cd to your target workspace and try again", Some(1));
            }
            for id in package_ids {
                let package = &bom.packages[&id];
                if package.is_repo() {
                    display!("{} ... ", package.id);
                    let rc = package.rc(bom.root()).unwrap();
                    match rc.status(None) {
                        Err(e) => {
                            error_and_exit(&e.to_string(), Some(1));
                        }
                        Ok(status) => {
                            if status.is_modified() {
                                display_redln!("Modified");
                                if !status.added.is_empty() {
                                    displayln!("  ADDED");
                                    for file in &status.added {
                                        displayln!("    {}", file.display());
                                    }
                                }
                                if !status.removed.is_empty() {
                                    displayln!("  DELETED");
                                    for file in &status.removed {
                                        displayln!("    {}", file.display());
                                    }
                                }
                                if !status.changed.is_empty() {
                                    displayln!("  CHANGED");
                                    for file in &status.changed {
                                        displayln!("    {}", file.display());
                                    }
                                }
                                if !status.conflicted.is_empty() {
                                    displayln!("  CONFLICTED");
                                    for file in &status.conflicted {
                                        display_redln!("    {}", file.display());
                                    }
                                }
                            } else {
                                display_greenln!("Clean");
                            }
                        }
                    }
                }
            }
        }
        Some("packages") => {
            let dir = get_dir_or_pwd(matches.subcommand_matches("packages").unwrap(), true);
            let bom = BOM::for_dir(&dir);
            println!("PACKAGE GROUPS");
            for (id, g) in &bom.groups {
                println!("  {}  ({})", id, g.packages.join(", "));
            }
            println!("");
            println!("PACKAGES");
            for (id, p) in &bom.packages {
                if let Some(path) = &p.path {
                    println!("  {}  ({})", id, path.display());
                } else {
                    println!("  {}", id);
                }
            }
        }
        Some("bom") => {
            let dir = get_dir_or_pwd(matches.subcommand_matches("bom").unwrap(), true);
            let bom = BOM::for_dir(&dir);
            println!("{}", bom);
        }
        Some("clean") => {
            let matches = matches.subcommand_matches("clean").unwrap();
            let package_ids = get_package_ids_from_args(matches, true);
            let bom = BOM::for_dir(&pwd());
            if !bom.is_workspace() {
                error_and_exit("The clean command must be run from within an existing workspace, please cd to your target workspace and try again", Some(1));
            }
            for id in package_ids {
                let package = &bom.packages[&id];
                if package.is_repo() {
                    display!("{} ... ", package.id);
                    let rc = package.rc(bom.root()).unwrap();
                    match rc.revert(None) {
                        Err(e) => {
                            error_and_exit(&e.to_string(), Some(1));
                        }
                        Ok(_status) => {
                            display_greenln!("OK");
                        }
                    }
                }
            }
        }
        Some("tag") => {
            let matches = matches.subcommand_matches("tag").unwrap();
            let force = matches.is_present("force");
            let tagname = matches.value_of("name").unwrap();
            let message = matches.value_of("message");
            let package_ids = get_package_ids_from_args(matches, true);
            let mut packages_with_existing_tag: Vec<&str> = vec![];
            let bom = BOM::for_dir(&pwd());
            if !bom.is_workspace() {
                error_and_exit("The tag command must be run from within an existing workspace, please cd to your target workspace and try again", Some(1));
            }
            for id in package_ids {
                let package = &bom.packages[&id];
                if package.is_repo() {
                    display!("{} ... ", package.id);
                    let rc = package.rc(bom.root()).unwrap();
                    match rc.tag(tagname, force, message) {
                        Err(e) => {
                            if e.to_string().contains("tag already exists") {
                                packages_with_existing_tag.push(&package.id);
                                display_yellowln!("Tag already exists");
                            }
                        }
                        Ok(_status) => {
                            display_greenln!("OK");
                        }
                    }
                }
            }
            if !packages_with_existing_tag.is_empty() {
                displayln!("");
                display_yellowln!("Some packages were not tagged successfully due to the tag already existing, but not necessarily in the same place as your current workspace view.");
                display_yellowln!(
                    "You can run the following command to force the tag onto your current view:"
                );
                displayln!("");
                let mut command = format!("  origen proj tag {} ", &tagname);
                command += &packages_with_existing_tag.join(" ");
                command += " --force";
                if let Some(msg) = message {
                    command += &format!(" --message \"{}\" ", msg);
                }
                displayln!("{}", command);
                displayln!("");
                exit(1);
            }
        }
        None => unreachable!(),
        _ => unreachable!(),
    }
}

/// Resolves package and group IDs in the args to a vector of package IDs.
/// If a given ID does not match a known package or group the process will be exited with an error.
/// Optionally return all packages if no packages arg given.
fn get_package_ids_from_args(matches: &ArgMatches, return_all_if_none: bool) -> Vec<String> {
    let mut package_args: Vec<&str> = match matches.values_of("packages") {
        Some(pkgs) => pkgs.map(|p| p).collect(),
        None => vec![],
    };
    if package_args.is_empty() && return_all_if_none {
        package_args = vec!["all"];
    }
    let bom = BOM::for_dir(&pwd());
    let package_ids = bom.resolve_ids(package_args);
    if let Err(e) = &package_ids {
        error_and_exit(&e.to_string(), Some(1));
    }
    package_ids.unwrap()
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

/// If validate is not true then the path returned may not be absolute and the caller
/// is responsible for handling that (if required) after then have created it
fn get_dir_or_pwd(matches: &ArgMatches, validate: bool) -> PathBuf {
    let dir = match matches.value_of("dir") {
        Some(x) => PathBuf::from(x),
        None => pwd(),
    };
    if validate {
        validate_path(&dir, true, true);
        dir.canonicalize().unwrap()
    } else {
        dir
    }
}

fn error_and_exit(msg: &str, exit_code: Option<i32>) {
    term::red("error: ");
    println!("{}", msg);
    if let Some(c) = exit_code {
        exit(c);
    }
}

/// Will exit and print an error message if the given path reference is invalid.
/// Caller must specify whether they want the path to exist or not and whether they expect it
/// to be a file or a dir (if applicable).
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
