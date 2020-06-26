use std::path::{Path, PathBuf};
use std::env;
use std::process::exit;
use origen::revision_control::RevisionControl;
use origen::revision_control::RevisionControlAPI;
use origen::utility::file_utils::to_relative_path;

pub fn run(matches: &clap::ArgMatches) {
    match matches.subcommand_name() {
        Some("status") => {
            let matches = matches.subcommand_matches("status").unwrap();
            let path = path(matches);
            let rc = match path.is_file() {
                false => rc(&path),
                true => rc(&path.parent().unwrap()),
            };
            let stat = rc.status(Some(&path)).expect("Something went wrong determining the status, see log for details");
            let pwd = std::env::current_dir().expect("Couldn't resolve the PWD");
            if stat.is_modified() {
                displayln!("The workspace contains the following modifications:");
                displayln!("");
                if !stat.added.is_empty() {
                    display_cyanln!("        new:");
                    for f in &stat.added {
                        display_cyanln!("                    {}", to_relative_path(f, Some(&pwd)).unwrap_or_else(|_| f.to_path_buf()).display());
                    }
                    displayln!("");
                }
                if !stat.removed.is_empty() {
                    display_redln!("        deleted:");
                    for f in &stat.removed {
                        display_redln!("                    {}", to_relative_path(f, Some(&pwd)).unwrap_or_else(|_| f.to_path_buf()).display());
                    }
                    displayln!("");
                }
                if !stat.changed.is_empty() {
                    display_yellowln!("        modified:");
                    for f in &stat.changed {
                        display_yellowln!("                    origen rc diff {}", to_relative_path(f, Some(&pwd)).unwrap_or_else(|_| f.to_path_buf()).display());
                    }
                    displayln!("");
                }
            } else {
                display_greenln!("Clean");
            }
        }
        None | _ => unreachable!(),
    }
}

fn rc(dir: &Path) -> RevisionControl {
    match RevisionControl::from_dir(dir) {
        Some(rc) => rc,
        None => {
            log_error!("No supported revision control tool/system found, check log for errors if you believe this should have worked");
            exit(1);
        }
    }
}

/// Returns the path contained in the args or else the PWD
fn path(matches: &clap::ArgMatches) -> PathBuf {
    match matches.value_of("path") {
        Some(x) => PathBuf::from(x).canonicalize().expect(&format!("PATH arg is invalid '{}'", x)),
        None => match env::current_dir() {
            Err(e) => {
                log_error!("{}", e);
                log_error!("Something has gone wrong trying to resolve the PWD, is it stale or do you not have read access to it?");
                exit(1);
            }
            Ok(d) => d,
        }
    }
}
