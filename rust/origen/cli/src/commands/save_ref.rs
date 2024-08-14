use std::path::Path;

use origen_metal::framework::reference_files;
use crate::commands::_prelude::*;

pub const BASE_CMD: &'static str = "save_ref";

gen_core_cmd_funcs!(
    BASE_CMD,
    "Save a reference version of the given file, this will be automatically checked for differences the next time it is generated",
    { |cmd: App<'a>| {
        cmd
            .arg(
                Arg::new("files")
                    .help("The name of the file(s) to be saved")
                    .action(SetArg)
                    .value_name("FILES")
                    .multiple(true)
                    .required_unless_one(&["new", "changed"]),
            )
            .arg(
                Arg::new("new")
                    .long("new")
                    .required(false)
                    .action(SetArgTrue)
                    .help("Update all NEW file references from the last generate run"),
            )
            .arg(
                Arg::new("changed")
                    .long("changed")
                    .required(false)
                    .action(SetArgTrue)
                    .help("Update all CHANGED file references from the last generate run"),
            )
    }}
);


pub fn run(matches: &clap::ArgMatches) -> Result<()> {
    let new = matches.contains_id("new");
    let changed = matches.contains_id("changed");
    let files = matches.get_many::<String>("files");

    if new {
        if let Err(e) = reference_files::apply_all_new_refs() {
            bail!("Something went wrong saving the NEW references - {}", e);
        }
    }

    if changed {
        if let Err(e) = reference_files::apply_all_changed_refs() {
            bail!(
                "Something went wrong updating the CHANGED references - {}",
                e
            );
        }
    }

    if let Some(files) = files {
        for key in files {
            if let Err(e) = reference_files::apply_ref(Path::new(key)) {
                bail!("Could not save '{}' - {}", key, e);
            }
        }
    }
    Ok(())
}
