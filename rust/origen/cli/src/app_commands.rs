//! This module handles the parsing of commands defined by the current application

use crate::python;
use crate::CommandHelp;
use clap::ArgMatches;
use std::collections::HashMap;
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
    pub alias: Option<String>,
    pub arg: Option<Vec<Arg>>,
    pub subcommand: Option<Vec<Command>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Arg {
    pub name: String,
    pub help: String,
    pub short: Option<String>,
    pub long: Option<String>,
    pub takes_value: Option<bool>,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub value_name: Option<String>,
    pub use_delimiter: Option<bool>,
    pub hidden: Option<bool>,
    pub switch: Option<bool>,
    pub upcased_name: Option<String>,
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
                for mut command in commands {
                    self.command_helps.push(CommandHelp {
                        name: command.name.clone(),
                        help: command.help.clone(),
                        shortcut: command.alias.clone(),
                    });
                    build_upcase_names(&mut command);
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

    pub fn dispatch(&self, mut matches: &ArgMatches) {
        let mut commands: Vec<String> = vec![];
        let mut given_args: HashMap<String, Vec<String>> = HashMap::new();
        let mut name;
        let mut command: Option<&Command> = None;

        while matches.subcommand_name().is_some() {
            // Don't need to worry about not finding here, clap has already pre-screened the given values
            name = matches.subcommand_name().unwrap();

            if let Some(cmd) = command {
                command = cmd
                    .subcommand
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|c| c.name == name);
            } else {
                command = self.commands.iter().find(|c| c.name == name);
            }

            matches = matches.subcommand_matches(&name).unwrap();

            if let Some(cmd) = command {
                if let Some(args) = &cmd.arg {
                    for arg in args {
                        if arg.multiple.is_some() && arg.multiple.unwrap() {
                            if let Some(v) = matches.values_of(&arg.name) {
                                let vals: Vec<String> = v.map(|v| v.to_string()).collect();
                                given_args.insert(arg.name.to_string(), vals);
                            }
                        } else {
                            if let Some(v) = matches.value_of(&arg.name) {
                                given_args.insert(arg.name.to_string(), vec![v.to_string()]);
                            }
                        }
                    }
                }
            }

            commands.push(name.to_string());
        }

        let mut cmd = "from origen.boot import __origen__; __origen__('_dispatch_', ".to_string();

        cmd += &format!(
            "commands=[{}]",
            &commands
                .iter()
                .map(|s| format!("r'{}'", s))
                .collect::<Vec<String>>()
                .join(",")
        );

        cmd += ", args={";
        let mut first = true;
        for (k, v) in given_args {
            if !first {
                cmd += ", ";
            }
            cmd += &format!(
                "r'{}': [{}]",
                &k,
                v.iter()
                    .map(|s| format!("r'{}'", s))
                    .collect::<Vec<String>>()
                    .join(",")
            );
            first = false;
        }
        cmd += "});";

        log_debug!("Launching Python: '{}'", &cmd);

        python::run(&cmd);
    }
}

fn build_upcase_names(command: &mut Command) {
    if let Some(args) = &mut command.arg {
        for arg in args {
            arg.upcased_name = Some(arg.name.to_uppercase());
        }
    }
    if let Some(subcommands) = &mut command.subcommand {
        for mut subcmd in subcommands {
            build_upcase_names(&mut subcmd);
        }
    }
}
