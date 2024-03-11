use crate::commands::_prelude::*;
use crate::STATUS;
use origen_metal::utils::version::{Version, ReleaseType, VersionWithTOML};
use origen::utility::github::{dispatch_workflow, get_latest_workflow_dispatch, get_branch_protections, lock_branch, unlock_branch, BranchProtections};
use origen_metal::utils::terminal::confirm_with_user;
use origen_metal::utils::revision_control::RevisionControlAPI;
use std::process::exit;
use origen_metal::utils::terminal::{redln, greenln};
use std::thread;
use std::time::Duration;

pub const BASE_CMD: &'static str = "publish";

pub const USE_CURRENT: &'static str = "current";
pub const NO_RELEASE: &'static str = "none";
const RELEASE_TYPES: [&str; 6] = [NO_RELEASE, USE_CURRENT, "major", "minor", "patch", "dev"];

pub (crate) fn publish_cmd<'a>() -> SubCmd<'a> {
    let subc = core_subcmd__no_exts__no_app_opts!(
        BASE_CMD,
        "Release Origen and/or Origen Metal Rust Libraries and/or Python Packages",
        { |cmd: App| {
            // Release types/version updates
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
                Arg::new("cli_release_type")
                .long("cli_release_type")
                .visible_alias("cli")
                .value_parser(RELEASE_TYPES)
                .takes_value(true)
                .help("Specify Origen's CLI release type")
            )

            // Release targets
            .arg(
                Arg::new("release_to_pypi_test")
                .long("release_to_pypi_test")
                .alias("pypi_test")
                .action(SetArgTrue)
                .help("Release to PyPi test instance (https://test.pypi.org/)")
            )
            .arg(
                Arg::new("no_pypi_release")
                .long("no_pypi_release")
                .action(SetArgFalse)
                .help("Do NOT release to pypi, even if Origen or OM versions are updated")
            )
            .arg(
                Arg::new("github_release")
                .long("github_release")
                .visible_alias("gh_rel")
                .action(SetArgTrue)
                .help("Create a GitHub release (https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)")
            )

            .arg(
                Arg::new("no_bump_origen_om_req")
                .long("no_bump_origen_om_req")
                .visible_alias("no_bump_o_req")
                .action(SetArgTrue)
                .help("Do not bump Origen's OM requirement")
            )

            .arg(
                Arg::new("version_update_only")
                .long("version_update_only")
                .visible_alias("versions_only")
                .action(SetArgTrue)
                .help("Only updates the version files. Does not check in or launch publishing action.")
                .conflicts_with("release_to_pypi_test")
                .conflicts_with("no_pypi_release")
                .conflicts_with("github_release")
            )
        }},
        core_subcmd__no_exts__no_app_opts!(
            "monitor",
            "Monitor the most recent/running publishing workflow run",
            { |cmd: App| {
                cmd.visible_alias("m")
            }}
        )
    );
    subc
}

pub(crate) fn run(invocation: &clap::ArgMatches) -> Result<()> {
    displayln!("Retrieving most recent publish workflow run...");
    let last_pub_run = get_latest_workflow_dispatch(*super::GH_OWNER, *super::GH_REPO, Some(*super::PUBLISH_WORKFLOW))?;
    if let Some((n, _subcmd)) = invocation.subcommand() {
        match n {
            "monitor" => {
                displayln!("Most Recent Publish Workflow:");
                displayln!("  Status:     {}{}", last_pub_run.status, {
                    if let Some(c) = last_pub_run.conclusion {
                        format!(" ({})", c)
                    } else {
                        "".to_string()
                    }
                });
                displayln!("  Started On: {}", last_pub_run.run_started_at);
                displayln!("  Started By: {}", last_pub_run.triggering_actor.login);
                displayln!("  Webview:    {}", last_pub_run.html_url);
                return Ok(());
            }
            _ => unreachable_invalid_subc!(n)
        }
    }

    displayln!("Checking that no publishing actions are currently running...");
    if !last_pub_run.completed() {
        redln("A publish workflow is already running! Cannot start a new publish action.");
        redln(&format!("  Running action started by {} at {}", last_pub_run.triggering_actor.login, last_pub_run.created_at));
        redln(&format!("  {}", last_pub_run.html_url));
        exit(1);
    } else {
        greenln("No publish workflows currently running!");
    }

    // Lock master branch
    displayln!("Locking branch '{}' while publishing...", *super::PUBLISH_BRANCH);
    lock_publish_branch(true)?;

    match 'checks: {
        // Check currently on master branch with attached HEAD
        displayln!("Checking workspace is on the master branch with attached HEAD...");
        let git = origen_metal::utils::revision_control::RevisionControl::git(
            &STATUS.origen_wksp_root,
            vec!("git@github.com:Origen-SDK/o2.git"),
            None
        );
        if !git.on_branch("master")? {
            redln("Publishing must be done on the master branch!");
            break 'checks Ok(false)
        } else {
            greenln("On master branch with attached HEAD");
        }

        // TODO PublishO2
        // Ensure no local changes
        let status = git.status(None)?;
        if status.is_modified() {
            status.summarize();
            redln("Changes found in workspace. Please check in your changes or stash them before rerunning");
            break 'checks Ok(false);
        }

        // Ensure up-to-date with remote
        git.fetch(None)?;
        let latest = git.confirm_latest_ref(*super::PUBLISH_BRANCH)?;
        if !latest.0 {
            redln(&format!("Current ref does not match latest ref for master branch ({} vs. {})", latest.1[0], latest.1[1]));
            break 'checks Ok(false);
        }

        // TODO PublishO2 Ensure a regression test passed with this commit

        // Get current versions
        // TODO PublishO2 Cleanup - move paths to shared location
        let om_pyproject_path = STATUS.origen_wksp_root.join("python").join("origen_metal").join("pyproject.toml");
        let origen_pyproject_path = STATUS.origen_wksp_root.join("python").join("origen").join("pyproject.toml");
        let cli_toml_loc = STATUS.origen_wksp_root.join("rust").join("origen").join("cli").join("cargo.toml");
        let mut py_om_ver = Version::from_pyproject_with_toml_handle(om_pyproject_path)?;
        let mut py_origen_ver = Version::from_pyproject_with_toml_handle(origen_pyproject_path)?;
        let mut cli_ver = Version::from_cargo_with_toml_handle(cli_toml_loc)?;

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
        let release_cli = extract_release(invocation, "cli_release_type", &mut cli_ver)?;

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
        let update_cli = summarize_release("Origen CLI", release_cli, &cli_ver);
        let new_req = {
            if update_origen_package && !*invocation.get_one::<bool>("no_bump_origen_om_req").unwrap() {
                Some(format!("~{}",py_om_ver.version().to_string()))
            } else {
                None
            }
        };
        if let Some(new) = new_req.as_ref() {
            let old_req = py_origen_ver.get_other(&*super::ORIGEN_OM_REQ_PATH)?.to_string();
            displayln!("Origen's OM requirement: Updating to '{}' (from '{}')", new, old_req);
        } else {
            displayln!("Origen's OM requirement: No update to Origen's OM minimum version");
        }
        if !confirm_with_user(Some("Proceed with release?"))? {
            displayln!("Exiting without sending release request...");
            break 'checks Ok(false);
        }

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
        if let Some(new) = new_req.as_ref() {
            py_origen_ver.set_other(&*super::ORIGEN_OM_REQ_PATH, new)?;
        }
        update_toml(update_origen_package, &mut py_origen_ver)?;

        if *invocation.get_one::<bool>("version_update_only").unwrap() {
            displayln!("Stopping release after version update...");
            break 'checks Ok(false);
        } else {
            let mut to_checkin = vec!();
            if update_om_package {
                displayln!("Checking in OM version at: '{}'", py_om_ver.source().display());
                to_checkin.push(py_om_ver.source().as_path());
            }
            if update_origen_package {
                displayln!("Checking in OM version at: '{}'", py_origen_ver.source().display());
                to_checkin.push(py_origen_ver.source().as_path());
            }
            if update_cli {
                displayln!("Checking in OM version at: '{}'", cli_ver.source().display());
                to_checkin.push(cli_ver.source().as_path());
            }
            if to_checkin.is_empty() {
                displayln!("No version files to check in...");
            } else {
                displayln!("Committing and pushing new version files...");
                unlock_publish_branch(true)?;
                git.checkin(Some(to_checkin), "Updated version files for next release", false)?;
                lock_publish_branch(true)?;
            }
        }

        // Send Github actions request to build and release
        let mut inputs = indexmap::IndexMap::new();
        inputs.insert("origen_metal_python_package", update_om_package.to_string());
        inputs.insert("origen_python_package", update_origen_package.to_string());
        inputs.insert("publish_pypi", (!invocation.get_one::<bool>("no_pypi_release").unwrap() && (update_origen_package || update_om_package)).to_string());
        inputs.insert("publish_pypi_test", invocation.get_one::<bool>("release_to_pypi_test").unwrap().to_string());
        inputs.insert("publish_github_release", invocation.get_one::<bool>("github_release").unwrap().to_string());
        displayln!("Sending request to GitHub Actions to build and release...");
        for (k, v) in &inputs {
            displayln!("  {}: {}", k, v);
        }
        dispatch_workflow(*super::GH_OWNER, *super::GH_REPO, *super::PUBLISH_WORKFLOW, *super::PUBLISH_BRANCH, Some(inputs))?;

        Ok::<bool, origen_metal::Error>(true)
    } {
        Ok(publishing) => {
            redln("Automation is incomplete");

            // Unlock branch
            unlock_publish_branch(true)?;
            if publishing {
                // Publishing started. Try to find the job and provide the link
                displayln!("Publishing workflow requested. Gathering link...");
                'wait_for_publish_run: {
                    for i in 0..10 {
                        let pub_run = get_latest_workflow_dispatch(*super::GH_OWNER, *super::GH_REPO, Some(*super::PUBLISH_WORKFLOW))?;
                        if last_pub_run.id != pub_run.id {
                            displayln!("Publishing workflow started. Use 'origen develop_origen publish monitor' or this link to monitor the progression:\n{}", &pub_run.html_url);
                            break 'wait_for_publish_run;
                        }
                        displayln!("Found same job ID as previous run. Retrying in 5 seconds... ({} of 10)", i+1);
                        thread::sleep(Duration::from_secs(5));
                    }
                    redln("Did not find an updated publish run. Please check the workflow run manually.");
                }
            } else {
                displayln!("No publishing workflow initiated");
            }
            Ok(())
        },
        Err(e) => {
            // Log the error, unlock the branch, and return the error
            redln("Encountered an error during publishing checks:");
            redln(&format!("{}", e));
            unlock_publish_branch(true)?;
            bail!("{}", e);
        }
    }
}

fn unlock_publish_branch(log_errors: bool) -> Result<BranchProtections> {
    displayln!("Unlocking branch '{}'...", *super::PUBLISH_BRANCH);
    match unlock_branch(*super::GH_OWNER, *super::GH_REPO, *super::PUBLISH_BRANCH) {
        Ok(res) => {
            greenln("Branch unlocked!");
            Ok(res)
        },
        Err(e) => {
            if log_errors {
                redln("Error unlocking branch '{}'. Branch must be manually unlocked. Received Error:");
                redln(&format!("{}", e));
            }
            Err(e)
        }
    }
}

fn lock_publish_branch(log_errors: bool) -> Result<BranchProtections> {
    match get_branch_protections(*super::GH_OWNER, *super::GH_REPO, *super::PUBLISH_BRANCH) {
        Ok(res) => {
            if res.is_locked() {
                let m = format!("Branch '{}' is unexpectedly locked!", *super::PUBLISH_BRANCH);
                redln(&m);
                bail!(&m);
            }
            let retn = lock_branch(*super::GH_OWNER, *super::GH_REPO, *super::PUBLISH_BRANCH)?;
            greenln("Branch locked!");
            Ok(retn)
        },
        Err(e) => {
            if log_errors {
                redln(&format!("Error locking branch '{}':", *super::PUBLISH_BRANCH));
                redln(&format!("{}", e));
            }
            Err(e)
        }
    }
}
