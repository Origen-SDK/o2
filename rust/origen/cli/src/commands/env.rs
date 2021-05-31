//! Some notes on how Origen's Python environment is setup and invoked:
//!

extern crate time;

use crate::python::{poetry_version, MIN_PYTHON_VERSION, PYTHON_CONFIG};
use clap::ArgMatches;
use online::online;
use origen::core::status::search_for;
use origen::core::term::*;
use origen::utility::file_actions as fa;
use regex::Regex;
use semver::VersionReq;
use std::process::Command;

pub fn run(matches: &ArgMatches) {
    match matches.subcommand_name() {
        Some("update") => {
            install_poetry();
            let _ = PYTHON_CONFIG.poetry_command().arg("update").status();

            // Don't think we need to do anything here, if something goes wrong Poetry will give a better
            // error message than we could
        }

        Some("setup") => {
            let mut origen_source_changed = false;
            let mut run_origen_build = false;

            let origen_root = match matches
                .subcommand_matches("setup")
                .unwrap()
                .value_of("origen")
            {
                None => None,
                Some(x) => {
                    let path = std::path::Path::new(x)
                        .canonicalize()
                        .expect("That path to Origen doesn't exist");
                    let (found, path) = search_for(vec![".origen_dev_workspace"], false, &path);
                    if found {
                        Some(path)
                    } else {
                        log_error!("Origen was not found at that path");
                        std::process::exit(1);
                    }
                }
            };

            print!("Is a suitable Python available? ... ");
            if PYTHON_CONFIG.available {
                greenln("YES");
            } else {
                redln("NO");
                println!("");
                println!(
                    "Could not find Python > {} available, please install this and try again.",
                    MIN_PYTHON_VERSION
                );
                std::process::exit(1);
            }

            print!("Is the internet accessible?     ... ");
            if online(None).is_ok() {
                greenln("YES");
            } else {
                redln("NO");
                println!("");
                println!("In future, Origen would now talk you through what files to get and where to put them, but that's not implemented yet, sorry!");
                std::process::exit(1);
            }

            install_poetry();

            let app_root = &origen::app().unwrap().root;
            let pyproject = app_root.join("pyproject.toml");
            if !pyproject.exists() {
                display_red!("ERROR: ");
                displayln!("application pyproject file not found!");
                std::process::exit(1);
            }

            if let Some(p) = origen_root {
                let origen_root = p.join("python");

                // Poetry seems to have a number of bugs when switching back and forth between path and version
                // references, this step ensures it comes up correctly, but should be removed in future
                delete_virtual_env();

                // Comment out the current reference to Origen
                let r = Regex::new(r"^\s*origen ?=").unwrap();
                if let Err(e) = fa::insert_before(&pyproject, &r, "#") {
                    display_redln!("{}", e);
                    std::process::exit(1);
                };

                // And add a new one pointing to the given path
                let r = Regex::new(r"^\s*\[\s*tool.poetry.dependencies\s*\].*").unwrap();
                let line = format!("\norigen = {{ path = \"{}\" }}", origen_root.display());
                if let Err(e) = fa::insert_after(&pyproject, &r, &line) {
                    display_redln!("{}", e);
                    std::process::exit(1);
                };

                // Make sure Rust nightly is enabled in the app dir, just do this quietly if it succeeds
                match origen::utility::command_helpers::exec_and_capture(
                    "rustup",
                    Some(vec!["override", "set", "nightly"]),
                ) {
                    Err(e) => log_error!("{}", e),
                    Ok((code, stdout, stderr)) => {
                        if !code.success() {
                            for line in stdout {
                                displayln!("{}", line);
                            }
                            for line in stderr {
                                display_redln!("{}", line);
                            }
                        }
                    }
                }

                origen_source_changed = true;
                run_origen_build = true;

            // We want to keep the Origen development apps permanently running on a local reference to Origen
            } else if !origen::STATUS.is_origen_present {
                // If we are about to switch from a path to a version reference then delete the virtual env to ensure the
                // switch happens cleanly, this is for a Poetry bug and should be removed in future
                if origen::STATUS.is_app_in_origen_dev_mode {
                    delete_virtual_env();
                }

                // Remove any path references to Origen
                let r = Regex::new(r"origen\s*=\s*\{\s*path\s*=").unwrap();
                if let Err(e) = fa::remove_line_all(&pyproject, &r) {
                    display_redln!("{}", e);
                    std::process::exit(1);
                };

                // Un-comment any version reference and if there is none then add a new one
                // pointing to the latest Origen version
                let r = Regex::new(r"^\s*origen\s*=").unwrap();
                if !(match fa::contains(&pyproject, &r) {
                    Ok(result) => result,
                    Err(e) => {
                        display_redln!("{}", e);
                        std::process::exit(1);
                    }
                }) {
                    let r = Regex::new(r"^#+\s*origen\s*=").unwrap();
                    match fa::replace(&pyproject, &r, "origen =") {
                        // If pyproject.toml does not contain any reference to origen then add it
                        Ok(replaced) => {
                            if !replaced {
                                let r = Regex::new(r"^\s*\[\s*tool.poetry.dependencies\s*\].*")
                                    .unwrap();
                                let line =
                                    format!("\norigen = \"{}\"", origen::STATUS.origen_version);
                                if let Err(e) = fa::insert_after(&pyproject, &r, &line) {
                                    display_redln!("{}", e);
                                    std::process::exit(1);
                                };
                            }
                        }
                        Err(e) => {
                            display_redln!("{}", e);
                            std::process::exit(1);
                        }
                    }
                    origen_source_changed = true;
                }
            }

            // Lower than this version has a bug which can crash with local path dependencies
            print!("Is PIP version >= 19.1?         ... ");

            // This gives an error that suggests it is not working when run on Windows, yet it solves the problem
            // seen with the earlier version on CI
            let args = vec!["run", "pip", "install", "pip==20.2.3"];
            let status = PYTHON_CONFIG.poetry_command().args(&args).status();

            if status.is_ok() {
                greenln("YES");
            } else {
                redln("NO");
            }

            print!("Are the app's deps. installed?  ... ");

            let status = PYTHON_CONFIG
                .poetry_command()
                .arg("install")
                .arg("--no-root")
                .status();

            if status.is_ok() {
                if origen_source_changed {
                    std::env::set_current_dir(&origen::app().unwrap().root)
                        .expect("Couldn't cd to the app root");

                    let status = PYTHON_CONFIG
                        .poetry_command()
                        .arg("update")
                        .arg("origen")
                        .status();
                    if status.is_ok() {
                        greenln("YES");
                        if run_origen_build {
                            println!("Building origen...");
                            let _ = Command::new("origen").arg("build").status();
                        }
                        std::process::exit(0);
                    } else {
                        redln("NO");
                        std::process::exit(1);
                    }
                } else {
                    greenln("YES");
                    std::process::exit(0);
                }
            } else {
                redln("NO");
                std::process::exit(1);
            }
        }
        None => unreachable!(),
        _ => unreachable!(),
    }
}

fn install_poetry() {
    let mut attempts = 0;
    while attempts < 3 {
        print!("Is a suitable Poetry available? ... ");
        let version = poetry_version();
        let required_poetry_version = VersionReq::parse("=1.1.6").unwrap();

        if version.is_some() && required_poetry_version.matches(&version.unwrap()) {
            greenln("YES");
            attempts = 3;
        } else {
            redln("NO");
            println!("");
            attempts = attempts + 1;

            if attempts == 3 {
                display_redln!("Failed to install Poetry, run again with -vvv to see what happened")
            } else {
                let mut c = Command::new(&PYTHON_CONFIG.command);
                c.arg("-m");
                c.arg("pip");
                c.arg("install");
                if attempts == 2 {
                    c.arg("--user");
                    displayln!(
                        "Trying again to install Poetry in user dir, please wait a few moments"
                    )
                } else {
                    displayln!("Installing Poetry, please wait a few moments")
                }
                c.arg("--ignore-installed");
                c.arg("poetry==1.1.6");
                match c.output() {
                    Ok(output) => {
                        let text = std::str::from_utf8(&output.stdout).unwrap();
                        log_trace!("{}", text);
                    }
                    Err(e) => log_debug!("{}", e),
                }
            }

            println!("");
        }
    }
}

fn delete_virtual_env() {
    log_trace!("Deleting Python virtual environment");
    if let Ok(path) = crate::python::virtual_env() {
        log_trace!("Path to virtual env found: '{}'", path.display());
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}
