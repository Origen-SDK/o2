use origen::{Result, ORIGEN_CONFIG, origen_config_metadata};
use crate::commands::_prelude::*;
// use crate::app_commands::Command as CommandTOML;

use super::{Command, Arg, build_commands};

// use crate::app_commands::Command as AppCommand;
use std::fs;
use std::path::PathBuf;
use origen::core::config::AuxillaryCommandsTOML;
use super::extensions::ExtensionTOML;
use super::{CommandTOML};

use clap;

pub const CMD_NAME: &'static str = "auxillary_commands";

// pub struct AuxCmdsTOML {
//     Vec<AuxCmd
// }

#[derive(Debug, Deserialize)]
pub (crate) struct CommandsToml {
    pub help: Option<String>,
    pub command: Option<Vec<CommandTOML>>,
    pub extension: Option<Vec<ExtensionTOML>>
}

#[derive(Default)]
pub struct AuxCmds {
    pub namespaces: IndexMap<String, AuxCmdNamespace>,
}

impl AuxCmds {
    pub fn new(exts: &mut Extensions) -> Result<Self> {
        let mut slf = Self::default();
        if let Some(aux_cmds_configs) = ORIGEN_CONFIG.auxillary_commands.as_ref() {
            for (i, config) in aux_cmds_configs.iter().enumerate() {
                match AuxCmdNamespace::new(i, config, exts) {
                    Ok(aux_ns) => {
                        let ns = aux_ns.namespace().to_string();
                        if let Some(existing_ns) = slf.namespaces.get(&ns) {
                            log_error!("Auxillary commands namespaced '{}' already exists.", ns);
                            log_error!("Cannot add namespace from config '{}'", origen_config_metadata().aux_cmd_sources[i].display());
                            log_error!("Namespace first defined in config '{}'", existing_ns.origin().display());
                        } else {
                            slf.namespaces.insert(ns, aux_ns);
                        }
                    }
                    Err(e) => {
                        log_error!(
                            "Unable to add auxillary commands at '{}' from config '{}'. The following error was met:",
                            config.path().display(),
                            origen_config_metadata().aux_cmd_sources[i].display()
                        );
                        // log_error!("Unable to add auxillary commands from config '{}'. The following error was met:", config.path);
                        log_error!("{}", e);
                    }
                }
            }
        }
        Ok(slf)
    }
}

pub struct AuxCmdNamespace {
    commands: IndexMap<String, Command>,
    pub top_commands: Vec<String>,
    index: usize,
    // path: PathBuf,
    help: Option<String>,
    // name: Option<String>,
}

impl AuxCmdNamespace {
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
        slf.commands.insert(current_path.clone(), Command::from_toml_cmd(current_cmd, &current_path));
    }

    pub fn new(index: usize, config: &AuxillaryCommandsTOML, exts: &mut Extensions) -> Result<Self> {
        let mut slf = Self {
            commands: IndexMap::new(),
            top_commands: vec!(),
            index: index,
            help: None,
        };

        let mut commands_toml = PathBuf::from(&config.path);
        if commands_toml.extension().is_none() {
            commands_toml.set_extension("toml");
        }

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
            slf.help = command_config.help.to_owned();

            if let Some(mut commands) = command_config.command {
                for mut cmd in commands {
                    slf.top_commands.push(cmd.name.to_owned());
                    Self::_add_cmd(&mut slf, cmd.name.to_owned(), &mut cmd);
                }
            }

            // TODO extensions?
            if let Some(mut extensions) = command_config.extension {
                for ext in extensions {
                    match exts.add_from_aux_toml(&slf, ext) {
                        Ok(_) => {},
                        Err(e) => log_error!("Failed to add extensions from aux commands '{}' ({}): {}", slf.namespace(), slf.path().display(), e)
                    }
                }
            }
        } else {
            bail!("Could not find auxillary commands file at '{}'", commands_toml.display());
        }
        Ok(slf)
    }

    pub fn namespace(&self) -> String {
        let config = &ORIGEN_CONFIG.auxillary_commands.as_ref().unwrap()[self.index];
        if let Some(n) = config.name.as_ref() {
            n.to_string()
        } else {
            format!("{}", PathBuf::from(&config.path).file_stem().unwrap().to_str().unwrap())
        }
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(&ORIGEN_CONFIG.auxillary_commands.as_ref().unwrap()[self.index].path)
    }

    pub fn root(&self) -> PathBuf {
        self.path().with_extension("")
    }

    pub fn origin(&self) -> PathBuf {
        origen_config_metadata().aux_cmd_sources[self.index].to_path_buf()
    }
}

pub (crate) fn add_helps(helps: &mut CmdHelps, aux_cmds: &AuxCmds) {
    helps.add_core_cmd(CMD_NAME).set_help_msg("Interface with auxillary commands");
    for (ns, cmds) in aux_cmds.namespaces.iter() {
        for (n, c) in cmds.commands.iter() {
            helps.add_aux_cmd(ns, n).set_help_msg(&c.help);
        }
    }
}

pub (crate) fn add_commands<'a>(app: App<'a>, helps: &'a CmdHelps, aux_commands: &'a AuxCmds, exts: &'a Extensions) -> Result<App<'a>> {
    // let help = "Interface with auxillary commands";
    // origen_commands.push(CommandHelp {
    //     name: CMD_NAME.to_string(),
    //     help: help.to_string(),
    //     shortcut: None,
    // });

    let mut aux_sub = helps.core_cmd(CMD_NAME)
        // .about(help)
        .visible_alias("aux_cmds")
        .arg_required_else_help(true);
        // .setting(AppSettings::ArgRequiredElseHelp);
        // .arg("details")
        // .arg("show_sources")

    for (ns, cmds) in aux_commands.namespaces.iter() {
        let mut aux_sub_sub = clap::Command::new(ns).setting(AppSettings::ArgRequiredElseHelp);
        if let Some(h) = cmds.help.as_ref() {
            aux_sub_sub = aux_sub_sub.about(h.as_str());
        }
        for top_cmd_name in cmds.top_commands.iter() {
            // aux_sub_sub = aux_sub_sub.subcommand(super::build_commands(&c, &pls));
            // aux_sub_sub = aux_sub_sub.subcommand(super::build_commands(&c, Box::new(|namespace, cmd| aux_commands.namespaces.get(namespace).unwrap().commands.get(cmd).unwrap())));
            // let c = cmds.get(top_cmd_name).unwrap();
            // println!("N {}", top_cmd_name);
            // println!("C {}", c.name);
            aux_sub_sub = aux_sub_sub.subcommand(build_commands(
                &cmds.commands.get(top_cmd_name).unwrap(),
                &|cmd, app| {
                    exts.apply_to_aux_cmd(&ns, cmd, app)
                },
                &|cmd| {
                    cmds.commands.get(cmd).unwrap()
                },
                &|cmd, app| {
                    helps.apply_helps(&CmdSrc::Aux(ns.to_string(), cmd.to_string()), app)
                }
            ));
            // aux_sub_sub = aux_sub_sub.subcommand(super::build_commands(&c, aux_commands::temp));
        }
        aux_sub = aux_sub.subcommand(aux_sub_sub);
    }
    Ok(app.subcommand(aux_sub))

    // if let Some(aux_cmds_configs) = ORIGEN_CONFIG.auxillary_commands.as_ref() {
    //     for (aux_cmds_config) in aux_cmds_configs.iter() {
    //         println!("Aux commands from {}", aux_cmds_config.path);

    //         let mut commands_toml = PathBuf::from(aux_cmds);
    //         if commands_toml.extension.is_none() {
    //             commands_toml.push(".toml");
    //         }

    //         if commands_toml.exists() {
    //             let content = match fs::read_to_string(&commands_toml) {
    //                 Ok(x) => x,
    //                 Err(e) => {
    //                     bail!("{}", e);
    //                 }
    //             };
    
    //             let command_config: CommandsToml = match toml::from_str(&content) {
    //                 Ok(x) => x,
    //                 Err(e) => {
    //                     bail!("Malformed commands.toml: {}", e);
    //                 }
    //             };
    
    //             if let Some(mut commands) = command_config.command {
    //                 let aux_namespace = SubCommand::with_name()
    //                 for mut cmd in commands {
    //                     // for (n, c) in pl.commands.iter() {
    //                         aux_namespace = aux_namespace.subcommand(crate::build_command(&cmd));
    //                     // }
    //                     // Self::_add_cmd(&mut slf, cmd.name.to_owned(), &mut cmd);
    //                 }
    //             //     for mut command in commands {
    //             //         slf.command_helps.push(CommandHelp {
    //             //             name: command.name.clone(),
    //             //             help: command.help.clone(),
    //             //             shortcut: command.alias.clone(),
    //             //         });
    //             //         build_upcase_names(&mut command);
    //             //         slf.commands.push(command);
    //             //     }
    //             }
    
    //         //     if let Some(mut extensions) = command_config.extension {
    //         //         for ext in extensions {
    //         //             match exts.add_from_pl_toml(&slf, ext) {
    //         //                 Ok(_) => {},
    //         //                 Err(e) => log_error!("Failed to add extensions from plugin '{}': {}", slf.name, e)
    //         //             }
    //         //             // slf.extensions.push(Extension::from_extension_toml(ExtensionSource::Plugin(slf.name.to_string()), ext)?);
    //         //         }
    //         //     }
    //         } else {
    //             bail!("Could not find auxillary commands file at {}", commands_toml.display());
    //         }
    

    //     }
    // }

    // let updated = app.subcommand(
    //     SubCommand::with_name(CMD_NAME)
    //         // .subcommand(
    //         //     SubCommand::with_name("list")
    //         //         .about("List the available plugins")
    //         //         .visible_alias("ls")
    //         //         // .arg(
    //         //         //     Arg::new("all")
    //         //         //         .help("Set the password for all datasets")
    //         //         //         .takes_value(false)
    //         //         //         .required(false)
    //         //         //         .long("all")
    //         //         //         .short('a'),
    //         //         // )
    //         // )
    // );

    // let help = "Access added commands from individual plugins";
    // origen_commands.push(CommandHelp {
    //     name: PL_CMD_NAME.to_string(),
    //     help: help.to_string(),
    //     shortcut: None,
    // });
    // let mut pl_sub = SubCommand::with_name(PL_CMD_NAME)
    //     .about(help)
    //     .visible_alias("pl");
    // if let Some(pls) = plugins {
    //     for (pl_name, pl) in pls.plugins.iter() {
    //         let mut pl_sub_sub = SubCommand::with_name(pl_name).setting(AppSettings::ArgRequiredElseHelp);
    //         // if let Some(pl_cmds) = pl.commands {
    //             // for (n, c) in pl_cmds {
    //             for (n, c) in pl.commands.iter() {
    //                 pl_sub_sub = pl_sub_sub.subcommand(crate::build_pl_commands(&c, &pls));
    //             }
    //             pl_sub = pl_sub.subcommand(pl_sub_sub);
    //         // }
    //     }
    // }
    // let updated = updated.subcommand(pl_sub);
    // // let updated = app.subcommand(
    // //     SubCommand::with_name(PL_CMD_NAME)
    // //         .about(help)
    // //         // .setting(AppSettings::ArgRequiredElseHelp)
    // //         .visible_alias("pl")
    // //         // .arg(
    // //         //     Arg::new("code")
    // //         //         .help("Set the password for all datasets")
    // //         //         .takes_value(true)
    // //         //         .value_name("CODE")
    // //         //         .multiple(true)
    // //         //         .required(true)
    // //         // )
    // // );

    // Ok(updated)
}