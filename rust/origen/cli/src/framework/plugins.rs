use origen::{Result, ORIGEN_CONFIG};
use crate::{python, CommandHelp};
use std::path::PathBuf;
use indexmap::IndexMap;
// use crate::app_commands::{CommandsToml};
// use crate::app_commands::Command as AppCommand;
use std::fs;
use clap::ArgMatches;
use std::collections::HashMap;
use crate::commands::_prelude::*;
use std::process::exit;
use super::{ClapCommand, Command, CommandsToml, CommandTOML, Extensions, Arg, build_path};
use crate::commands::launch_as;

pub const PL_MGR_CMD_NAME: &'static str = "plugins";
pub const PL_CMD_NAME: &'static str = "plugin";
pub const PL_MGR_LIST_CMD: [&'static str; 2] = [PL_MGR_CMD_NAME, "list"];


pub fn run_pl_mgr(cmd: RunInput, plugins: Option<&Plugins>) -> Result<()> {
    if let Some(subcmd) = cmd.subcommand() {
        let sub = subcmd.1;
        match subcmd.0 {
            "list" => {
                if let Some(pls) = plugins {
                    displayln!("Available plugins:\n");
                    for (name, pl) in pls.plugins.iter() {
                        displayln!("{}", name);
                    }
                } else {
                    displayln!("No plugins available!");
                }
            },
            _ => unreachable!()
        }
    }
    Ok(())
}

pub fn run_pl(cmd: RunInput, mut app: &clap::App, exts: &crate::Extensions, plugins: Option<&Plugins>) -> Result<()> {
    if let Some(subcmd) = cmd.subcommand() {
        let sub = subcmd.1;
        plugins.unwrap().plugins.get(subcmd.0).unwrap().dispatch(sub, app, exts, plugins)
    } else {
        Ok(())
    }
}

pub (crate) fn add_helps(helps: &mut CmdHelps, plugins: Option<&Plugins>) {
    helps.add_core_cmd(PL_MGR_CMD_NAME).set_help_msg("Interface with the Origen plugin manager");
    helps.add_core_sub_cmd(&PL_MGR_LIST_CMD).set_help_msg("List the available plugins");

    helps.add_core_cmd(PL_CMD_NAME).set_help_msg("Access added commands from individual plugins");
    if let Some(pls) = plugins {
        for (pl_name, pl) in pls.plugins.iter() {
            for (n, c) in pl.commands.iter() {
                helps.add_pl_cmd(pl_name, n).set_help_msg(&c.help);
            }
        }
    }
}

// cmd_helps: &'a mut crate::CmdHelps, 
pub (crate) fn add_commands<'a>(app: App<'a>, helps: &'a CmdHelps, plugins: Option<&'a Plugins>, exts: &'a Extensions) -> Result<App<'a>> {
    // let help = "Interface with the Origen plugin manager";
    // helps.add_core_cmd(PL_MGR_CMD_NAME.to_string()).set_help_msg("Interface with the Origen plugin manager");
    // origen_commands.push(CommandHelp {
    //     name: PL_MGR_CMD_NAME.to_string(),
    //     help: help.to_string(),
    //     shortcut: None,
    // });
    let updated = app.subcommand(
        helps.apply_core_cmd_helps(
            PL_MGR_CMD_NAME,
            ClapCommand::new(PL_MGR_CMD_NAME)
                // .about(help)
                .visible_alias("pl_mgr")
                .visible_alias("pls")
                .arg_required_else_help(true)
                .subcommand(
                    helps.core_subc(&PL_MGR_LIST_CMD)
                        .visible_alias("ls")
                    // ClapCommand::new("list")
                    //     .about("List the available plugins")
                    //     .visible_alias("ls")
                        // .arg(
                        //     Arg::new("all")
                        //         .help("Set the password for all datasets")
                        //         .takes_value(false)
                        //         .required(false)
                        //         .long("all")
                        //         .short('a'),
                        // )
                )
        )
    );

    // let help = "Access added commands from individual plugins";
    // origen_commands.push(CommandHelp {
    //     name: PL_CMD_NAME.to_string(),
    //     help: help.to_string(),
    //     shortcut: None,
    // });
    let mut pl_sub = ClapCommand::new(PL_CMD_NAME)
        // .about(help)
        .visible_alias("pl");
    pl_sub = helps.apply_core_cmd_helps(PL_CMD_NAME, pl_sub);

    if let Some(pls) = plugins {
        for (pl_name, pl) in pls.plugins.iter() {
            let mut pl_sub_sub = ClapCommand::new(pl_name.as_str()).setting(AppSettings::ArgRequiredElseHelp);
            // if let Some(pl_cmds) = pl.commands {
                // for (n, c) in pl_cmds {
                // for (n, c) in pl.commands.iter() {
                for n in pl.top_commands.iter() {
                        // pl_sub_sub = pl_sub_sub.subcommand(crate::build_pl_commands(&c, &pls));
                    pl_sub_sub = pl_sub_sub.subcommand(
                        super::build_commands(
                            // c, // &cmds.commands.get(top_cmd_name).unwrap(),
                            &pl.commands.get(n).unwrap(),
                            // cmd_helps,
                            &|cmd, app| {
                                // println!("cmd... {}", cmd);
                                // println!("pl name.. {}", pl_name);
                                exts.apply_to_pl_cmd(&pl_name, cmd, app)
                            },
                            &|cmd| {
                                // let split = cmd.split_once('.').unwrap();
                                // println!("S: {:?}", split);
                                pls.plugins.get(pl_name).unwrap().commands.get(cmd).unwrap()
                                // pls.plugins.get(split.0).unwrap().commands.get(split.1).unwrap()
                            },
                            &|cmd, app| {
                                // let split = cmd.split_once('.').unwrap();
                                helps.apply_helps(&CmdSrc::Plugin(pl_name.to_string(), cmd.to_string()), app)
                            }
                        )
                    );
//         for c in subcommands {
//             // let subcmd = build_command(&c);
//             let split = c.split_once('.').unwrap();
//             let subcmd = build_pl_commands(plugins.plugins.get(split.0).unwrap().commands.get(split.1).unwrap(), plugins);
//             cmd = cmd.subcommand(subcmd);
//         }

                }
                pl_sub = pl_sub.subcommand(pl_sub_sub);
            // }
        }
    }
    let updated = updated.subcommand(pl_sub);
    // let updated = app.subcommand(
    //     SubCommand::with_name(PL_CMD_NAME)
    //         .about(help)
    //         // .setting(AppSettings::ArgRequiredElseHelp)
    //         .visible_alias("pl")
    //         // .arg(
    //         //     Arg::new("code")
    //         //         .help("Set the password for all datasets")
    //         //         .takes_value(true)
    //         //         .value_name("CODE")
    //         //         .multiple(true)
    //         //         .required(true)
    //         // )
    // );

    Ok(updated)
}

pub struct Plugins {
    pub plugins: IndexMap<String, Plugin>
    // pub commands: HashMap<String, (Command, CommandHelp)>
}

impl Plugins {
    pub fn new(exts: &mut Extensions) -> Result<Option<Self>> {
        if ORIGEN_CONFIG.should_collect_plugins() {
            let mut slf = Self {
                plugins: IndexMap::new(),
            };

            python::run_with_callbacks(
                "import _origen; _origen._display_plugin_roots()",
                Some(&mut |line| {
                    if let Some((status, result)) = line.split_once('|') {
                        match status {
                            "success" => {
                                if let Some((name, path)) = result.split_once('|') {
                                    //let pl_config = PluginConfig::from_path(path)
                                    // pl_cmds.insert(name, PathBuf::from(path));
                                    match Plugin::new(name, PathBuf::from(path), exts) {
                                        Ok(pl) => {
                                            slf.plugins.insert(name.to_string(), pl);
                                        },
                                        Err(e) => {
                                            log_error!("{}", e);
                                            log_error!("Unable to collect plugin {}", path);
                                        }
                                    }
                                } else {
                                    log_error!("Malformed output when collecting plugin roots (post status): {}", result)
                                }
                            },
                            _ => log_error!("Unknown status when collecting plugin roots: {}", status)
                        }
                    } else {
                        log_error!("Malformed output encountered when collecting plugin roots: {}", line);
                    }
                    // output_lines += &format!("{}\n", line);
                    // println!("{}", line);
                }),
                None,
            )?;
            Ok(Some(slf))
        } else {
            Ok(None)
        }
    }

    pub fn find_command(&self, mut matches: &ArgMatches) -> Option<&Command> {
        for (n, pl) in self.plugins.iter() {
            if let Some(cmd) = pl.find_command(matches) {
                return Some(cmd);
            }
        }
        None
    }

    // pub fn dispatch(&self, mut matches: &ArgMatches) {
    // }
}

pub struct Plugin {
    pub name: String,
    pub root: PathBuf,
    // TODO see about making this indices instead of duplicating string
    pub top_commands: Vec<String>,
    // pub command_helps: Vec<CommandHelp>,
    // pub commands: Vec<Command>,
    pub commands: IndexMap::<String, Command>,
    // pub extensions: Vec<Extension>,
}

impl Plugin {
    fn _add_cmd(slf: &mut Self, current_path: String, current_cmd: &mut CommandTOML) {
        // let mut sub_commands: Option<Vec<String>>;
        // {
        //     build_upcase_names(current_cmd);
        // }
        if let Some(ref mut sub_cmds) = current_cmd.subcommand {
            // sub_commands = Some(vec![]);
            for mut sub in sub_cmds {
                // sub_commands.as_mut().push(sub.name.to_string());
                Self::_add_cmd(slf, format!("{}.{}", current_path, &sub.name), &mut sub);
            }
        } else {
            // sub_commands = None;
        }
        // println!("PATH: {}", current_path);
        slf.commands.insert(current_path.clone(), Command::from_toml_cmd(current_cmd, &current_path));
    }

    pub fn new(name: &str, path: PathBuf, exts: &mut Extensions) -> Result<Self> {
        // fn add_command(current_cmd, current_path, plugins) {
        //     plugins.insert()
        // }

        let mut slf = Self {
            name: name.to_string(),
            root: path,
            top_commands: vec!(),
            commands: IndexMap::new(),
            // extensions: vec!(),
            // command_helps: vec![],
            // commands: vec![],
        };

        let commands_toml = slf.root.join("commands.toml");

        if commands_toml.exists() {
            let content = match fs::read_to_string(&commands_toml) {
                Ok(x) => x,
                Err(e) => {
                    bail!("{}", e);
                }
            };

            let command_config: CommandsToml = match toml::from_str(&content) {
                Ok(x) => x,
                Err(e) => {
                    bail!("Malformed commands.toml: {}", e);
                }
            };

            if let Some(mut commands) = command_config.command {
                for mut cmd in commands {
                    slf.top_commands.push(cmd.name.to_owned());
                    Self::_add_cmd(&mut slf, cmd.name.to_owned(), &mut cmd);
                }
            //     for mut command in commands {
            //         slf.command_helps.push(CommandHelp {
            //             name: command.name.clone(),
            //             help: command.help.clone(),
            //             shortcut: command.alias.clone(),
            //         });
            //         build_upcase_names(&mut command);
            //         slf.commands.push(command);
            //     }
            }

            if let Some(mut extensions) = command_config.extension {
                for ext in extensions {
                    match exts.add_from_pl_toml(&slf, ext) {
                        Ok(_) => {},
                        Err(e) => log_error!("Failed to add extensions from plugin '{}': {}", slf.name, e)
                    }
                    // slf.extensions.push(Extension::from_extension_toml(ExtensionSource::Plugin(slf.name.to_string()), ext)?);
                }
            }
        }

        Ok(slf)
    }

    pub fn command_helps(&self) -> Vec<CommandHelp> {
        let mut helps = vec!();
        for (_, cmd) in self.commands.iter() {
            helps.push(CommandHelp {
                name: cmd.name.clone(),
                help: cmd.help.clone(),
                shortcut: cmd.alias.clone(),
            });
        }
        helps
    }

    // pub fn with_command(&self) -> Option<Vec<&Command>> {
        
    // }

    pub fn get_mut_command(&mut self, path: &str) -> Option<&mut Command> {
        todo!();
        // let retn_cmd = None;
        // let split = path.split('.').collect::<Vec<&str>>();
        // if let Some(mut cmd) = self.commands.get_mut(split.pop().unwrap()) {
        //     while true {
        //         if let Some(s) = split.pop() {
        //             if let Some(c) = cmd.subcommand.as_ref().unwrap().iter_mut().find(|c| c.name == s) {
        //                 cmd = c;
        //                 retn_cmd = Some(c);
        //             } else {
        //                 return None;
        //             }
        //         } else {
        //             break;
        //         }
        //     }
        // }
        // retn_cmd
    }

    pub fn find_command(&self, mut matches: &ArgMatches) -> Option<&Command> {
        todo!();
        // let mut commands: Vec<&Command> = vec![];
        // let mut given_args: HashMap<String, Vec<String>> = HashMap::new();
        // let mut name;
        // let mut command: Option<&Command> = None;

        // while matches.subcommand_name().is_some() {
        //     // Don't need to worry about not finding here, clap has already pre-screened the given values
        //     name = matches.subcommand_name().unwrap();

        //     if let Some(cmd) = command {
        //         command = cmd
        //             .subcommand
        //             .as_ref()
        //             .unwrap()
        //             .iter()
        //             .find(|c| c.name == name);
        //     } else {
        //         command = self.commands.iter().find(|c| c.name == name);
        //     }

        //     // matches = matches.subcommand_matches(&name).unwrap();

        //     if let Some(cmd) = command {
        //         // if let Some(args) = &cmd.arg {
        //         //     for arg in args {
        //         //         if arg.multiple.is_some() && arg.multiple.unwrap() {
        //         //             if let Some(v) = matches.values_of(&arg.name) {
        //         //                 let vals: Vec<String> = v.map(|v| v.to_string()).collect();
        //         //                 given_args.insert(arg.name.to_string(), vals);
        //         //             }
        //         //         } else {
        //         //             if let Some(v) = matches.value_of(&arg.name) {
        //         //                 given_args.insert(arg.name.to_string(), vec![v.to_string()]);
        //         //             }
        //         //         }
        //         //     }
        //         // }
        //         commands.push(cmd);
        //     }

        //     // commands.push(name.to_string());
        // }
        // commands.last().map( |c| *c)
        // // if commands.is_empty() {
        // //     None
        // // } else {
        // //     Some(commands.last())
        // // }
    }

    pub fn dispatch(&self, cmd: &clap::ArgMatches, mut app: &clap::App, exts: &crate::Extensions, plugins: Option<&crate::Plugins>) -> Result<()> {
        if let Some(subc) = cmd.subcommand() {
            let path = build_path(&cmd)?;

            let mut matches = cmd;
            let mut path_pieces: Vec<String> = vec!();
            let mut overrides = IndexMap::new();
            app = app.find_subcommand("plugin").unwrap().find_subcommand(&self.name).unwrap();
            while matches.subcommand_name().is_some() {
                let n = matches.subcommand_name().unwrap();
                matches = matches.subcommand_matches(&n).unwrap();
                app = app.find_subcommand(n).unwrap();
                // path_pieces.push(format!("r'{}'", n));
                path_pieces.push(n.to_string());
            }
            // println!("dis: {} {}", subc.0, &path);

            launch_as("_plugin_dispatch_", Some(&path_pieces), matches, app, exts.get_pl_ext(&self.name, &path), plugins, Some(
                {
                    overrides.insert("dispatch_root".to_string(), Some(format!("r'{}/commands'", &self.root.display())));
                    overrides.insert("dispatch_src".to_string(), Some(format!("r'{}'", &self.name)));
                    // overrides.insert("dispatch_cmds".to_string(), Some(format!("[r'{}', {}]", &self.name, path_pieces.join(", "))));
                    overrides
                }
            ), None);

            Ok(())
        } else {
            // FOR_PR get message from aux/app
            todo!()
        }

        // let mut commands: Vec<String> = vec![];
        // let mut given_args: HashMap<String, Vec<String>> = HashMap::new();
        // let mut name;
        // let mut path: String = "".to_string();
        // let mut current_cmd: Option<&Command> = None;
        // let mut args_str: String = "".to_string();

        // while matches.subcommand_name().is_some() {
        //     name = matches.subcommand_name().unwrap();
        //     if path.is_empty() {
        //         path = name.to_string();
        //     } else {
        //         path = format!("{}.{}", path, name);
        //     }

        //     // if let Some(cmd) = current_cmd {
        //     //     current_cmd = 
        //     // }

        //     matches = matches.subcommand_matches(&name).unwrap();
        //     if let Some(cmd) = self.commands.get(&path) {
        //         // println!("Found command at {}", path);
        //         // if let Some(args) = &cmd.arg {
        //         //     for arg in args {
        //         //         if arg.multiple.is_some() && arg.multiple.unwrap() {
        //         //             if let Some(v) = matches.values_of(&arg.name) {
        //         //                 let vals: Vec<String> = v.map(|v| v.to_string()).collect();
        //         //                 given_args.insert(arg.name.to_string(), vals);
        //         //             }
        //         //         } else {
        //         //             if let Some(v) = matches.value_of(&arg.name) {
        //         //                 given_args.insert(arg.name.to_string(), vec![v.to_string()]);
        //         //             }
        //         //         }
        //         //     }
        //         // }
        //         if let Some(args) = &cmd.args {
        //             for arg in args {
        //                 if arg.multiple.is_some() && arg.multiple.unwrap() {
        //                     if let Some(v) = matches.values_of(&arg.name) {
        //                         // let vals: Vec<String> = v.map(|v| v.to_string()).collect();
        //                         // given_args.insert(arg.name.to_string(), vals);
        //                         args_str += &format!(", r'{}': [{}]", &arg.name, v.map(|v| format!("r'{}'", v)).collect::<Vec<String>>().join(","));
        //                     }
        //                 } else {
        //                     if let Some(v) = matches.value_of(&arg.name) {
        //                         // given_args.insert(arg.name.to_string(), vec![v.to_string()]);
        //                         args_str += &format!(", r'{}': r'{}'", &arg.name, v);
        //                     } else if matches.contains_id(&arg.name) {
        //                         args_str += &format!(", r'{}': True", &arg.name);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     commands.push(name.to_string());
        // }
        // // println!("Final command {:?}", commands);

        // // Build the dispatch command
        // let mut cmd = "from origen.boot import run_cmd; run_cmd('_plugin_dispatch_', ".to_string();

        // cmd += &format!(
        //     "commands=[r'{}', {}]",
        //     &self.name,
        //     &commands
        //         .iter()
        //         .map(|s| format!("r'{}'", s))
        //         .collect::<Vec<String>>()
        //         .join(",")
        // );

        // cmd += ", args={";
        // if !args_str.is_empty() {
        //     cmd += &args_str[2..args_str.len()];
        // }
        // // let mut first = true;
        // // for (k, v) in given_args {
        // //     if !first {
        // //         cmd += ", ";
        // //     }
        // //     cmd += &format!(
        // //         "r'{}': [{}]",
        // //         &k,
        // //         v.iter()
        // //             .map(|s| format!("r'{}'", s))
        // //             .collect::<Vec<String>>()
        // //             .join(",")
        // //     );
        // //     first = false;
        // // }
        // cmd += "});";

        // log_debug!("Launching Python: '{}'", &cmd);
        // println!("Launching Python: '{}'", &cmd);

        // match python::run(&cmd) {
        //     Err(e) => {
        //         log_error!("{}", &e);
        //         exit(1);
        //     }
        //     Ok(exit_status) => {
        //         if exit_status.success() {
        //             exit(0);
        //         } else {
        //             exit(exit_status.code().unwrap_or(1));
        //         }
        //     }
        // }

        // // Unnecessary with exists
        // Ok(())
    }
}

// #[derive(Debug, Deserialize)]
// pub struct PluginConfigTOML {
//     pub commands: Option<Vec<AppCommand>>,
// }

// pub(crate) fn collect_plugin_commands(pl_cmds: &mut IndexMap<String, Vec<CommandHelp>>) -> Result<()> {
//     if ORIGEN_CONFIG.should_collect_plugins() {
//         python::run_with_callbacks(
//             "import _origen; _origen._plugin_roots()",
//             Some(&mut |line| {
//                 if let Some((status, result)) = line.split_once('|') {
//                     match status {
//                         "success" => {
//                             if let Some((name, path)) = result.split_once('|') {
//                                 //let pl_config = PluginConfig::from_path(path)
//                                 // pl_cmds.insert(name, PathBuf::from(path));
//                                 match Plugin::new(name, PathBuf::from(path)) {
//                                     Ok(pl) => self.plugins.insert(name.to_string(), pl),
//                                     Err(e) => {
//                                         log_error!("{}", e);
//                                         log_error!("Unable to collect plugin {}", path)
//                                     }
//                                 }
//                             } else {
//                                 log_error!("Malformed out when collecting plugin roots (post status): {}", result)
//                             }
//                         },
//                         _ => log_error!("Unknown status when collecting plugin roots: {}", status)
//                     }
//                 } else {
//                     log_error!("Malformed output encountered when collecting plugin roots: {}", line);
//                 }
//                 // output_lines += &format!("{}\n", line);
//                 // println!("{}", line);
//             }),
//             None,
//         )?;
//     }
//     Ok(())
// }

// if let Some(pl_config) = origen::CONFIG.plugins.as_ref() {
//     python::run_with_callbacks("import _origen; _origen._plugin_roots()",
//         Some(&mut |line| {
//             // output_lines += &format!("{}\n", line);
//             println!("{}", line);
//         }),
//         None,
//     )?;
//     if pl_config.discover_plugins() {
//         python::get_plugin_roots()?;
//     }
// }
