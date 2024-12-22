use crate::commands::_prelude::*;
use origen_metal::tera::{Context, Tera};
use std::process::exit;
use std::env;
use std::fs::{create_dir, File};
use std::path::{Path, PathBuf};
use crate::_generated::python::PYTHONS;
use crate::python::get_current_user_and_email;

pub const BASE_CMD: &'static str = "new";
pub const WS_CMD: &'static str = "workspace";
pub const APP_CMD: &'static str = "application";
pub const PL_CMD: &'static str = "plugin";

lazy_static! {
    static ref APP_NS_DIR: &'static str = "app_namespace_dir/";
}

// This includes a map of all template files, it is built by cli/build.rs at compile time.
// All files in each sub-directory of commands/new/templates are accessible via a map named after the
// uppercased sub_directory, e.g.
//      PYTHON_APP = { "pyproject.toml" => "[tool.poetry]...", "config/application.toml" => "..." }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new app.
include!(concat!(env!("OUT_DIR"), "/new_app_templates.rs"));

macro_rules! common_new_args {
    ( $cmd: expr, $new_type: expr ) => {{
        $cmd.arg(req_sv_arg!("name", "NAME", concat!($new_type, " name")))
        .arg(sv_opt!("desc", "DESC", concat!("Description of the ", $new_type)).visible_alias("description"))
        .arg(sv_opt!("path", "PATH", "Path to build the new workspace").short('p'))
    }};
}

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
    }}),
    core_subcmd__no_exts__no_app_opts!(PL_CMD, "Create a new workspace", { |cmd: App| {
        common_new_args!(cmd, "Plugin")
            .visible_alias("pl")
    }}),
    // TODO origen new - support new app
    core_subcmd__no_exts__no_app_opts!(APP_CMD, "Create a new application", { |cmd: App| {
        common_new_args!(cmd, "Application")
            .visible_alias("app")
            // .arg(sv_opt!("dut", "DUT NAME", "Use a different DUT name (default: dut). Cannot be used with --no_dut"))
            // .arg(sf_opt!("no_dut", "Do not create a DUT. Cannot be used with --dut <DUT NAME>"))
    }})
);

pub fn current_user_to_author() -> Result<String> {
    let info = get_current_user_and_email()?;
    Ok(format!("{} {} <{}>", info.0, info.1, info.2))
}

pub fn run(invocation: &clap::ArgMatches) -> origen::Result<()> {
    if let Some((n, subcmd)) = invocation.subcommand() {
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

        //  Check output dir but hold off until other checks have passed
        if out_dir.exists() {
            // Check directory is empty
            if !out_dir.read_dir()?.next().is_none() {
                log_error!("Target directory {} is not empty!", &out_dir.display());
                exit(1);
            }
        }

        context.insert("name", name);
        context.insert("desc", subcmd.get_one::<String>("desc").unwrap_or(&"".to_string()));

        // Add author to context
        let mut author = "".to_string();
        if origen_fe_available!() {
            match current_user_to_author() {
                Ok(n) => author = n,
                Err(e) => {
                    log_warning!("Errors occurred getting the current username and email from origen: {}", e);
                }
            }
        } else {
            if let Err(e) = origen_metal::try_lookup_and_set_current_user() {
                log_warning!("Errors occurred populating current user: {}", e);
            } else {
                let users = origen_metal::users();
                match users.current_user() {
                    Ok(u) => {
                        match u.username() {
                            Ok(username) => {
                                match u.get_email() {
                                    Ok(e) => {
                                        if let Some(email) = e {
                                            author += &format!("{} <{}>", &username, &email);
                                        } else {
                                            log_warning!("Could not retrieve user email. Only including username in 'author'");
                                            author += &username;
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
                    },
                    Err(e) => {
                        log_warning!("Errors occurred populating current user: {}", e);
                    }
                }
            }
        }
        context.insert("author", &author);

        // Add Python, Origen, and other library versions
        context.insert("origen_version", &origen::STATUS.origen_version.to_string());
        context.insert("python_version", &format!(
            ">={},<={}",
            PYTHONS[2].strip_prefix("python").unwrap(),
            PYTHONS.last().unwrap().strip_prefix("python").unwrap()
        ));
        // TODO origen new - find a better way than hard-coding pytest version
        context.insert("pytest_version", "^7");
        log_trace!("'origen new' context: {:?}", context);

        let mut tera = Tera::default();
        for (n, contents) in SHARED.entries() {
            tera.add_raw_template(&format!("shared/{}", n), contents)?;
        }

        let (app_gen, pl_gen, ws_gen, path_base): (bool, bool, bool, &str);
        match n {
            WS_CMD => {
                app_gen = false;
                pl_gen = false;
                ws_gen = true;
                path_base = "workspace";
                for (n, contents) in WORKSPACE.entries() {
                    tera.add_raw_template(&format!("{}/{}", path_base, n), contents)?;
                }
            },
            PL_CMD => {
                app_gen = false;
                pl_gen = true;
                ws_gen = false;
                path_base = "plugin";

                for (n, contents) in PY_APP.entries() {
                    tera.add_raw_template(&format!("{}/{}", path_base, n), contents)?;
                }
            },
            APP_CMD => {
                app_gen = true;
                pl_gen = false;
                ws_gen = false;
                path_base = "app";

                todo!()
            }, // TODO origen enw - support new app
            _ => unreachable_invalid_subc!(n)
        }
        context.insert("app_gen", &app_gen);
        context.insert("pl_gen", &pl_gen);
        context.insert("ws_gen", &ws_gen);

        if !out_dir.exists() {
            create_dir(&out_dir)?;
        }

        let base_prefix = format!("{}/", path_base);
        let mut errored = false;
        for t in tera.get_template_names() {
            if let Some(p) = t.strip_prefix(&base_prefix) {
                let path;
                if let Ok(p2) = Path::new(p).strip_prefix(*APP_NS_DIR) {
                    log_debug!("Moving template from '{}' space to '{}' space", *APP_NS_DIR, name);
                    path = out_dir.join(name).join(p2);
                } else {
                    path = out_dir.join(p);
                }
                displayln!("Rendering template...");
                displayln!("    {}", t);
                displayln!("=>  {}", path.display());

                std::fs::create_dir_all(path.parent().unwrap_or_else(|| {Path::new("")}))?;
                let f = File::create(path)?;

                match tera.render_to(t, &context, f) {
                    Ok(()) => {
                        display_greenln!("    Success!");
                    },
                    Err(e) => {
                        errored = true;
                        display_redln!("    {}", origen_metal::Error::from(e));
                    }
                }
            }
        }

        if errored {
            display_redln!("Failed to create new {}. Please review output logs for errors message.", path_base);
            bail!("Failed to create new {}", path_base)
        } else {
            display_greenln!("Created new {}:", path_base);
            display_greenln!("    {}", &out_dir.display());
            Ok(())
        }
    } else {
        unreachable!()
    }
}