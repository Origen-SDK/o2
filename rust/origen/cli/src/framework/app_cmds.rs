use origen::{Result, STATUS};
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;
use crate::commands::_prelude::*;
use super::build_commands;
use origen::core::application::Application;

use super::{Command, CommandsToml, CommandTOML, Extensions};

// pub const CMD_NAME: &'static str = "commands";
pub const APP_COMMANDS: [&'static str; 2] = [crate::commands::app::CMD_NAME, "commands"];

pub struct AppCmds {
    pub root: PathBuf,
    pub top_commands: Vec<String>,
    pub commands: IndexMap::<String, Command>,
}

impl AppCmds {
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

    pub fn new(app: &Application, exts: &mut Extensions) -> Result<Self> {
        let mut slf = Self {
            root: app.root.to_owned(),
            top_commands: vec!(),
            commands: IndexMap::new(),
        };

        // let commands_toml = slf.root.join("config").join("commands.toml");
        for commands_toml in app.config().cmd_paths() {
            // if commands_toml.exists() {
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
            // TODO error on help given?
            // slf.help = command_config.help.to_owned();

            if let Some(commands) = command_config.command {
                for mut cmd in commands {
                    slf.top_commands.push(cmd.name.to_owned());
                    Self::_add_cmd(&mut slf, cmd.name.to_owned(), &mut cmd);
                }
            }

            if let Some(extensions) = command_config.extension {
                for ext in extensions {
                    match exts.add_from_app_toml(ext) {
                        Ok(_) => {},
                        Err(e) => log_error!("Failed to add extensions from application from '{}': {}", &commands_toml.display(), e)
                    }
                }
            }
        }
        Ok(slf)
    }

    pub fn cmds_root(&self) -> Result<PathBuf> {
        let mut r = self.root.to_owned();
        r.push(STATUS.app.as_ref().unwrap().name());
        r.push("commands");
        Ok(r)
    }
}

pub (crate) fn add_helps(helps: &mut CmdHelps, app_cmds: &AppCmds) {
    helps.add_core_sub_cmd(&APP_COMMANDS).set_help_msg("Interface with commands added by the application");
    for (n, c) in app_cmds.commands.iter() {
        helps.add_app_cmd(n).set_help_msg(&c.help);
    }
}

pub (crate) fn add_commands<'a>(app: App<'a>, helps: &'a CmdHelps, app_commands: &'a AppCmds, exts: &'a Extensions) -> Result<App<'a>> {
    // let help = "Interface with commands added by the application";
    // origen_commands.push(CommandHelp {
    //     name: CMD_NAME.to_string(),
    //     help: help.to_string(),
    //     shortcut: None,
    // });

    let mut app_cmds_cmd = helps.core_subc(&APP_COMMANDS)
        // ClapCommand::new(CMD_NAME)
        //     .about(help)
            // .setting(AppSettings::ArgRequiredElseHelp)
            .visible_alias("cmds")
            // .setting(AppSettings::ArgRequiredElseHelp)
            .arg_required_else_help(true);
            // .subcommand(
            //     ClapCommand::new("list")
            //         .about("List the available plugins")
            //         .visible_alias("ls")
            // );

    for top_cmd_name in app_commands.top_commands.iter() {
        // app_cmds_cmd = app_cmds_cmd.subcommand(build_commands(cmds).unwrap(), &|cmd| {
        //     cmds.commands.get(cmd).unwrap()
        // });


        app_cmds_cmd = app_cmds_cmd.subcommand(build_commands(
            &app_commands.commands.get(top_cmd_name).unwrap(),
            &|cmd, app| {
                exts.apply_to_app_cmd(cmd, app)
            },
            &|cmd| {
                app_commands.commands.get(cmd).unwrap()
            },
            &|cmd, app| {
                helps.apply_helps(&CmdSrc::App(cmd.to_string()), app)
            },
        ));
    }
    Ok(app.subcommand(app_cmds_cmd))
}
