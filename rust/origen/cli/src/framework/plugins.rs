use origen::{Result, ORIGEN_CONFIG};
use crate::python;
use std::path::PathBuf;
use indexmap::IndexMap;
use std::fs;
use crate::commands::_prelude::*;
use super::{ClapCommand, Command, CommandsToml, CommandTOML, Extensions, build_path};
use super::helps::NOT_EXTENDABLE_MSG;

pub (crate) fn add_pl_ns_helps(helps: &mut CmdHelps, plugins: Option<&Plugins>) {
    if let Some(pls) = plugins {
        for (pl_name, pl) in pls.plugins.iter() {
            for (n, c) in pl.commands.iter() {
                helps.add_pl_cmd(pl_name, n).set_help_msg(&c.help);
            }
        }
    }
}

pub (crate) fn add_pl_ns_subcmds<'a>(mut pl_sub: App<'a>, helps: &'a CmdHelps, plugins: &'a Plugins, exts: &'a Extensions) -> Result<App<'a>> {
    for (pl_name, pl) in plugins.plugins.iter() {
        let mut pl_sub_sub = ClapCommand::new(pl_name.as_str()).setting(AppSettings::ArgRequiredElseHelp).after_help(NOT_EXTENDABLE_MSG);
        for n in pl.top_commands.iter() {
            pl_sub_sub = pl_sub_sub.subcommand(
                super::build_commands(
                    &pl.commands.get(n).unwrap(),
                    &|cmd, app, opt_cache| {
                        exts.apply_to_pl_cmd(&pl_name, cmd, app, opt_cache)
                    },
                    &|cmd| {
                        plugins.plugins.get(pl_name).unwrap().commands.get(cmd).unwrap()
                    },
                    &|cmd, app| {
                        helps.apply_helps(&CmdSrc::Plugin(pl_name.to_string(), cmd.to_string()), app)
                    }
                )
            );
        }
        pl_sub = pl_sub.subcommand(pl_sub_sub)
    }
    Ok(pl_sub)
}

pub struct Plugins {
    pub plugins: IndexMap<String, Plugin>
}

impl Plugins {
    pub fn new(exts: &mut Extensions) -> Result<Option<Self>> {
        if ORIGEN_CONFIG.should_collect_plugins() {
            let mut slf = Self {
                plugins: IndexMap::new(),
            };

            python::run_with_callbacks(
                "import _origen; _origen.plugins.display_plugin_roots()",
                Some(&mut |line| {
                    if let Some((status, result)) = line.split_once('|') {
                        match status {
                            "success" => {
                                if let Some((name, path)) = result.split_once('|') {
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
                }),
                None,
            )?;
            Ok(Some(slf))
        } else {
            Ok(None)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

pub struct Plugin {
    pub name: String,
    pub root: PathBuf,
    // TODO see about making this indices instead of duplicating string
    pub top_commands: Vec<String>,
    pub commands: IndexMap::<String, Command>,
}

impl Plugin {
    fn _add_cmd(slf: &mut Self, current_path: String, current_cmd: &mut CommandTOML, parent_cmd: Option<&Command>) -> Result<bool> {
        if let Some(c) = Command::from_toml_cmd(current_cmd, &current_path, CmdSrc::Plugin(slf.name.to_owned(), current_path.to_string()), parent_cmd)? {
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

    pub fn new(name: &str, path: PathBuf, exts: &mut Extensions) -> Result<Self> {
        let mut slf = Self {
            name: name.to_string(),
            root: path,
            top_commands: vec!(),
            commands: IndexMap::new(),
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

            if let Some(commands) = command_config.command {
                for mut cmd in commands {
                    if Self::_add_cmd(&mut slf, cmd.name.to_owned(), &mut cmd, None)? {
                        slf.top_commands.push(cmd.name.to_owned());
                    }
                }
            }

            if let Some(extensions) = command_config.extension {
                for ext in extensions {
                    match exts.add_from_pl_toml(&slf, ext) {
                        Ok(_) => {},
                        Err(e) => log_error!("Failed to add extensions from plugin '{}': {}", slf.name, e)
                    }
                }
            }
        }

        Ok(slf)
    }

    pub fn dispatch(&self, cmd: &clap::ArgMatches, mut app: &clap::App, exts: &crate::Extensions, plugins: Option<&crate::Plugins>) -> Result<()> {
        if cmd.subcommand().is_some() {
            let path = build_path(&cmd)?;

            let mut matches = cmd;
            let mut path_pieces: Vec<String> = vec!();
            let mut overrides = IndexMap::new();
            app = app.find_subcommand("plugin").unwrap().find_subcommand(&self.name).unwrap();
            while matches.subcommand_name().is_some() {
                let n = matches.subcommand_name().unwrap();
                matches = matches.subcommand_matches(&n).unwrap();
                app = app.find_subcommand(n).unwrap();
                path_pieces.push(n.to_string());
            }

            launch_as("_plugin_dispatch_", Some(&path_pieces), matches, app, exts.get_pl_ext(&self.name, &path), plugins, Some(
                {
                    overrides.insert("dispatch_root".to_string(), Some(format!("r'{}/commands'", &self.root.display())));
                    overrides.insert("dispatch_src".to_string(), Some(format!("r'{}'", &self.name)));
                    overrides
                }
            ));

            Ok(())
        } else {
            // This case shouldn't happen as any non-valid command should be
            // caught previously by clap and a non-command invocation should
            // print the help message.
            unreachable!("Expected a plugin name but none was found!");
        }
    }
}
