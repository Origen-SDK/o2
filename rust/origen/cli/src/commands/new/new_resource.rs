use super::PY_BLOCK;
use clap::ArgMatches;
use regex::Regex;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

pub fn run(matches: &ArgMatches) {
    match matches.subcommand_name() {
        Some("dut") => {
            let name = matches
                .subcommand_matches("dut")
                .unwrap()
                .value_of("name")
                .unwrap()
                .to_string();

            let mut name = clean_and_validate_resource_name(&name, "NAME");

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
                    let _ = std::fs::create_dir_all(target_file.parent().unwrap());
                }
                std::fs::write(&target_file, &content)
                    .expect(&format!("Couldn't create '{}'", &target_file.display()));
            }
        }
        Some("block") => {
            let matches = matches.subcommand_matches("block").unwrap();
            let name = matches.value_of("name").unwrap().to_string();
            let mut nested = false;
            let parent = matches.value_of("parent");
            let mut block_name = clean_and_validate_resource_name(&name, "NAME");

            if let Some(p) = parent {
                nested = true;
                if name.contains("/") {
                    display_red!("ERROR: ");
                    displayln!("The NAME '{}' is invalid, when specifying a PARENT argument the NAME cannot also contain a leading parent name(s)",  name);
                    std::process::exit(1);
                }
                let par = clean_and_validate_resource_name(p, "PARENT");
                block_name = format!("{}/{}", &par, &block_name);
            }

            let mut top = true;
            let mut path = origen::app().unwrap().app_dir().join("blocks");
            let mut i = 0;
            let names = block_name.split("/").collect::<Vec<&str>>();

            for n in &names {
                if !top {
                    if nested && i == names.len() - 1 {
                        path = path.join("sub_blocks");
                    } else {
                        path = path.join("derivatives");
                    }
                }
                path = path.join(n);

                generate_block(&path, top, nested && i == names.len() - 1);
                top = false;
                i += 1;
            }
        }
        None => unreachable!(),
        _ => unreachable!(),
    }
}

fn generate_dut(dir: &Path, top: bool) {
    let mut context = Context::new();

    context.insert("top", &top);
    context.insert("dut", &true);

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

fn generate_block(dir: &Path, top: bool, nested: bool) {
    let mut context = Context::new();

    context.insert("top", &top);
    context.insert("nested", &nested);

    if !dir.exists() {
        std::fs::create_dir_all(dir).expect(&format!("Couldn't create '{}'", dir.display()));
        write_block_file(dir, &context, "controller.py");
        write_block_file(dir, &context, "registers.py");
        write_block_file(dir, &context, "sub_blocks.py");
        if !nested {
            write_block_file(dir, &context, "attributes.py");
            write_block_file(dir, &context, "levels.py");
            write_block_file(dir, &context, "services.py");
            write_block_file(dir, &context, "timing.py");
        }
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
///
/// The following mods will be make to the returned value:
///   * any '\' will be replace with '/'
///   * 'derivatives/' or 'sub_blocks' in the name will be removed
///   * if it leads with the app name or 'blocks' then it will be removed
fn clean_and_validate_resource_name(name: &str, resource_id: &str) -> String {
    let contains_special_chars = Regex::new(r"[^0-9a-z_]").unwrap();
    let starts_with_number = Regex::new(r"^[0-9]").unwrap();

    let name = name.replace(r#"\"#, "/");
    let mut names: Vec<&str> = vec![];

    for n in name.split('/') {
        if contains_special_chars.is_match(n) || starts_with_number.is_match(n) {
            display_red!("ERROR: ");
            displayln!("The {} '{}' is invalid, all resource names must be lowercased, underscored, start with a letter and contain no special characters", resource_id, name);
            std::process::exit(1);
        }
        if n != origen::app().unwrap().name()
            && n != "blocks"
            && n != "sub_blocks"
            && n != "derivatives"
        {
            names.push(n);
        }
    }
    names.join("/")
}

/// Returns a path to the block directory for the given resource name
fn block_dir(name: &str) -> PathBuf {
    let mut path = origen::app().unwrap().app_dir().join("blocks");
    let mut top = false;

    for n in name.split("/") {
        if !top {
            path = path.join("derivatives");
        }
        path = path.join(n);
        top = false;
    }
    path
}
