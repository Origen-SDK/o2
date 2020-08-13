use super::PY_BLOCK;
use clap::ArgMatches;
use regex::Regex;
use std::path::Path;
use tera::{Context, Tera};

pub fn run(matches: &ArgMatches) {
    match matches.subcommand_name() {
        Some("dut") => {
            let mut name = matches
                .subcommand_matches("dut")
                .unwrap()
                .value_of("name")
                .unwrap()
                .to_string();

            validate_resource_name(&name, "NAME");

            // Add the leading 'dut' to the fully qualified new DUT name if missing
            if !name.starts_with("dut/") {
                name = format!("dut/{}", &name);
            }

            let mut top = true;
            let mut path = origen::app().unwrap().app_dir().join("blocks");

            for n in name.split("/") {
                if !top {
                    path = path.join("derivatives");
                }
                path = path.join(n);

                generate_dut(&path, top);
                top = false;
            }

            // Create a target file for the new DUT
            let last_name = format!("{}.py", name.split("/").last().unwrap());
            let target_file = origen::app()
                .unwrap()
                .root
                .join("targets")
                .join("dut")
                .join(last_name);
            let content = format!("origen.app.instantiate_dut(\"{}\")", name.replace("/", "."));
            if !target_file.exists() {
                display_green!("      create  ");
                displayln!(
                    "{}",
                    target_file
                        .strip_prefix(&origen::app().unwrap().root)
                        .unwrap()
                        .display()
                );
                if !target_file.parent().unwrap().exists() {
                    std::fs::create_dir_all(target_file.parent().unwrap());
                }
                std::fs::write(&target_file, &content)
                    .expect(&format!("Couldn't create '{}'", &target_file.display()));
            }
        }
        None => unreachable!(),
        _ => unreachable!(),
    }
}

fn generate_dut(dir: &Path, top: bool) {
    let mut context = Context::new();

    context.insert("top", &top);

    if !dir.exists() {
        std::fs::create_dir_all(dir).expect(&format!("Couldn't create '{}'", dir.display()));
        write_block_file(dir, &context, "attributes.py");
        write_block_file(dir, &context, "controller.py");
        write_block_file(dir, &context, "levels.py");
        write_block_file(dir, &context, "pins.py");
        write_block_file(dir, &context, "registers.py");
        write_block_file(dir, &context, "services.py");
        write_block_file(dir, &context, "sub_blocks.py");
        write_block_file(dir, &context, "timing.py");
    } else {
        // Need to do anything here, should we check and build a controller if missing?
    }
}

fn write_block_file(dir: &Path, context: &Context, file_name: &str) {
    let full_path = dir.join(file_name);
    let local_path = full_path
        .strip_prefix(&origen::app().unwrap().root)
        .unwrap();
    display_green!("      create  ");
    displayln!("{}", local_path.display());
    let mut tera = Tera::default();
    let contents = tera.render_str(&PY_BLOCK[file_name], &context).unwrap();
    std::fs::write(&full_path, &contents)
        .expect(&format!("Couldn't create '{}'", &full_path.display()));
}

/// Validate that the given name meets the following criteria:
///   * lowercased
///   * underscored
///   * starts with a letter
///   * doesn't contain any special characters
///
/// If not an error message will be output to the console and the command will terminate
fn validate_resource_name(name: &str, resource_id: &str) {
    let contains_special_chars = Regex::new(r"[^0-9a-z_]").unwrap();
    let starts_with_number = Regex::new(r"^[0-9]").unwrap();

    for n in name.split('/') {
        if contains_special_chars.is_match(n) || starts_with_number.is_match(n) {
            display_red!("ERROR: ");
            displayln!("The {} '{}' is invalid, all resource names must be lowercased, underscored, start with a letter and contain no special characters", resource_id, name);
            std::process::exit(1);
        }
    }
}
