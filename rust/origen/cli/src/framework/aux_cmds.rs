use origen::{Result, ORIGEN_CONFIG, origen_config_metadata};
use crate::commands::_prelude::*;
use super::{Command, build_commands};
use std::fs;
use std::path::PathBuf;
use origen::core::config::AuxillaryCommandsTOML;
use super::extensions::ExtensionTOML;
use super::{CommandTOML};
use clap::Command as ClapCommand;
use super::helps::NOT_EXTENDABLE_MSG;

pub (crate) fn add_aux_ns_helps(helps: &mut CmdHelps, aux_cmds: &AuxCmds) {
    for (ns, cmds) in aux_cmds.namespaces.iter() {
        for (n, c) in cmds.commands.iter() {
            helps.add_aux_cmd(ns, n).set_help_msg(&c.help);
        }
    }
}

#[inline]
pub (crate) fn aux_ns_subcmd<'a>(mut aux_sub: App<'a>, helps: &'a CmdHelps, aux_commands: &'a AuxCmds, exts: &'a Extensions) -> Result<App<'a>> {
    for (ns, cmds) in aux_commands.namespaces.iter() {
        let mut aux_sub_sub = ClapCommand::new(ns).arg_required_else_help(true).after_help(NOT_EXTENDABLE_MSG);
        if let Some(h) = cmds.help.as_ref() {
            aux_sub_sub = aux_sub_sub.about(h.as_str());
        }
        for top_cmd_name in cmds.top_commands.iter() {
            aux_sub_sub = aux_sub_sub.subcommand(build_commands(
                &cmds.commands.get(top_cmd_name).unwrap(),
                &|cmd, app, opt_cache| {
                    exts.apply_to_aux_cmd(&ns, cmd, app, opt_cache)
                },
                &|cmd| {
                    cmds.commands.get(cmd).unwrap()
                },
                &|cmd, app| {
                    helps.apply_helps(&CmdSrc::Aux(ns.to_string(), cmd.to_string()), app)
                }
            ));
        }
        aux_sub = aux_sub.subcommand(aux_sub_sub);
    }
    Ok(aux_sub)
}

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
}

impl AuxCmdNamespace {
    fn _add_cmd(slf: &mut Self, current_path: String, current_cmd: &mut CommandTOML, parent_cmd: Option<&Command>) -> Result<bool> {
        if let Some(c) = Command::from_toml_cmd(current_cmd, &current_path, CmdSrc::Aux(slf.namespace(), current_path.to_string()), parent_cmd)? {
            if let Some(ref mut sub_cmds) = current_cmd.subcommand {
                for mut sub in sub_cmds {
                    Self::_add_cmd(slf, format!("{}.{}", current_path, &sub.name), &mut sub, Some(&c))?;
                }
            }
            slf.commands.insert(current_path.clone(), c);
            Ok(true)
        } else {
            Ok(false)
        }
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

            if let Some(commands) = command_config.command {
                for mut cmd in commands {
                    if Self::_add_cmd(&mut slf, cmd.name.to_owned(), &mut cmd, None)? {
                        slf.top_commands.push(cmd.name.to_owned());
                    }
                }
            }

            if let Some(extensions) = command_config.extension {
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
