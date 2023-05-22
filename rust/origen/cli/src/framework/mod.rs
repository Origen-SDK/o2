pub mod helps;
pub mod extensions;
pub mod plugins;
pub mod aux_cmds;
pub mod app_cmds;
pub mod core_cmds;

use std::collections::HashMap;
use origen_metal::indexmap::IndexMap;

pub use extensions::{Extensions, ExtensionTOML, Extension};
pub use plugins::{Plugins, Plugin};
pub use aux_cmds::AuxCmds;
pub use app_cmds::AppCmds;
pub use helps::{CmdHelps, CmdHelp, CmdSrc};
use std::{env};

use clap::{App};
use clap::Command as ClapCommand;
use clap::Arg as ClapArg;
use origen::{Result, in_app_invocation};
use crate::commands::_prelude::clap_arg_actions::*;

#[macro_export]
macro_rules! uses_reserved_prefix {
    ($q:expr) => {{
        $q.starts_with(crate::framework::extensions::EXT_BASE_PREFIX)
    }}
}

#[macro_export]
macro_rules! err_processing_cmd_preface {
    ($func:ident, $cmd_path:expr, $msg:expr, $($arg:expr),* $(,)?) => {{
        $func!(concat!("When processing command '{}': ", $msg), $cmd_path, $($arg),*)
    }}
}

#[macro_export]
macro_rules! log_err_processing_cmd {
    ($cmd_path:expr, $msg:expr, $($arg:expr),* $(,)?) => {{
        crate::err_processing_cmd_preface!(log_error, $cmd_path, $msg, $($arg),*)
    }};
}

#[macro_export]
macro_rules! from_toml_args {
    ($toml_args: expr, $cmd_path: expr) => {{
        let mut current_names: Vec<Option<&str>> = vec!();
        $toml_args.as_ref()
            .map(|args| args.iter()
                .filter_map( |a| {
                    if let Some(i) = current_names.iter().position( |n| *n == Some(&a.name)) {
                        crate::log_err_processing_cmd!(
                            $cmd_path,
                            "Argument '{}' is already present. Subsequent occurrences will be skipped (first occurrence at index {})",
                            &a.name,
                            i
                        );
                        current_names.push(None);
                        None
                    } else if crate::uses_reserved_prefix!(a.name) { // a.name.starts_with(crate::framework::extensions::EXT_BASE_NAME) {
                        crate::log_err_processing_cmd!(
                            $cmd_path,
                            "Argument '{}' uses reserved prefix '{}'. This option will not be available.",
                            &a.name,
                            crate::framework::extensions::EXT_BASE_NAME
                        );
                        current_names.push(None);
                        None
                    } else {
                        current_names.push(Some(&a.name));
                        Some(crate::framework::Arg::from_toml(a))
                    }
                })
                .collect::<Vec<crate::framework::Arg>>())
    }}
}

#[macro_export]
macro_rules! from_toml_opts {
    ($toml_opts: expr, $cmd_path: expr, $parent: expr) => {
        crate::from_toml_opts!($toml_opts, $cmd_path, $parent, None::<&str>)
    };
    ($toml_opts: expr, $cmd_path: expr, $parent: expr, $ext: expr) => {{
        let mut current_names: Vec<Option<&str>> = vec!();
        $toml_opts.as_ref()
            .map(|opts| opts.iter()
                .filter_map( |o| {
                    if let Some(i) = current_names.iter().position( |n| *n == Some(&o.name)) {
                        if $ext.is_some() {
                            crate::log_err_processing_cmd!(
                                $cmd_path,
                                "Option '{}' extended from {} is already present. Subsequent occurrences will be skipped (first occurrence at index {})",
                                &o.name,
                                $parent,
                                i
                            );
                        } else {
                            crate::log_err_processing_cmd!(
                                $cmd_path,
                                "Option '{}' is already present. Subsequent occurrences will be skipped (first occurrence at index {})",
                                &o.name,
                                i
                            );
                        }
                        current_names.push(None);
                        None
                    } else if crate::uses_reserved_prefix!(o.name) {
                        if $ext.is_some() {
                            crate::log_err_processing_cmd!(
                                $cmd_path,
                                "Option '{}' extended from {} uses reserved prefix '{}'. This option will not be available",
                                &o.name,
                                $parent,
                                crate::framework::extensions::EXT_BASE_NAME
                            );
                        } else {
                            crate::log_err_processing_cmd!(
                                $cmd_path,
                                "Option '{}' uses reserved prefix '{}'. This option will not be available",
                                &o.name,
                                crate::framework::extensions::EXT_BASE_NAME
                            );
                        }
                        current_names.push(None);
                        None
                    } else {
                        current_names.push(Some(&o.name));
                        Some(crate::framework::Opt::from_toml(o, $cmd_path, $parent, $ext))
                    }
                })
                .collect::<Vec<crate::framework::Opt>>())
    }}
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
    // Name offset from the command type (e.g., cmd.subc, instead of origen.cmd.subc, or plugin.pl.cmd.subc)
    pub full_name: String,
    pub add_mode_opt: Option<bool>,
    pub add_target_opt: Option<bool>,
    // FOR_PR seems src not parent
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
        let fp = slf.parent.to_string();
        slf.args = from_toml_args!(cmd.arg, &fp);
        slf.opts = from_toml_opts!(cmd.opt, &fp, &slf.parent);
        if let Some(args) = slf.args.as_ref() {
            if let Some(opts) = slf.opts.as_mut() {
                opts.retain(|o| {
                    if let Some(idx) = args.iter().position(|a| a.name == o.name) {
                        crate::log_err_processing_cmd!(
                            &fp,
                            "Option '{}' conflicts with Arg of the same name (Arg #{})",
                            o.name,
                            idx,
                        );
                        false
                    } else {
                        true
                    }
                });
            }
        }
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

#[derive(Debug)]
pub struct CmdOptCache {
    opt_names: Vec<String>,
    lns: HashMap<String, (bool, usize)>,
    ilns: HashMap<String, (bool, usize)>,
    ln_aliases: HashMap<String, (bool, usize)>,
    sns: HashMap<char, (bool, usize)>,
    sn_aliases: HashMap<char, (bool, usize)>,
    ext_opt_names: IndexMap<String, usize>,
    exts: Vec<String>,
    cmd_path: String,
    last_needs_visible_full_name: bool,
    current: String,

}

macro_rules! processing_exts {
    ($slf:expr) => {{
        $slf.exts.len() > 0
    }}
}

macro_rules! conflict_err_msg {
    ($self:expr, $conflict:expr, $name:expr, $conflict_type:expr, $with_type:expr) => {{
        if processing_exts!($self) {
            if $conflict.0 {
                // Conflict with the command itself
                log_err_processing_cmd!(
                    $self.cmd_path,
                    concat!($conflict_type, " '{}' for extension option '{}', from {}, conflicts with ", $with_type, " from command option '{}'"),
                    $name,
                    $self.current,
                    $self.exts.last().unwrap(),
                    $self.opt_names[$conflict.1]
                );
            } else {
                // Conflict with an extension
                let e = $self.ext_opt_names.get_index($conflict.1).unwrap();
                log_err_processing_cmd!(
                    $self.cmd_path,
                    concat!($conflict_type, " '{}' for extension option '{}', from {}, conflicts with ", $with_type, " for extension '{}' provided by {}"),
                    $name,
                    $self.current,
                    $self.exts.last().unwrap(),
                    e.0,
                    $self.exts[*e.1]
                );
            }
        } else {
            log_err_processing_cmd!(
                $self.cmd_path,
                concat!($conflict_type, " '{}' for command option '{}' conflicts with ", $with_type, " from option '{}'"),
                $name,
                $self.current,
                $self.opt_names[$conflict.1]
            );
        }
    }}
}

macro_rules! cache {
    ($slf:expr, $cache:ident, $to_cache:expr) => {{
        $slf.$cache.insert($to_cache, {
            if processing_exts!($slf) {
                (false, $slf.ext_opt_names.len() - 1)
            } else {
                (true, $slf.opt_names.len() - 1)
            }
        });
    }}
}

impl CmdOptCache {
    pub fn new(cmd_path: String) -> Self {
        let slf = Self {
            opt_names: Vec::new(),
            lns: HashMap::new(),
            ilns: HashMap::new(),
            ln_aliases: HashMap::new(),
            sns: HashMap::new(),
            sn_aliases: HashMap::new(),
            ext_opt_names: IndexMap::new(),
            exts: Vec::new(),
            cmd_path: cmd_path,
            last_needs_visible_full_name: true,
            current: "".to_string(),
        };
        slf
    }

    pub fn unchecked_populated(cmd: &App, cmd_path: String) -> Self {
        let mut slf = Self::new(cmd_path);
        for (i, arg) in cmd.get_arguments().enumerate() {
            let mut push_arg = false;
            if let Some(ln) = arg.get_long() {
                slf.lns.insert(ln.to_string(), (true, i));
                push_arg = true;
            }
            if let Some(lns) = arg.get_all_aliases() {
                slf.ln_aliases.extend(lns.iter().map( |ln| (ln.to_string(), (true, i))).collect::<Vec<(String, (bool, usize))>>());
                push_arg = true;
            }
            if let Some(sn) = arg.get_short() {
                slf.sns.insert(sn, (true, i));
                push_arg = true;
            }
            if let Some(sns) = arg.get_all_short_aliases() {
                slf.sn_aliases.extend(sns.iter().map( |sn| (*sn, (true, i))).collect::<Vec<(char, (bool, usize))>>());
                push_arg = true;
            }

            if push_arg {
                slf.opt_names.push(arg.get_id().to_string());
            }
        }
        slf
    }

    pub fn register(&mut self, name: &String, ext: Option<&Extension>) -> bool {
        // TODO name conflict. Probably a better way to deal with this but just skip for now
        self.current = name.to_string();
        if let Some(e) = ext {
            self.exts.push(e.source.to_string());
            if !self.ext_opt_names.contains_key(name) {
                self.ext_opt_names.insert(name.to_string(), self.exts.len() - 1);
            }
        } else {
            self.opt_names.push(name.to_string());
        }
        true
    }

    pub fn iln_conflicts(&mut self, iln: &String) -> bool {
        if let Some(conflict) = self.ln_aliases.get(iln) {
            conflict_err_msg!(self, conflict, iln, "Inferred long name", "long name alias");
            true
        } else if let Some(conflict) = self.lns.get(iln) {
            conflict_err_msg!(self, conflict, iln, "Inferred long name", "long name");
            true
        } else if let Some(conflict) = self.ilns.get(iln) {
            conflict_err_msg!(self, conflict, iln, "Inferred long name", "inferred long name");
            true
        } else {
            cache!(self, ilns, iln.to_owned());
            self.last_needs_visible_full_name = false;
            false
        }
    }

    pub fn ln_conflicts(&mut self, ln: &String) -> bool {
        if let Some(conflict) = self.ln_aliases.get(ln) {
            conflict_err_msg!(self, conflict, ln, "Long name", "long name alias");
            true
        } else if let Some(conflict) = self.lns.get(ln) {
            conflict_err_msg!(self, conflict, ln, "Long name", "long name");
            true
        } else if let Some(conflict) = self.ilns.get(ln) {
            conflict_err_msg!(self, conflict, ln, "Long name", "inferred long name");
            true
        } else {
            cache!(self, lns, ln.to_owned());
            self.last_needs_visible_full_name = false;
            false
        }
    }

    pub fn sn_conflicts(&mut self, sn: char) -> bool {
        if let Some(conflict) = self.sn_aliases.get(&sn) {
            conflict_err_msg!(self, conflict, sn, "Short name", "short name alias");
            true
        } else if let Some(conflict) = self.sns.get(&sn) {
            conflict_err_msg!(self, conflict, sn, "Short name", "short name");
            true
        } else {
            cache!(self, sns, sn);
            self.last_needs_visible_full_name = false;
            false
        }
    }

    pub fn non_conflicting_snas(&mut self, snas: &Vec<char>) -> Vec<char> {
        snas.iter().filter_map( |sna| {
            if let Some(conflict) = self.sn_aliases.get(sna) {
                conflict_err_msg!(self, conflict, sna, "Short name alias", "short name alias");
                None
            } else if let Some(conflict) = self.sns.get(sna) {
                conflict_err_msg!(self, conflict, sna, "Short name alias", "short name");
                None
            } else {
                cache!(self, sn_aliases, *sna);
                Some(*sna)
            }
        }).collect::<Vec<char>>()
    }

    pub fn non_conflicting_lnas<'a>(&mut self, lnas: &'a Vec<String>) -> Vec<&'a str> {
        lnas.iter().filter_map( |lna| {
            if let Some(conflict) = self.ln_aliases.get(lna) {
                conflict_err_msg!(self, conflict, lna, "Long name alias", "long name alias");
                None
            } else if let Some(conflict) = self.lns.get(lna) {
                conflict_err_msg!(self, conflict, lna, "Long name alias", "long name");
                None
            } else if let Some(conflict) = self.ilns.get(lna) {
                conflict_err_msg!(self, conflict, lna, "Long name alias", "inferred long name");
                None
            } else {
                cache!(self, ln_aliases, lna.to_owned());
                Some(lna.as_str())
            }
        }).collect::<Vec<&str>>()
    }

    pub fn needs_visible_full_name(&mut self) -> bool {
        let retn = self.last_needs_visible_full_name;
        self.last_needs_visible_full_name = true;
        retn
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
    pub full_name: Option<String>,
}

use core::fmt::Display;
impl Opt {
    // FOR_PR is cmd_path/parent the same here?
    fn from_toml(opt: &OptTOML, cmd_path: &str, parent: &dyn Display, ext: Option<&str>) -> Self {
        macro_rules! gen_err {
            ($msg:tt $(,)? $($arg:expr),*) => {{
                if ext.is_some() {
                    log_err_processing_cmd!(cmd_path, concat!("Option '{}' extended from {} ", $msg), opt.name, parent, $($arg),*);
                } else {
                    log_err_processing_cmd!(cmd_path, concat!("Option '{}' ", $msg), opt.name, $($arg),*);
                }
            }}
        }

        macro_rules! res_opt_ln_msg {
            ($conflict:expr, $name:expr) => {
                gen_err!(
                    "tried to use reserved option {} '{}' and will not be available as '--{}'",
                    $conflict,
                    $name,
                    $name
                );
            }
        }
        macro_rules! res_opt_sn_msg {
            ($conflict:expr, $name:expr) => {
                gen_err!(
                    "tried to use reserved option {} '{}' and will not be available as '-{}'",
                    $conflict,
                    $name,
                    $name
                )
            }
        }
        macro_rules! res_prefix_msg {
            ($conflict:expr, $name:expr) => {
                gen_err!(
                    "uses reserved prefix '{}' in {} '{}' and will not be available as '--{}'",
                    crate::framework::extensions::EXT_BASE_NAME,
                    $conflict,
                    $name,
                    $name
                )
            }
        }

        let ln = opt.long.as_ref().and_then ( |ln| {
            if RESERVED_OPT_NAMES.contains(&ln.as_str()) {
                res_opt_ln_msg!("long name", ln);
                None
            } else if uses_reserved_prefix!(ln) {
                res_prefix_msg!("long name", ln);
                None
            } else {
                Some(ln.to_owned())
            }
        });

        let sn = opt.short.as_ref().and_then ( |sn| {
            if RESERVED_OPT_SHORT_NAMES.contains(sn) {
                res_opt_sn_msg!("short name", sn);
                None
            } else {
                Some(*sn)
            }
        });

        Self {
            name: opt.name.to_owned(),
            help: opt.help.to_owned(),
            takes_value: opt.takes_value,
            multiple: opt.multiple,
            required: opt.required,
            value_name: opt.value_name.to_owned(),
            use_delimiter: opt.use_delimiter,
            short_aliases: {
                let mut snas: HashMap<&char, usize> = HashMap::new();
                opt.short_aliases.as_ref().map( |sns| sns.iter().enumerate().filter_map( |(i, sna)| {
                    if RESERVED_OPT_SHORT_NAMES.contains(sna) {
                        res_opt_sn_msg!("short name alias", sna);
                        return None;
                    } else if sn.is_some() && (sn.as_ref().unwrap() == sna) {
                        gen_err!("specifies short name alias '{}' but it conflicts with the option's short name", sna);
                        return None;
                    } else if let Some(idx) = snas.get(sna) {
                        gen_err!("repeats short name alias '{}' (first occurrence at index {})", sna, idx);
                        return None;
                    }
                    snas.insert(sna, i);
                    Some(*sna)
                }).collect())
            },
            short: sn,
            long_aliases: {
                let mut lnas: HashMap<&String, usize> = HashMap::new();
                opt.long_aliases.as_ref().map( |lns| lns.iter().enumerate().filter_map( |(i, lna)| {
                    if RESERVED_OPT_NAMES.contains(&lna.as_str()) {
                        res_opt_ln_msg!("long name alias", lna);
                        return None
                    } else if uses_reserved_prefix!(lna) {
                        res_prefix_msg!("long name alias", lna);
                        return None
                    } else if ln.is_some() && (ln.as_ref().unwrap() == lna) {
                        gen_err!("specifies long name alias '{}' but it conflicts with the option's long name", lna);
                        return None
                    } else if (&opt.name == lna) && ln.is_none() {
                        gen_err!("specifies long name alias '{}' but it conflicts with the option's inferred long name. If this is intentional, please set this as the option's long name", lna);
                        return None
                    } else if let Some(idx) = lnas.get(lna) {
                        gen_err!("repeats long name alias '{}' (first occurrence at index {})", lna, idx);
                        return None
                    }
                    lnas.insert(lna, i);
                    Some(lna.to_owned())
                }).collect())
            },
            long: ln,
            hidden: opt.hidden,
            upcased_name: {
                if opt.value_name.is_some() {
                    None
                } else {
                    Some(opt.name.to_uppercase())
                }
            },
            full_name: None,
        }
    }

    pub fn id(&self) -> &str {
        if let Some(fname) = self.full_name.as_ref() {
            fname.as_str()
        } else {
            self.name.as_str()
        }
    }
}

pub (crate) fn build_commands<'a, F, G, H>(cmd_def: &'a Command, exts: &G, cmd_container: &F, apply_helps: &H) -> App<'a>
where
    F: Fn(&str) -> &'a Command,
    G: Fn(&str, App<'a>, &mut CmdOptCache) -> App<'a>,
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

    let mut cache = CmdOptCache::new(cmd_def.parent.to_string());
    if let Some(opts) = cmd_def.opts.as_ref() {
        cmd = apply_opts(opts, cmd, &mut cache, None);
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
    cmd = exts(&cmd_def.full_name, cmd, &mut cache);
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

pub (crate) fn apply_opts<'a>(opts: &'a Vec<Opt>, mut cmd: App<'a>, cache: &mut CmdOptCache, from_ext: Option<&Extension>) -> App<'a> {
    for opt_def in opts {
        cache.register(&opt_def.name, from_ext);
        let mut opt = clap::Arg::new(opt_def.id()).action(CountArgs).help(opt_def.help.as_str());

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
            if cache.ln_conflicts(ln) {
                // long name clashes - try inferred long name
                if !cache.iln_conflicts(&opt_def.name) {
                    opt = opt.long(&opt_def.name);
                }
            } else {
                opt = opt.long(ln);
            }
        } else {
            if !opt_def.short.is_some() {
                if !cache.iln_conflicts(&opt_def.name) {
                    opt = opt.long(&opt_def.name);
                }
            }
        }
        if let Some(sn) = opt_def.short {
            if !cache.sn_conflicts(sn) {
                opt = opt.short(sn);
            } else {
                if !opt.get_long().is_some() {
                    if !cache.iln_conflicts(&opt_def.name) {
                        opt = opt.long(&opt_def.name);
                    }
                }
            }
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
            let v;
            v = cache.non_conflicting_lnas(lns);
            opt = opt.visible_aliases(&v[..]);
        }

        if let Some(sns) = opt_def.short_aliases.as_ref() {
            let to_add;
            to_add = cache.non_conflicting_snas(sns);
            opt = opt.visible_short_aliases(&to_add[..]);
        }

        if from_ext.is_some() {
            let full_name = opt_def.full_name.as_ref().unwrap();
            if cache.needs_visible_full_name() {
                opt = opt.long(full_name);
            } else {
                opt = opt.alias(full_name.as_str());
            }
        } else {
            if cache.needs_visible_full_name() {
                log_err_processing_cmd!(
                    cache.cmd_path,
                    "Unable to place unique long name, short name, or inferred long name for command option '{}'. Please resolve any previous conflicts regarding this option or add/update this option's name, long name, or short name",
                    opt_def.name
                );
                continue;
            }
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
pub const VERBOSITY_OPT_NAME: &str = "verbose";
pub const VERBOSITY_OPT_SHORT_NAME: char = 'v';
pub const VERBOSITY_OPT_LNA: &str = "verbosity";
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
    VERBOSITY_OPT_NAME, VERBOSITY_OPT_LNA,
];

pub const RESERVED_OPT_SHORT_NAMES: &[char] = &[
    HELP_OPT_SHORT_NAME, TARGET_OPT_SN, VERBOSITY_OPT_SHORT_NAME
];

static VERBOSITY_HELP_STR: &str = "Terminal verbosity level e.g. -v, -vv, -vvv";
static VERBOSITY_KEYWORD_HELP_STR: &str = "Keywords for verbose listeners";

pub const VOV_OPT_NAME: &str = "version_or_verbosity";

pub fn add_verbosity_opts<'a>(cmd: ClapCommand<'a>, split_v: bool) -> ClapCommand<'a> {
    if split_v {
        cmd.arg(
            ClapArg::new(VERBOSITY_OPT_NAME)
            .long(VERBOSITY_OPT_NAME)
            .visible_alias(VERBOSITY_OPT_LNA)
            .action(CountArgs)
        )
        .arg(
            ClapArg::new(VOV_OPT_NAME)
            .short(VERBOSITY_OPT_SHORT_NAME)
            .action(CountArgs)
        )
    } else {
        cmd.arg(
            ClapArg::new(VERBOSITY_OPT_NAME)
            .long(VERBOSITY_OPT_NAME)
            .visible_alias(VERBOSITY_OPT_LNA)
            .short(VERBOSITY_OPT_SHORT_NAME)
            .action(CountArgs)
            .global(true)
            .help(VERBOSITY_HELP_STR),
        )
    }
    .arg(
        ClapArg::new(VERBOSITY_KEYWORDS_OPT_NAME)
        .long(VERBOSITY_KEYWORDS_OPT_NAME)
        .visible_alias(VERBOSITY_KEYWORDS_OPT_LONG_NAME)
        .multiple(true)
        .action(AppendArgs)
        .global(true)
        .help(VERBOSITY_KEYWORD_HELP_STR)
        .number_of_values(1)
        .use_value_delimiter(true)
    )
}

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

#[macro_export]
macro_rules! output_dir_opt {
    () => {{
        Arg::new("output_dir")
            .short('o')
            .long("output-dir")
            .visible_alias("output_dir")
            .help("Override the default output directory (<APP ROOT>/output)")
            .action(SetArg)
            .value_name("OUTPUT_DIR")
    }}
}

pub const REF_DIR_OPT_LNAS: &[&str] = &["reference_dir", "ref_dir", "reference-dir"];

#[macro_export]
macro_rules! ref_dir_opt {
    () => {{
        Arg::new("reference_dir")
            .short('r')
            .long("ref-dir")
            .visible_aliases(&crate::framework::REF_DIR_OPT_LNAS)
            .help("Override the default reference directory (<APP ROOT>/.ref)")
            .action(SetArg)
            .value_name("REFERENCE_DIR")
    }}
}
