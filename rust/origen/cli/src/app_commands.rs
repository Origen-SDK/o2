//! This module handles the parsing and dispatch of commands defined by the current application

use crate::CommandHelp;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct AppCommands {
    root: PathBuf,
    pub command_helps: Vec<CommandHelp>,
    pub commands: Vec<Command>,
}

#[derive(Debug, Deserialize)]
struct CommandsToml {
    command: Option<Vec<Command>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub help: String,
    pub short: Option<String>,
    pub arg: Option<Vec<Arg>>,
    pub subcommand: Option<Vec<Command>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Arg {
    pub name: String,
    pub help: String,
    pub short: Option<String>,
    pub takes_value: Option<bool>,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub value_name: Option<String>,
    pub use_delimiter: Option<bool>,
    pub hidden: Option<bool>,
}

impl AppCommands {
    pub fn new(root: &Path) -> AppCommands {
        AppCommands {
            root: root.to_path_buf(),
            command_helps: vec![],
            commands: vec![],
        }
    }

    /// Parse the commands from config/commands.toml if found.
    /// Will swallow any errors that occur and print an error message, but allowing the CLI
    /// boot to continue.
    pub fn parse_commands(&mut self) {
        let commands_toml = self.root.join("config").join("commands.toml");
        if commands_toml.exists() {
            let content = match fs::read_to_string(&commands_toml) {
                Ok(x) => x,
                Err(e) => {
                    log_error!("{}", e);
                    return;
                }
            };

            let command_config: CommandsToml = match toml::from_str(&content) {
                Ok(x) => x,
                Err(e) => {
                    log_error!("Malformed config/commands.toml: {}", e);
                    return;
                }
            };

            if let Some(commands) = command_config.command {
                for command in commands {
                    self.command_helps.push(CommandHelp {
                        name: command.name.clone(),
                        help: command.help.clone(),
                        shortcut: command.short.clone(),
                    });
                    self.commands.push(command);
                }
            }
        }
    }

    pub fn max_name_width(&self) -> Option<usize> {
        self.command_helps
            .iter()
            .map(|c| c.name.chars().count())
            .max()
    }
}
