pub mod app;
pub mod env;
pub mod exec;
// pub mod fmt;
pub mod interactive;
// pub mod mode;
// pub mod new;
// pub mod proj;
// pub mod save_ref;
pub mod target;
// pub mod mailer;
pub mod credentials;
pub mod develop_origen;
pub mod generate;
pub mod eval;
pub mod aux_cmds;
pub mod plugin;
pub mod plugins;
pub mod _prelude;

use crate::python;
use crate::strs_to_cli_arr;

use indexmap::map::IndexMap;
use origen::{LOGGER, STATUS}; // clean_mode # O1_MODE_SUPPORT clean_mode function available, or remove
use std::process::exit;
use _prelude::{SetArgTrue, CountArgs};

use clap::{App, ArgMatches};
use crate::framework::extensions::{Extension, ExtensionSource};
use crate::Plugins;
use std::collections::HashMap;

#[macro_export]
macro_rules! unreachable_invalid_subc {
    ($subcmd:expr) => {{
        unreachable!("Uncaught Invalid Subcommand {} From {}", $subcmd, module_path!())
    }}
}

#[macro_export]
macro_rules! print_subcmds_available_msg {
    () => {{
        println!("Run with 'help' or '-h' to see available subcommands");
    }}
}

#[macro_export]
macro_rules! gen_simple_run_func {
    ($base_cmd: expr) => {
        pub(crate) fn run(mut invocation: &clap::ArgMatches, mut cmd_def: &clap::App, exts: &crate::Extensions, plugins: Option<&crate::Plugins>) -> origen::Result<()> {
            let mut path_pieces: Vec<String> = vec!();
            cmd_def = cmd_def.find_subcommand($base_cmd).unwrap();
            if invocation.subcommand_name().is_some() {
                while invocation.subcommand_name().is_some() {
                    let n = invocation.subcommand_name().unwrap();
                    invocation = invocation.subcommand_matches(&n).unwrap();
                    cmd_def = cmd_def.find_subcommand(n).unwrap();
                    path_pieces.push(n.to_string());
                }
                crate::commands::launch(
                    Some($base_cmd),
                    Some(&path_pieces),
                    invocation,
                    cmd_def,
                    exts.get_core_ext(&format!("{}.{}", $base_cmd, path_pieces.join("."))),
                    plugins,
                    None,
                );
            } else {
                crate::commands::launch_from_invocation(
                    invocation,
                    cmd_def,
                    exts.get_core_ext($base_cmd),
                    plugins,
                );
            }
            Ok(())
        }
    };
    () => {
        crate::gen_simple_run_func!(BASE_CMD);
    }
}

pub fn launch_as(
    cmd: &str,
    subcmds: Option<&Vec<String>>,
    invocation: &ArgMatches,
    cmd_def: &App,
    cmd_exts: Option<&Vec<Extension>>,
    plugins: Option<&Plugins>,
    overrides: Option<IndexMap<String, Option<String>>>,
) -> ()
{
    launch(Some(cmd), subcmds, invocation, cmd_def, cmd_exts, plugins, overrides)
}
pub fn launch_from_invocation(invocation: &ArgMatches, cmd_def: &App, cmd_exts: Option<&Vec<Extension>>, plugins: Option<&Plugins>) {
    launch(None, None, invocation, cmd_def, cmd_exts, plugins, None)
}

pub fn launch(base_cmd: Option<&str>, subcmds: Option<&Vec<String>>, invocation: &ArgMatches, cmd_def: &App, cmd_exts: Option<&Vec<Extension>>, plugins: Option<&Plugins>, overrides: Option<IndexMap<String, Option<String>>>) {
    macro_rules! as_name {
        ($arg_name:expr) => {{
            if $arg_name.starts_with(crate::framework::extensions::EXT_BASE_NAME) {
                $arg_name.splitn(4, ".").last().unwrap()
            } else {
                $arg_name
            }
        }}
    }

    let mut args: Vec<String> = vec!();
    let mut arg_indices: Vec<String> = vec!();

    let mut opt_names = HashMap::new();
    let mut ext_args: HashMap<&ExtensionSource, Vec<String>> = HashMap::new();
    let mut ext_arg_indices: HashMap<&ExtensionSource, Vec<String>> = HashMap::new();
    if let Some(exts) = cmd_exts {
        for ext in exts {
            if let Some(opts) = ext.opts.as_ref() {
                for opt in opts {
                    opt_names.insert(opt.full_name.as_ref().unwrap().as_str(), &ext.source);
                    if !ext_args.contains_key(&ext.source) {
                        ext_args.insert(&ext.source, vec!());
                        ext_arg_indices.insert(&ext.source, vec!());
                    }
                }
            }
        }
    }

    let mut targets = None;

    for arg in cmd_def.get_arguments() {
        let arg_n= arg.get_id();
        if arg_n == "verbose" || arg_n == "verbosity_keywords" {
            continue;
        }

        if invocation.contains_id(arg_n) {
            if arg_n == "targets" {
                targets = Some(invocation.get_many::<String>(arg_n).unwrap());
                continue;
            } else if arg_n == "no_targets" {
                if *invocation.get_one::<bool>(arg_n).unwrap() {
                    targets = Some(clap::parser::ValuesRef::default());
                }
                continue;
            } else if arg_n == "mode" {
                // O1_MODE_SUPPORT
                todo!("Mode argument is not currently supported!");
            }

            let arg_str: String;
            if arg.is_takes_value_set() {
                if arg.is_multiple_values_set() {
                    // Give to Python as an array of string values
                    let r = invocation.get_many::<String>(arg_n).unwrap().map(|x| format!("\"{}\"", x)).collect::<Vec<String>>();
                    arg_str = format!("r'{}': [{}]", as_name!(arg_n), r.join(", "));
                } else {
                    // Give to Python a single string value
                    arg_str = format!("r'{}': r'{}'", as_name!(arg_n), invocation.get_one::<String>(arg_n).unwrap());
                }
            } else {
                match arg.get_action() {
                    SetArgTrue => {
                        if *(invocation.get_one::<bool>(arg_n).unwrap()) {
                            arg_str = format!("r'{}': True", as_name!(arg_n));
                        } else {
                            continue;
                        }
                    },
                    CountArgs => {
                        let count = *(invocation.get_one::<u8>(arg_n).unwrap());
                        if count > 0 {
                            arg_str = format!("r'{}': {}", as_name!(arg_n), count);
                        } else {
                            continue;
                        }
                    },
                    _ => {
                        log_error!("Unsupported action '{:#?}' for arg '{}'", arg.get_action(), as_name!(arg_n)); //arg_str = format!("r'{}': True", arg_n)
                        exit(1);
                    }
                }
            }
            let indices_str = format!(
                "r'{}': [{}]",
                as_name!(arg_n),
                invocation.indices_of(arg_n).unwrap().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
            );
            if let Some(ext_src) = opt_names.get(arg_n) {
                ext_args.get_mut(ext_src).unwrap().push(arg_str);
                ext_arg_indices.get_mut(ext_src).unwrap().push(indices_str);
            } else {
                args.push(arg_str);
                arg_indices.push(indices_str);
            }
        }
    }

    let mut cmd = format!("from origen.boot import run_cmd; run_cmd('{}'", base_cmd.unwrap_or_else(|| cmd_def.get_name()));
    if let Some(subs) = subcmds.as_ref() {
        cmd += &format!(", subcmds=[{}]", subs.iter().map( |s| format!("r'{}'", s) ).collect::<Vec<String>>().join(", "));
    }
    cmd += &format!(", args={{{}}}, arg_indices={{{}}}", args.join(", "), arg_indices.join(", "));

    let mut app_ext_str = "".to_string();
    let mut pl_ext_str = "".to_string();
    let mut aux_ext_str = "".to_string();
    let mut app_ext_indices_str = "".to_string();
    let mut pl_ext_indices_str = "".to_string();
    let mut aux_ext_indices_str = "".to_string();
    if !ext_args.is_empty() {
        for ext in ext_args {
            match ext.0 {
                ExtensionSource::App => {
                    app_ext_str = ext.1.join(", ");
                    app_ext_indices_str = ext_arg_indices[ext.0].join(", ");
                },
                ExtensionSource::Plugin(ref pl_name) => {
                    pl_ext_str += &format!(", '{}': {{{}}}", pl_name, ext.1.join(", "));
                    pl_ext_indices_str += &format!(", '{}': {{{}}}", pl_name, ext_arg_indices[ext.0].join(", "));
                },
                ExtensionSource::Aux(ref ns, _) => {
                    aux_ext_str += &format!(", '{}': {{{}}}", ns, ext.1.join(", "));
                    aux_ext_indices_str += &format!(", '{}': {{{}}}", ns, ext_arg_indices[ext.0].join(", "));
                },
            }
        }
        if !pl_ext_str.is_empty() {
            pl_ext_str = pl_ext_str[2..].to_string();
            pl_ext_indices_str = pl_ext_indices_str[2..].to_string();
        }
        if !aux_ext_str.is_empty() {
            aux_ext_str = aux_ext_str[2..].to_string();
            aux_ext_indices_str = aux_ext_indices_str[2..].to_string();
        }
    }
    cmd += &format!(
        concat!(
            ", ext_args={{'app': {{{}}}, 'plugin': {{{}}}, 'aux': {{{}}}}}",
            ", ext_arg_indices={{'app': {{{}}}, 'plugin': {{{}}}, 'aux': {{{}}}}}"
        ),
        app_ext_str,
        pl_ext_str,
        aux_ext_str,
        app_ext_indices_str,
        pl_ext_indices_str,
        aux_ext_indices_str,
    );

    if let Some(exts) = cmd_exts {
        let mut ext_setups: Vec<String> = vec!();
        for ext in exts {
            let mut ext_setup = "{".to_string();
            match ext.source {
                ExtensionSource::App => {
                    ext_setup += &format!("'source': 'app', 'root': r'{}', 'name': None", origen::app().unwrap().root.join(format!("{}/commands/extensions/", STATUS.app.as_ref().unwrap().name())).display());
                },
                ExtensionSource::Plugin(ref pl_name) => {
                    ext_setup += &format!(
                        "'root': r'{}', 'name': r'{}', 'source': 'plugin'",
                        plugins.unwrap().plugins.get(pl_name).unwrap().root.as_path().join("commands/extensions/").display(),
                        pl_name,
                    );
                },
                ExtensionSource::Aux(ref ns, ref path) => {
                    ext_setup += &format!(
                        "'root': r'{}', 'name': r'{}', 'source': 'aux'",
                        path.display(),
                        ns,
                    );
                }
            }
            ext_setup += "}";
            ext_setups.push(ext_setup);
        }
        cmd += &format!(", extensions=[{}]", ext_setups.join(", "));
    }

    if let Some(pls) = plugins {
        cmd += &format!(
            ", plugins={{{}}}",
            pls.plugins.iter().map(|(n, pl)| format!("'{}': {{'root': r'{}'}}", n, pl.root.display())).collect::<Vec<String>>().join(", ")
        );
    }

    if let Some(top_overrides) = overrides {
        for (name, val) in top_overrides.iter() {
            if let Some(v) = val {
                cmd += &format!(", {}={}", name, v);
            }
        }
    }

    if let Some(targs) = targets {
        if targs.clone().count() == 0 {
            cmd += ", targets=False"
        } else {
            cmd += &format!(", {}", strs_to_cli_arr!("targets", targs));
        }
    }
    cmd += &format!(", verbosity={}", LOGGER.verbosity());
    cmd += &format!(", {}", strs_to_cli_arr!("verbosity_keywords", origen::LOGGER.data().keywords.iter()));
    cmd += ");";

    log_debug!("Launching Python: '{}'", &cmd);

    match python::run(&cmd) {
        Err(e) => {
            log_error!("{}", &e);
            exit(1);
        }
        Ok(exit_status) => {
            if exit_status.success() {
                exit(0);
            } else {
                exit(exit_status.code().unwrap_or(1));
            }
        }
    }
}
