use origen_metal::framework::reference_files;
use std::path::Path;

pub fn run(matches: &clap::ArgMatches) {
    let mut exit_code = 0;

    let new = matches.is_present("new");
    let changed = matches.is_present("changed");
    let files = matches.values_of("files");

    if new {
        if let Err(e) = reference_files::apply_all_new_refs() {
            log_error!("Something went wrong saving the NEW references - {}", e);
            exit_code = 1;
        }
    }

    if changed {
        if let Err(e) = reference_files::apply_all_changed_refs() {
            log_error!(
                "Something went wrong updating the CHANGED references - {}",
                e
            );
            exit_code = 1;
        }
    }

    if let Some(files) = files {
        for key in files {
            if let Err(e) = reference_files::apply_ref(Path::new(key)) {
                log_error!("Could not save '{}' - {}", key, e);
                exit_code = 1;
            }
        }
    }
    std::process::exit(exit_code);
}
