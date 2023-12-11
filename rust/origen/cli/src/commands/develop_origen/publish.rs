use crate::commands::_prelude::*;
use crate::STATUS;
use origen::utility::version::{Version, ReleaseType, VersionWithTOML};
use origen::utility::github::{dispatch_workflow, get_latest_workflow_dispatch};
use origen_metal::utils::terminal::confirm_or_exit;
use origen_metal::utils::revision_control::RevisionControlAPI;

pub const BASE_CMD: &'static str = "publish";

pub const USE_CURRENT: &'static str = "current";
pub const NO_RELEASE: &'static str = "none";
const RELEASE_TYPES: [&str; 6] = [NO_RELEASE, USE_CURRENT, "major", "minor", "patch", "dev"];

pub (crate) fn publish_cmd<'a>() -> SubCmd<'a> {
    core_subcmd__no_exts__no_app_opts!(
        BASE_CMD,
        "Release Origen and/or Origen Metal Rust Libraries and/or Python Packages",
        { |cmd: App| {
            cmd.arg(
                Arg::new("origen_release_type")
                .long("origen_release_type")
                .visible_alias("origen")
                .value_parser(RELEASE_TYPES)
                .takes_value(true)
                .help("Specify Origen Python Package's release type")
            )
            .arg(
                Arg::new("om_release_type")
                .long("origen_metal_release_type")
                .visible_alias("om")
                .visible_alias("metal")
                .value_parser(RELEASE_TYPES)
                .takes_value(true)
                .help("Specify Origen Metal Python Package's release type")
            )
            .arg(
                Arg::new("test_pypi")
                .long("pypi_test_server")
                .action(SetArgTrue)
                .help("Release to PyPi test instance (https://test.pypi.org/)")
            )
            .arg(
                Arg::new("github_release")
                .long("github_release")
                .visible_alias("gh_rel")
                .action(SetArgTrue)
                .help("Create a GitHub release (https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)")
            )
            .arg(
                Arg::new("bump_origen_om_req")
                .long("bump_origen_om_req")
                .visible_alias("bump_o_req")
                .action(SetArgTrue)
                .help("Set Origen's OM requirement to this OM version")
            )
        }}
    )
}

use std::process::exit;
use origen_metal::utils::terminal::redln;
pub(crate) fn run(invocation: &clap::ArgMatches) -> Result<()> {
    // TODO PublishO2
    // Make sure no other publishing actions are running

    // TODO PublishO2
    // Lock master branch
    // Need to wrap below steps to ensure master will be unlocked even if there's a problem with another check

    // Check currently on master branch with attached HEAD
    let git = origen_metal::utils::revision_control::RevisionControl::git(&STATUS.origen_wksp_root, vec!("git@github.com:Origen-SDK/o2.git"), None);
    if !git.on_branch("master")? {
        redln("Publishing must be done on the master branch!");
        exit(1);
    }

    // TODO PublishO2
    // Ensure no local changes
    // let status = git.status(None)?;
    // if status.is_modified() {
    //     status.summarize();
    //     redln("Changes found in workspace. Please check in your changes or stash them before rerunning");
    //     exit(1);
    // }

    // TODO PublishO2
    // Ensure up-to-date with remote
    // Currently issues with git2-rs - cannot authenticate
    // git.fetch(None)?;
    // git.list_refs(None)?;

    // TODO PublishO2 Ensure a regression test passed with this commit

    // Get current versions
    // TODO PublishO2 Cleanup - move paths to shared location
    let om_pyproject_path = STATUS.origen_wksp_root.join("python").join("origen_metal").join("pyproject.toml");
    let origen_pyproject_path = STATUS.origen_wksp_root.join("python").join("origen").join("pyproject.toml");
    let mut py_om_ver = Version::from_pyproject_with_toml_handle(om_pyproject_path)?;
    let mut py_origen_ver = Version::from_pyproject_with_toml_handle(origen_pyproject_path)?;

    // Extract release types
    fn extract_release(invoc: &clap::ArgMatches, cli_name: &str, ver: &mut VersionWithTOML) -> Result<bool> {
        if let Some(rel_type) = invoc.get_one::<String>(cli_name) {
            match rel_type.as_str() {
                USE_CURRENT => Ok(true),
                NO_RELEASE => Ok(false),
                _ => {
                    ver.increment(ReleaseType::try_from(rel_type.as_str())?)?;
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }
    let release_py_om = extract_release(invocation, "om_release_type", &mut py_om_ver)?;
    let release_py_origen = extract_release(invocation, "origen_release_type", &mut py_origen_ver)?;

    // Bump releases if needed and summarize to user
    fn summarize_release(name: &str, release: bool, ver: &VersionWithTOML) -> bool {
        if release {
            if ver.was_version_updated() {
                displayln!(
                    "{}: Releasing '{}' Version: {} (from {})",
                    name,
                    ver.rel_type().expect("Expected version should have been updated"),
                    ver.version(),
                    ver.orig_version()
                );
                true
            } else {
                displayln!("{}: Releasing Current Version: {}", ver.version(), name);
                false
            }
        } else {
            displayln!("{}: No release", name);
            false
        }
    }
    displayln!("Release Summary:");
    let update_om_package = summarize_release("Origen Metal Python Package", release_py_om, &py_om_ver);
    let update_origen_package = summarize_release("Origen Python Package", release_py_origen, &py_origen_ver);
    confirm_or_exit(Some("Proceed with release?"), Some("Exiting without sending release request!"), None)?;

    // Update the TOMLs
    // Make sure all versions updated successfully before any checking in
    fn update_toml(should_update: bool, ver: &mut VersionWithTOML) -> Result<()> {
        if should_update {
            ver.write()?;
            displayln!("Updated version in {}", ver.source().display());
        }
        Ok(())
    }
    displayln!("Updating versions in TOML files...");
    update_toml(update_om_package, &mut py_om_ver)?;
    if *invocation.get_one::<bool>("bump_origen_om_req").unwrap() {
        // TODO PublishingO2 this should be automatically set on an OM + Origen release
        let prev_ver = py_origen_ver.toml["tool"]["poetry"]["dependencies"]["origen_metal"].to_string();
        displayln!("Bumping Origen's 'origen_metal' requirement from {} to {}", prev_ver, py_om_ver.version());
        py_origen_ver.toml["tool"]["poetry"]["dependencies"]["origen_metal"] = origen_metal::toml_edit::value(py_om_ver.version().to_string());
    }
    update_toml(update_origen_package, &mut py_origen_ver)?;

    // TODO PublishO2 Check in updated files
    // if update_om_package {
    //     // git.checkin()?;
    // }
    // if update_origen_package {
    //     // git.checkin()?;
    // }
    // For now, if updating occurred, user must manually push updates and re-run publishing with updated master
    // let status = git.status(None)?;
    // if status.is_modified() {
    //     status.summarize();
    //     redln("Automation is incomplete. Please check in updated files and rerun using 'current' package versions");
    //     exit(1);
    // }

    // TODO PublishO2 Cancel regressions if a push occurred
    // If a push to master occurred, cancel the run:
    //   Will likely fail if libraries/packages were updated with to-be-released versions
    //   May delay the start of the publishing action

    // TODO PublishO2 Send Github actions request to build and release
    // displayln!("Sending request to GitHub Actions to build and release the following:");
    // let inputs = std::collections::HashMap::new();
    // let res = dispatch_workflow("Origen-SDK", "o2", "publish.yml", "master", Some(inputs))?;
    // let res = get_latest_workflow_dispatch("Origen-SDK", "o2", Some("publish.yml"))?;
    // res.cancel()?;
    // TODO PublishingO2 debug stuff to remove
    // let res = origen::utility::github::get_workflow_run_by_id("Origen-SDK", "o2", 7041427651)?;
    // println!("Result: {:?}", res);
    // let res = get_latest_workflow_dispatch("Origen-SDK", "o2", Some("publish.yml"))?;
    // println!("Result: {:?}", res);
    // res.refresh()?;
    redln("Automation is incomplete");

    Ok(())
}