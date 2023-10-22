use crate::commands::_prelude::*;
use origen_metal::tera::{Context, Tera};
use std::process::exit;
use std::env;
use std::fs::{create_dir, File};
use std::path::PathBuf;
use crate::_generated::python::PYTHONS;

pub const BASE_CMD: &'static str = "new";
pub const WS_CMD: &'static str = "workspace";
pub const APP_CMD: &'static str = "application";

use phf::phf_map;

// This includes a map of all template files, it is built by cli/build.rs at compile time.
// All files in each sub-directory of commands/new/templates are accessible via a map named after the
// uppercased sub_directory, e.g.
//      PYTHON_APP = { "pyproject.toml" => "[tool.poetry]...", "config/application.toml" => "..." }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new app.
include!(concat!(env!("OUT_DIR"), "/new_app_templates.rs"));

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Create a new origen environment (e.g., app, workspace)",
    { |cmd: App<'a>| {
        cmd.arg_required_else_help(true)
    }},
    core_subcmd__no_exts__no_app_opts!(WS_CMD, "Create a new workspace", { |cmd: App| {
        cmd.visible_alias("ws")
            .arg(req_sv_arg!("name", "NAME", "Workspace name"))
            .arg(sv_opt!("desc", "DESC", "Description of the workspace").visible_alias("description"))
            .arg(sv_opt!("path", "PATH", "Path to build the new workspace").short('p'))
    }})
    // TODO origen new - support new app
    // core_subcmd__no_exts__no_app_opts!(APP_CMD, "Create a new application", { |cmd: App| {
    //     cmd.visible_alias("app")
    // }})
);

pub fn run(invocation: &clap::ArgMatches) -> origen::Result<()> {
    if let Some((n, subcmd)) = invocation.subcommand() {
        match n {
            WS_CMD => {
                let mut tera = match Tera::new("templates/workspace/*.tera") {
                    Ok(t) => t,
                    Err(e) => {
                        println!("Failed to parse workspace templates: {}", e);
                        exit(1);
                    }
                };
                let mut context = Context::new();
                let name = subcmd.get_one::<String>("name").unwrap();

                let mut out_dir;
                if let Some(p) = subcmd.get_one::<PathBuf>("path") {
                    if p.is_relative() {
                        out_dir = env::current_dir()?;
                        out_dir.push(p);
                    } else {
                        out_dir = p.to_path_buf();
                    }
                } else {
                    out_dir = env::current_dir()?;
                    out_dir.push(&name);
                }

                if out_dir.exists() {
                    // Check directory is empty
                    if !out_dir.read_dir()?.next().is_none() {
                        log_error!("Target directory {} is not empty!", &out_dir.display());
                        exit(1);
                    }
                } else {
                    create_dir(&out_dir)?;
                }
                println!("Creating new workspace at {}", &out_dir.display());

                context.insert("name", name);
                context.insert("desc", subcmd.get_one::<String>("desc").unwrap_or(&"".to_string()));
                context.insert("app_gen", &false);

                let users = origen_metal::users();
                let mut author = "".to_string();
                context.insert("python_version", &format!(
                    ">={},<={}",
                    PYTHONS[2].strip_prefix("python").unwrap(),
                    PYTHONS.last().unwrap().strip_prefix("python").unwrap()
                ));
                if let Ok(u) = users.current_user() {
                    match u.username() {
                        Ok(username) => {
                            match u.get_email() {
                                Ok(e) => {
                                    if let Some(email) = e {
                                        author += &format!("{} <{}>", &username, &email);
                                    }
                                }
                                Err(e) => {
                                    log_warning!("Cannot retrieve current user's email: {}", e.msg);
                                }
                            }
                        },
                        Err(e) => {
                            log_warning!("Cannot retrieve current user: {}", e.msg);
                        }
                    }
                } else {
                    log_warning!("Cannot populate current user");
                }
                context.insert("author", &author);
                context.insert("origen_version", &origen::STATUS.origen_version.to_string());
                // TODO origen new - find a better way than hard-coding pytest version
                context.insert("pytest_version", "^7");

                for (n, contents) in SHARED.entries() {
                    tera.add_raw_template(&format!("shared/{}", n), contents)?;
                }
                for (n, contents) in WORKSPACE.entries() {
                    tera.add_raw_template(&format!("workspace/{}", n), contents)?;
                }

                for (n, _) in WORKSPACE.entries() {
                    let f = File::create(out_dir.join(n))?;
                    tera.render_to(&format!("workspace/{}", n), &context, f)?;
                }
                Ok(())
            },
            APP_CMD => todo!(), // TODO origen enw - support new app
            _ => unreachable_invalid_subc!(n)
        }
    } else {
        unreachable!()
    }
}