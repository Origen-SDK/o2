// FOR_PR clean up this and entire directory
pub mod helps;
pub mod extensions;
pub mod plugins;
pub mod aux_cmds;
pub mod app_cmds;
pub mod core_cmds;

pub use extensions::{Extensions, ExtensionTOML};
pub use plugins::{Plugins, Plugin};
pub use aux_cmds::AuxCmds;
pub use app_cmds::AppCmds;
pub use helps::{CmdHelps, CmdHelp, CmdSrc};
use std::{env};

use clap::{App};
use clap::Command as ClapCommand;
use origen::{Result, in_app_invocation};
use crate::commands::_prelude::clap_arg_actions::*;

#[macro_export]
macro_rules! from_toml_args {
    ($toml_args: expr) => {
        $toml_args.as_ref()
            .map(|args| args.iter()
                .map( |a| crate::framework::Arg::from_toml(a))
                .collect::<Vec<crate::framework::Arg>>())
    }
}

#[macro_export]
macro_rules! from_toml_opts {
    ($toml_opts: expr, $cmd_path: expr, $parent: expr) => {
        crate::from_toml_opts!($toml_opts, $cmd_path, $parent, None)
    };
    ($toml_opts: expr, $cmd_path: expr, $parent: expr, $ext: expr) => {
        $toml_opts.as_ref()
            .map(|opts| opts.iter()
                .map( |o| crate::framework::Opt::from_toml(o, $cmd_path, $parent, $ext))
                .collect::<Vec<crate::framework::Opt>>())
    }
}

pub trait Applies {
    fn in_app_context(&self) -> Option<bool>;
    fn in_global_context(&self) -> Option<bool>;
    fn on_env(&self) -> Option<&Vec<String>>;
    fn on_env_error_msg(&self, e: &String) -> String;

    fn applies(&self) -> Result<bool> {
        Ok(self.applies_with_env()? && self.applies_in_context()?)
    }

    fn applies_in_app_context(&self) -> Result<bool> {
        Ok(self.in_app_context().unwrap_or(true))
    }

    fn applies_in_global_context(&self) -> Result<bool> {
        Ok(self.in_global_context().unwrap_or(true))
    }

    fn applies_in_context(&self) -> Result<bool> {
        if in_app_invocation() {
            self.applies_in_app_context()
        } else {
            self.applies_in_global_context()
        }
    }

    fn applies_with_env(&self) -> Result<bool> {
        if let Some(envs) = self.on_env() {
            for e in envs {
                let mut s = e.splitn(1, '=');
                let e_name= s.next().ok_or_else( || self.on_env_error_msg(e))?.trim();
                let e_val = s.next();
                match env::var(e_name) {
                    Ok(val) => {
                        if let Some(v) = e_val {
                            if v == val {
                                return Ok(true);
                            }
                        } else {
                            return Ok(true);
                        }
                    },
                    Err(err) => match err {
                        env::VarError::NotPresent => {},
                        _ => {
                            return Err(err.into());
                        }
                    }
                }
            }
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

#[derive(Debug, Deserialize)]
pub (crate) struct CommandsToml {
    pub command: Option<Vec<CommandTOML>>,
    pub extension: Option<Vec<ExtensionTOML>>
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandTOML {
    pub name: String,
    pub help: String,
    pub alias: Option<String>,
    pub arg: Option<Vec<ArgTOML>>,
    pub opt: Option<Vec<OptTOML>>,
    pub subcommand: Option<Vec<CommandTOML>>,
    pub add_target_opt: Option<bool>,
    pub add_mode_opt: Option<bool>,
    pub in_global_context: Option<bool>,
    pub in_app_context: Option<bool>,
    pub on_env: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub help: String,
    pub alias: Option<String>,
    pub args: Option<Vec<Arg>>,
    pub opts: Option<Vec<Opt>>,
    pub subcommands: Option<Vec<String>>,
    pub full_name: String,
    pub add_mode_opt: Option<bool>,
    pub add_target_opt: Option<bool>,
    pub parent: CmdSrc,
    pub in_global_context: Option<bool>,
    pub in_app_context: Option<bool>,
    pub on_env: Option<Vec<String>>,
}

impl Command {
    pub fn from_toml_cmd(cmd: &CommandTOML, cmd_path: &str, parent: CmdSrc, parent_cmd: Option<&Self>) -> Result<Option<Self>> {
        let mut slf = Self {
            name: cmd.name.to_owned(),
            help: cmd.help.to_owned(),
            alias: cmd.alias.to_owned(),
            args: None,
            opts: None,
            subcommands: None,
            full_name: cmd_path.to_string(),
            add_mode_opt: cmd.add_mode_opt.or_else(|| {
                if let Some(p) = parent_cmd {
                    p.add_mode_opt.to_owned()
                } else {
                    None
                }
            }),
            add_target_opt: cmd.add_target_opt.or_else(|| {
                if let Some(p) = parent_cmd {
                    p.add_target_opt.to_owned()
                } else {
                    None
                }
            }),
            parent: parent,
            in_global_context: cmd.in_global_context,
            in_app_context: cmd.in_app_context,
            on_env: cmd.on_env.to_owned(),
        };
        if !slf.applies()? {
            return Ok(None)
        }
        slf.args = from_toml_args!(cmd.arg);
        slf.opts = from_toml_opts!(cmd.opt, cmd_path, &slf.parent);
        slf.subcommands = cmd.subcommand.as_ref().map(|sub_cmds| 
            sub_cmds.iter().map(|c| format!("{}.{}", cmd_path, &c.name.to_string())).collect::<Vec<String>>()
        );
        Ok(Some(slf))
    }

    pub fn add_mode_opt(&self) -> bool {
        self.add_mode_opt.unwrap_or(true)
    }

    pub fn add_target_opt(&self) -> bool {
        self.add_target_opt.unwrap_or(true)
    }
}

impl Applies for Command {
    fn in_global_context(&self) -> Option<bool> {
        self.in_global_context
    }

    fn in_app_context(&self) -> Option<bool> {
        self.in_app_context
    }

    fn on_env(&self) -> Option<&Vec<String>> {
        self.on_env.as_ref()
    }

    fn on_env_error_msg(&self, e: &String) -> String {
        format!("Failed to parse 'on_env' '{}', for command {}", e, self.full_name)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ArgTOML {
    pub name: String,
    pub help: String,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub value_name: Option<String>,
    pub use_delimiter: Option<bool>,
}

#[derive(Debug)]
pub struct Arg {
    pub name: String,
    pub help: String,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub value_name: Option<String>,
    pub use_delimiter: Option<bool>,
    pub upcased_name: Option<String>,
}

impl Arg {
    fn from_toml(arg: &ArgTOML) -> Self {
        Self {
            name: arg.name.to_owned(),
            help: arg.help.to_owned(),
            multiple: arg.multiple,
            required: arg.required,
            value_name: arg.value_name.to_owned(),
            use_delimiter: arg.use_delimiter,
            upcased_name: {
                if arg.value_name.is_some() {
                    None
                } else {
                    Some(arg.name.to_uppercase())
                }
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptTOML {
    pub name: String,
    pub help: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub takes_value: Option<bool>,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub value_name: Option<String>,
    pub use_delimiter: Option<bool>,
    pub short_aliases: Option<Vec<char>>,
    pub long_aliases: Option<Vec<String>>,
    pub hidden: Option<bool>,
}

#[derive(Debug)]
pub struct Opt {
    pub name: String,
    pub help: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub takes_value: Option<bool>,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub value_name: Option<String>,
    pub use_delimiter: Option<bool>,
    pub short_aliases: Option<Vec<char>>,
    pub long_aliases: Option<Vec<String>>,
    pub hidden: Option<bool>,
    pub upcased_name: Option<String>,
}

use core::fmt::Display;
impl Opt {
    fn from_toml(opt: &OptTOML, cmd_path: &str, parent: &dyn Display, ext: Option<&str>) -> Self {
        let err_prefix;
        if let Some(e) = ext {
            err_prefix = format!("Option '{}' extended from '{}' for command '{}' tried to use reserved option", opt.name, parent, e);
        } else {
            err_prefix = format!("Option '{}' from command '{}' tried to use reserved option", opt.name, parent);
        }

        Self {
            name: opt.name.to_owned(),
            help: opt.help.to_owned(),
            short: {
                opt.short.as_ref().and_then ( |sn| {
                    if RESERVED_OPT_SHORT_NAMES.contains(sn) {
                        log_error!(
                            "{} short name '{}' and will not be available as '-{}'",
                            err_prefix,
                            sn,
                            sn
                        );
                        None
                    } else {
                        Some(*sn)
                    }
                })
            },
            long: {
                opt.long.as_ref().and_then ( |ln| {
                    if RESERVED_OPT_NAMES.contains(&ln.as_str()) {
                        log_error!(
                            "{} long name '{}' and will not be available as '--{}'",
                            err_prefix,
                            ln,
                            ln
                        );
                        None
                    } else {
                        Some(ln.to_owned())
                    }
                })
            },
            takes_value: opt.takes_value,
            multiple: opt.multiple,
            required: opt.required,
            value_name: opt.value_name.to_owned(),
            use_delimiter: opt.use_delimiter,
            short_aliases: {
                opt.short_aliases.as_ref().map (|sns| sns.iter().filter_map( |sn| {
                    if RESERVED_OPT_SHORT_NAMES.contains(sn) {
                        log_error!(
                            "{} short name alias '{}' and will not be available as '-{}'",
                            err_prefix,
                            sn,
                            sn
                        );
                        None
                    } else {
                        Some(*sn)
                    }
                }).collect())
            },
            long_aliases: {
                opt.long_aliases.as_ref().map( |lns| lns.iter().filter_map( |ln| {
                    if RESERVED_OPT_NAMES.contains(&ln.as_str()) {
                        log_error!(
                            "{} long name alias '{}' and will not be available as '--{}'",
                            err_prefix,
                            ln,
                            ln
                        );
                        None
                    } else {
                        Some(ln.to_owned())
                    }
                }).collect())
            },
            hidden: opt.hidden,
            upcased_name: {
                if opt.value_name.is_some() {
                    None
                } else {
                    Some(opt.name.to_uppercase())
                }
            },
        }
    }
}

pub (crate) fn build_commands<'a, F, G, H>(cmd_def: &'a Command, exts: &G, cmd_container: &F, apply_helps: &H) -> App<'a>
where
    F: Fn(&str) -> &'a Command,
    G: Fn(&str, App<'a>) -> App<'a>,
    H: Fn(&str, App<'a>) -> App<'a>
{
    let mut cmd = ClapCommand::new(&cmd_def.name);

    cmd = add_app_opts(cmd, cmd_def.add_mode_opt(), cmd_def.add_target_opt());

    // TODO need test case for cmd alias
    if cmd_def.alias.is_some() {
        cmd = cmd.visible_alias(cmd_def.alias.as_ref().unwrap().as_str());
    }

    if let Some(args) = cmd_def.args.as_ref() {
        cmd = apply_args(args, cmd);
    }

    if let Some(opts) = cmd_def.opts.as_ref() {
        cmd = apply_opts(opts, cmd);
    }

    if let Some(subcommands) = &cmd_def.subcommands {
        for c in subcommands {
            let subcmd = build_commands(
                cmd_container(c),
                exts,
                cmd_container,
                apply_helps,
            );
            cmd = cmd.subcommand(subcmd);
        }
    }
    cmd = exts(&cmd_def.full_name, cmd);
    cmd = apply_helps(&cmd_def.full_name, cmd);

    cmd
}

pub (crate) fn apply_args<'a>(args: &'a Vec<Arg>, mut cmd: App<'a>) -> App<'a> {
    for arg_def in args {
        let mut arg = clap::Arg::new(arg_def.name.as_str())
            .action(SetArg)
            .help(arg_def.help.as_str());

        if let Some(vn) = arg_def.value_name.as_ref() {
            arg = arg.value_name(vn);
        } else {
            arg = arg.value_name(arg_def.upcased_name.as_ref().unwrap());
        }

        if let Some(d) = arg_def.use_delimiter {
            arg = arg.use_value_delimiter(d);
            arg = arg.multiple_values(true).action(AppendArgs);
        }
        if let Some(m) = arg_def.multiple {
            arg = arg.multiple_values(m).action(AppendArgs);
        }

        if let Some(r) = arg_def.required {
            arg = arg.required(r);
        }

        // FOR_PR is hidden arg a thing?
        // if arg_def.hidden.is_some() {
        //     arg = arg.hidden(arg_def.hidden.unwrap())
        // }
        cmd = cmd.arg(arg);
    }
    cmd
}

pub (crate) fn apply_opts<'a>(opts: &'a Vec<Opt>, mut cmd: App<'a>) -> App<'a> {
    for opt_def in opts {
        let mut opt = clap::Arg::new(opt_def.name.as_str()).action(CountArgs).help(opt_def.help.as_str());

        if let Some(val_name) = opt_def.value_name.as_ref() {
            opt = opt.value_name(val_name).action(SetArg);
        }
        if let Some(tv) = opt_def.takes_value {
            if tv {
                opt = opt.action(AppendArgs);
            }
        }
        if let Some(ud) = opt_def.use_delimiter {
            if ud {
                opt = opt.use_value_delimiter(ud);
            }
            opt = opt.multiple_values(true);
            opt = opt.action(AppendArgs);
        }
        if let Some(m) = opt_def.multiple {
            if m {
                opt = opt.multiple(m).action(AppendArgs);
            } else {
                opt = opt.multiple(m).action(SetArg);
            }
        }

        if let Some(ln) = opt_def.long.as_ref() {
            opt = opt.long(ln);
        } else {
            if !opt_def.short.is_some() {
                opt = opt.long(&opt_def.name);
            }
        }
        if let Some(sn) = opt_def.short {
            opt = opt.short(sn);
        }

        if let Some(r) = opt_def.required {
            opt = opt.required(r);
        }

        if let Some(h) = opt_def.hidden {
            opt = opt.hidden(h);
        }

        if opt.get_action().takes_values() && opt_def.value_name.is_none() {
            opt = opt.value_name(opt_def.upcased_name.as_ref().unwrap().as_str());
        }

        if let Some(lns) = opt_def.long_aliases.as_ref() {
            let v = lns.iter().map( |s| s.as_str() ).collect::<Vec<&str>>();
            opt = opt.visible_aliases(&v[..]);
        }

        if let Some(sns) = opt_def.short_aliases.as_ref() {
            opt = opt.visible_short_aliases(&sns[..].iter().map(|a| *a).collect::<Vec<char>>());
        }

        cmd = cmd.arg(opt);
    }
    cmd
}

pub fn build_path<'a>(mut matches: &'a clap::ArgMatches) -> Result<String> {
    let mut path_pieces = vec!();
    while matches.subcommand_name().is_some() {
        let n = matches.subcommand_name().unwrap();
        matches = matches.subcommand_matches(&n).unwrap();
        path_pieces.push(n);
    }
    Ok(path_pieces.join("."))
}

pub const HELP_OPT_NAME: &str = "help";
pub const HELP_OPT_SHORT_NAME: char = 'h';
pub const VERBOSITY_KEYWORDS_OPT_NAME: &str = "verbosity_keywords";
pub const VERBOSITY_KEYWORDS_OPT_LONG_NAME: &str = "vk";
pub const VERBOSITY_OPT_NAME: &str = "verbosity";
pub const VERBOSITY_OPT_SHORT_NAME: char = 'v';
pub const TARGET_OPT_NAME: &str = "targets";
pub const TARGET_OPT_ALIAS: &str = "target";
pub const TARGET_OPT_SN: char = 't';
pub const NO_TARGET_OPT_NAME: &str = "no_targets";
pub const NO_TARGET_OPT_ALIAS: &str = "no_target";
pub const MODE_OPT_NAME: &str = "mode";

pub const RESERVED_OPT_NAMES: &[&str] = &[
    HELP_OPT_NAME,
    VERBOSITY_KEYWORDS_OPT_NAME, VERBOSITY_KEYWORDS_OPT_LONG_NAME,
    TARGET_OPT_NAME, TARGET_OPT_ALIAS,
    NO_TARGET_OPT_NAME, NO_TARGET_OPT_ALIAS,
    MODE_OPT_NAME,
    VERBOSITY_OPT_NAME
];

pub const RESERVED_OPT_SHORT_NAMES: &[char] = &[
    HELP_OPT_SHORT_NAME, TARGET_OPT_SN, VERBOSITY_OPT_SHORT_NAME
];

macro_rules! add_mode_opt {
    ($cmd:expr) => {
        $cmd.arg(clap::Arg::new(crate::framework::MODE_OPT_NAME)
            .long("mode")
            .value_name("MODE")
            .help("Override the default mode currently set by the workspace for this command")
            .action(crate::framework::SetArg)
        )
    }
}

macro_rules! add_target_opt {
    ($cmd:expr) => {
        $cmd.arg(clap::Arg::new(crate::framework::TARGET_OPT_NAME)
            .short('t')
            .long(crate::framework::TARGET_OPT_NAME)
            .visible_alias(TARGET_OPT_ALIAS)
            .help("Override the targets currently set by the workspace for this command")
            .action(crate::commands::_prelude::AppendArgs)
            .use_value_delimiter(true)
            .multiple_values(true)
            .value_name("TARGETS")
            .conflicts_with(crate::framework::NO_TARGET_OPT_NAME)
        ).arg(clap::Arg::new(crate::framework::NO_TARGET_OPT_NAME)
            .long(crate::framework::NO_TARGET_OPT_NAME)
            .visible_alias(NO_TARGET_OPT_ALIAS)
            .help("Clear any targets currently set by the workspace for this command")
            .action(crate::commands::_prelude::SetArgTrue)
        )
    }
}

pub fn add_app_opts(mut cmd: ClapCommand, add_mode_opt: bool, add_target_opt: bool) -> ClapCommand {
    if in_app_invocation() {
        if add_target_opt {
            cmd = add_target_opt!(cmd);
        }
        if add_mode_opt {
            cmd = add_mode_opt!(cmd);
        }
    }
    cmd
}

pub fn add_all_app_opts(cmd: ClapCommand) -> ClapCommand {
    if in_app_invocation() {
        add_mode_opt!(add_target_opt!(cmd))
    } else {
        cmd
    }
}
