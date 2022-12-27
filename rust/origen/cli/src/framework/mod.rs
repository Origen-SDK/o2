// FOR_PR clean up this and entire directory
pub mod helps;
pub mod extensions;
pub mod plugins;
pub mod aux_cmds;
pub mod app_cmds;
#[macro_use]
pub mod core_cmds;

pub use extensions::{Extensions, ExtensionTOML};
pub use plugins::{Plugins, Plugin};
pub use aux_cmds::AuxCmds;
pub use app_cmds::AppCmds;
pub use helps::{CmdHelps, CmdHelp, CmdSrc};

use clap::{App};
use clap::Command as ClapCommand;
use origen::Result;
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
    ($toml_opts: expr) => {
        $toml_opts.as_ref()
            .map(|opts| opts.iter()
                .map( |o| crate::framework::Opt::from_toml(o))
                .collect::<Vec<crate::framework::Opt>>())
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
    // pub consolidate_subc_run_funcs: Option<bool>,
    // pub run_func: Option<String>,
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
    // pub consolidate_subc_run_funcs: Option<bool>,
    // pub run_func: Option<String>,
}

impl Command {
    pub fn from_toml_cmd(cmd: &CommandTOML, cmd_path: &str) -> Self {
        Self {
            name: cmd.name.to_owned(),
            help: cmd.help.to_owned(),
            alias: cmd.alias.to_owned(),
            // args: cmd.arg.to_owned(),
            args: from_toml_args!(cmd.arg),
            opts: from_toml_opts!(cmd.opt),
            // opts: cmd.opt.as_ref().map(|opts| opts.iter().map( |o| Opt::from_toml(o)).collect::<Vec<Opt>>()),
            subcommands: cmd.subcommand.as_ref()
                .map(|sub_cmds| sub_cmds.iter()
                    .map(|c| format!("{}.{}", cmd_path, &c.name.to_string()))
                    .collect::<Vec<String>>()
                ),
            full_name: cmd_path.to_string(),
        }
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

impl Opt {
    fn from_toml(opt: &OptTOML) -> Self {
        Self {
            name: opt.name.to_owned(),
            help: opt.help.to_owned(),
            short: opt.short,
            long: opt.long.to_owned(),
            takes_value: opt.takes_value,
            multiple: opt.multiple,
            required: opt.required,
            value_name: opt.value_name.to_owned(),
            use_delimiter: opt.use_delimiter,
            short_aliases: opt.short_aliases.to_owned(),
            long_aliases: opt.long_aliases.to_owned(),
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

// pub (crate) fn build_upcase_names(command: &mut CommandTOML) {
//     if let Some(args) = &mut command.arg {
//         for arg in args {
//             arg.upcased_name = Some(arg.name.to_uppercase());
//         }
//     }
//     if let Some(subcommands) = &mut command.subcommand {
//         for mut subcmd in subcommands {
//             build_upcase_names(&mut subcmd);
//         }
//     }
// }
// helps: &'a mut CmdHelps, 
pub (crate) fn build_commands<'a, F, G, H>(cmd_def: &'a Command, exts: &G, cmd_container: &F, apply_helps: &H) -> App<'a>
where
    F: Fn(&str) -> &'a Command,
    G: Fn(&str, App<'a>) -> App<'a>,
    H: Fn(&str, App<'a>) -> App<'a>
{
    let mut cmd = ClapCommand::new(&cmd_def.name); // .about(cmd_def.help.as_str());
    // TODO need test case for cmd alias
    if cmd_def.alias.is_some() {
        cmd = cmd.visible_alias(cmd_def.alias.as_ref().unwrap().as_str());
    }

    if let Some(args) = cmd_def.args.as_ref() {
        // let mut req_arg_index = 0;
        cmd = apply_args(args, cmd);
        // for arg_def in args.iter() {
        //     let mut arg = clap::Arg::new(arg_def.name.as_str())
        //         .takes_value(true)
        //         .help(arg_def.help.as_str());

        //     if let Some(vn) = arg_def.value_name.as_ref() {
        //         arg = arg.value_name(vn);
        //     } else {
        //         arg = arg.value_name(arg_def.upcased_name.as_ref().unwrap());
        //     }

        //     if let Some(d) = arg_def.use_delimiter {
        //         arg = arg.use_value_delimiter(d);
        //         arg = arg.multiple_values(true);
        //     }
        //     if let Some(m) = arg_def.multiple {
        //         arg = arg.multiple_values(m);
        //     }

        //     if let Some(r) = arg_def.required {
        //         arg = arg.required(r);
        //     }

        //     cmd = cmd.arg(arg);
        // }
    }

    if let Some(opts) = cmd_def.opts.as_ref() {
        // For each arg
        // for j in 0..cmd_def.args.len() {
        cmd = apply_opts(opts, cmd);
        // for opt_def in opts.iter() {
            // let opt_def = &cmd_def.opts.as_ref().unwrap()[j];
            // println!("opt name: {}", opt_def.name);
            // let mut opt = clap::Arg::new(opt_def.name.as_str()).help(opt_def.help.as_str());

            // if let Some(ud) = opt_def.use_delimiter {
            //     opt = opt.use_value_delimiter(ud);
            //     opt = opt.multiple_values(true);
            //     opt = opt.takes_value(true);
            // }
            // if let Some(m) = opt_def.multiple {
            //     opt = opt.multiple(m);
            //     opt = opt.takes_value(true);
            // }
            // if let Some(val_name) = opt_def.value_name.as_ref() {
            //     opt = opt.value_name(val_name);
            //     opt = opt.takes_value(true);
            // }
            // if let Some(tv) = opt_def.takes_value {
            //     opt = opt.takes_value(tv);
            // }

            // if !opt.is_takes_value_set() {
            //     opt = opt.action(clap::ArgAction::Count);
            // }

            // // if let Some(s) = opt_def.stack.or(true) {
            // //     if !arg.is_takes_value_set {
            // //         arg.action(clap::ArgAction::Count);
            // //         // arg = arg.multiple_occurrences(s);
            // //     }
            // // }
            // if let Some(ln) = opt_def.long.as_ref() {
            //     opt = opt.long(ln);
            // } else {
            //     if !opt_def.short.is_some() {
            //         opt = opt.long(&opt_def.name);
            //     }
            // }
            // if let Some(sn) = opt_def.short {
            //     opt = opt.short(sn);
            //     // let chars = n.chars();
            //     // if let Some(c) = chars.next() {
            //     //     arg = arg.short(c);
            //     // }
            //     // if let Some(c) = chars.next() {
            //     //     bail!
            //     // }
            //     // arg = arg.short(opt_def.short.as_ref().unwrap().chars().next().unwrap())
            // }


            // // // If this is an arg without a switch
            // // if arg_def.switch.is_some() && !arg_def.switch.unwrap() {
            // //     // Do nothing?
            // // } else {
            // //     if arg_def.long.is_some() {
            // //         arg = arg.long(&arg_def.long.as_ref().unwrap());
            // //     } else {
            // //         arg = arg.long(&arg_def.name);
            // //     }
            // //     if arg_def.short.is_some() {
            // //         arg = arg.short(arg_def.short.as_ref().unwrap().chars().next().unwrap())
            // //         // arg = arg.short(arg_def.short.as_ref().unwrap())
            // //     }
            // // }

            // if let Some(r) = opt_def.required {
            //     opt = opt.required(r);
            // }

            // if let Some(h) = opt_def.hidden {
            //     opt = opt.hidden(h);
            // }

            // // if opt_def.takes_value.unwrap_or(false) || opt_def.value_name.is_some() || opt_def.multiple.is_some() {
            // if opt.is_takes_value_set() && opt_def.value_name.is_none() {
            //     opt = opt.value_name(opt_def.upcased_name.as_ref().unwrap());
            //     // if let Some(val_name) = opt_def.value_name.as_ref() {
            //     //     opt = opt.value_name(val_name);
            //     // } else {
            //     //     opt = opt.value_name(opt_def.upcased_name.as_ref().unwrap());
            //     // }
            // }

            // if let Some(lns) = opt_def.long_aliases.as_ref() {
            //     let v = lns.iter().map( |s| s.as_str() ).collect::<Vec<&str>>();
            //     opt = opt.visible_aliases(&v[..]);
            // }

            // if let Some(sns) = opt_def.short_aliases.as_ref() {
            //     // let v = sns.iter().map( |s| s.as_str() ).collect::<Vec<&str>>();
            //     opt = opt.visible_short_aliases(&sns[..]);
            // }

            // cmd = cmd.arg(opt);
        // }
    }

    if let Some(subcommands) = &cmd_def.subcommands {
        for c in subcommands {
            // let subcmd = build_command(&c);
            // let split = c.split_once('.').unwrap();
            // let subcmd = build_pl_commands(plugins.plugins.get(split.0).unwrap().commands.get(split.1).unwrap(), plugins);
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
    // cmd = cmd.about(helps.first().as_ref().unwrap().help.as_ref().unwrap().as_str());

    cmd
}

pub (crate) fn apply_args<'a>(args: &'a Vec<Arg>, mut cmd: App<'a>) -> App<'a> {
    for arg_def in args {
        // let arg_def = &cmd_def.arg.as_ref().unwrap()[j];
        // let mut arg = clap::Arg::new(arg_def.name.as_str()).help(arg_def.help.as_str());
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

        // if let Some(ln) = arg_def.long.as_ref() {
        //     arg = arg.long(ln);
        // } else {
        //     arg = arg.long(&arg_def.name);
        // }

        // if arg_def.short.is_some() {
        //     // TODO add error handling
        //     // arg = arg.short(arg_def.short.as_ref().unwrap().chars().next().unwrap())
        //     // arg = arg.short(arg_def.short.as_ref().unwrap())
        // }

        // If this is an arg without a switch
        // if arg_def.switch.is_some() && !arg_def.switch.unwrap() {
        //     // Do nothing?
        // } else {
        //     if arg_def.long.is_some() {
        //         arg = arg.long(&arg_def.long.as_ref().unwrap());
        //     } else {
        //         arg = arg.long(&arg_def.name);
        //     }
        //     if arg_def.short.is_some() {
        //         arg = arg.short(arg_def.short.as_ref().unwrap().chars().next().unwrap())
        //         // arg = arg.short(arg_def.short.as_ref().unwrap())
        //     }
        // }

        // if let Some(tv) = arg_def.takes_value.as_ref() {
        //     arg = arg.takes_value(*tv);
        // }

        // if arg_def.multiple.is_some() {
        //     arg = arg.multiple(arg_def.multiple.unwrap())
        // }
        // if arg_def.required.is_some() {
        //     arg = arg.required(arg_def.required.unwrap());
        // }
        // if arg_def.use_delimiter.is_some() {
        //     arg = arg.use_delimiter(arg_def.use_delimiter.unwrap())
        // }
        // if arg_def.hidden.is_some() {
        //     arg = arg.hidden(arg_def.hidden.unwrap())
        // }

        // if arg_def.takes_value.unwrap_or(false) || arg_def.value_name.is_some() || arg_def.multiple.is_some() {
        //     if arg_def.value_name.is_some() {
        //         arg = arg.value_name(arg_def.value_name.as_ref().unwrap())
        //     } else {
        //         arg = arg.value_name(arg_def.upcased_name.as_ref().unwrap())
        //     }
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
            // opt = opt.takes_value(true);
        }
        if let Some(tv) = opt_def.takes_value {
            if tv {
                // opt = opt.takes_value(tv);
                opt = opt.action(AppendArgs);
            }
        }
        if let Some(ud) = opt_def.use_delimiter {
            if ud {
                opt = opt.use_value_delimiter(ud);
            }
            opt = opt.multiple_values(true);
            opt = opt.action(AppendArgs); // takes_value(true);
        }
        if let Some(m) = opt_def.multiple {
            if m {
                opt = opt.multiple(m).action(AppendArgs);
            } else {
                opt = opt.multiple(m).action(SetArg);
            }
            // opt = opt.takes_value(true);
        }

        // if !opt.is_takes_value_set() {
        //     opt = opt.action(clap::ArgAction::Count);
        // }
        // println!("action {:?} {}", opt.get_action(), opt.get_action().takes_values());
        // if !opt.get_action().takes_values() {
        //     opt = opt.action(CountArgs);
        // }

        // if let Some(s) = opt_def.stack.or(true) {
        //     if !arg.is_takes_value_set {
        //         arg.action(clap::ArgAction::Count);
        //         // arg = arg.multiple_occurrences(s);
        //     }
        // }
        if let Some(ln) = opt_def.long.as_ref() {
            opt = opt.long(ln);
        } else {
            if !opt_def.short.is_some() {
                opt = opt.long(&opt_def.name);
            }
        }
        if let Some(sn) = opt_def.short {
            opt = opt.short(sn);
            // let chars = n.chars();
            // if let Some(c) = chars.next() {
            //     arg = arg.short(c);
            // }
            // if let Some(c) = chars.next() {
            //     bail!
            // }
            // arg = arg.short(opt_def.short.as_ref().unwrap().chars().next().unwrap())
        }


        // // If this is an arg without a switch
        // if arg_def.switch.is_some() && !arg_def.switch.unwrap() {
        //     // Do nothing?
        // } else {
        //     if arg_def.long.is_some() {
        //         arg = arg.long(&arg_def.long.as_ref().unwrap());
        //     } else {
        //         arg = arg.long(&arg_def.name);
        //     }
        //     if arg_def.short.is_some() {
        //         arg = arg.short(arg_def.short.as_ref().unwrap().chars().next().unwrap())
        //         // arg = arg.short(arg_def.short.as_ref().unwrap())
        //     }
        // }

        if let Some(r) = opt_def.required {
            opt = opt.required(r);
        }

        if let Some(h) = opt_def.hidden {
            opt = opt.hidden(h);
        }

        // if opt_def.takes_value.unwrap_or(false) || opt_def.value_name.is_some() || opt_def.multiple.is_some() {
        if opt.get_action().takes_values() && opt_def.value_name.is_none() {
            opt = opt.value_name(opt_def.upcased_name.as_ref().unwrap().as_str());
            // if let Some(val_name) = opt_def.value_name.as_ref() {
            //     opt = opt.value_name(val_name);
            // } else {
            //     opt = opt.value_name(opt_def.upcased_name.as_ref().unwrap());
            // }
        }

        if let Some(lns) = opt_def.long_aliases.as_ref() {
            let v = lns.iter().map( |s| s.as_str() ).collect::<Vec<&str>>();
            opt = opt.visible_aliases(&v[..]);
        }

        if let Some(sns) = opt_def.short_aliases.as_ref() {
            // let v = sns.iter().map( |s| s.as_str() ).collect::<Vec<&str>>();
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
        // if path.is_empty() {
        //     path = name.to_string();
        // } else {
        //     path = format!("{}.{}", path, name);
        // }

        // if let Some(cmd) = current_cmd {
        //     current_cmd = 
        // }

        // if let Some(cmd) = self.commands.get(&path) {
        //     // println!("Found command at {}", path);
        //     // if let Some(args) = &cmd.arg {
        //     //     for arg in args {
        //     //         if arg.multiple.is_some() && arg.multiple.unwrap() {
        //     //             if let Some(v) = matches.values_of(&arg.name) {
        //     //                 let vals: Vec<String> = v.map(|v| v.to_string()).collect();
        //     //                 given_args.insert(arg.name.to_string(), vals);
        //     //             }
        //     //         } else {
        //     //             if let Some(v) = matches.value_of(&arg.name) {
        //     //                 given_args.insert(arg.name.to_string(), vec![v.to_string()]);
        //     //             }
        //     //         }
        //     //     }
        //     // }
        //     if let Some(args) = &cmd.arg {
        //         for arg in args {
        //             if arg.multiple.is_some() && arg.multiple.unwrap() {
        //                 if let Some(v) = matches.values_of(&arg.name) {
        //                     // let vals: Vec<String> = v.map(|v| v.to_string()).collect();
        //                     // given_args.insert(arg.name.to_string(), vals);
        //                     args_str += &format!(", r'{}': [{}]", &arg.name, v.map(|v| format!("r'{}'", v)).collect::<Vec<String>>().join(","));
        //                 }
        //             } else {
        //                 if let Some(v) = matches.value_of(&arg.name) {
        //                     // given_args.insert(arg.name.to_string(), vec![v.to_string()]);
        //                     args_str += &format!(", r'{}': r'{}'", &arg.name, v);
        //                 } else if matches.contains_id(&arg.name) {
        //                     args_str += &format!(", r'{}': True", &arg.name);
        //                 }
        //             }
        //         }
        //     }
        // }
        // commands.push(name.to_string());
    }
    Ok(path_pieces.join("."))
}

// pub fn get_cmd_def<'a>(mut matches: &clap::ArgMatches, mut app: &'a App<'a>) -> &'a App<'a> {
//     while matches.subcommand_name().is_some() {
//         let n = matches.subcommand_name().unwrap();
//         matches = matches.subcommand_matches(&n).unwrap();
//         app = app.find_subcommand(n).unwrap();
//     }
//     app

//     // for (n, pl) in self.plugins.iter() {
//     //     if let Some(cmd) = pl.find_command(matches) {
//     //         return Some(cmd);
//     //     }
//     // }
//     // None
// }
