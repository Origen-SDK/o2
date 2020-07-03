#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate origen;

mod app_commands;
mod commands;
mod python;

use app_commands::AppCommands;
use clap::{App, AppSettings, Arg, SubCommand};
use origen::{LOGGER, STATUS};
use std::path::Path;

#[derive(Clone)]
pub struct CommandHelp {
    name: String,
    help: String,
    shortcut: Option<String>,
}

impl CommandHelp {
    fn render(&self, width: usize) -> String {
        let mut msg = "".to_string();
        msg += &format!("{:width$} {}", self.name, self.help, width = width + 3);
        if let Some(a) = &self.shortcut {
            msg += &format!(" [aliases: {}]", a);
        }
        msg += "\n";
        msg
    }
}

// This is the entry point for the Origen CLI tool
fn main() {
    // Set the verbosity immediately, this is to allow log statements to work in really
    // low level stuff, e.g. when building the STATUS
    let re = regex::Regex::new(r"-([vV]+)").unwrap();
    let mut verbosity: u8 = 0;
    for arg in std::env::args() {
        if let Some(captures) = re.captures(&arg) {
            let x = captures.get(1).unwrap().as_str();
            verbosity = x.chars().count() as u8;
        }
    }
    origen::initialize(Some(verbosity));

    let version = match STATUS.is_app_present {
        true => format!("Origen CLI: {}", STATUS.origen_version.to_string()),
        false => format!("Origen: {}", STATUS.origen_version.to_string()),
    };

    let mut app = App::new("")
        .setting(AppSettings::ArgRequiredElseHelp)
        .before_help("Origen, The Semiconductor Developer's Kit")
        .after_help("See 'origen <command> -h' for more information on a specific command.")
        .version(&*version)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Terminal verbosity level e.g. -v, -vv, -vvv"),
        );

    // The main help message is going to be automatically generated to allow us to handle and clearly
    // separate commands added by the app and plugins.
    // When a command is added below it must also be added to these vectors.
    let mut origen_commands: Vec<CommandHelp> = vec![];
    let mut app_commands: Vec<CommandHelp> = vec![];

    /************************************************************************************/
    /******************** Global only commands ******************************************/
    /************************************************************************************/
    if !STATUS.is_app_present {
        let proj_help = "Manage multi-repository project areas and workspaces";
        origen_commands.push(CommandHelp {
            name: "proj".to_string(),
            help: proj_help.to_string(),
            shortcut: None,
        });

        app = app
            //************************************************************************************/
            .subcommand(
                SubCommand::with_name("proj")
                    .display_order(1)
                    .about(proj_help)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .subcommand(SubCommand::with_name("init")
                        .display_order(5)
                        .about("Initialize a new project directory (create an initial project BOM)")
                        .arg(Arg::with_name("dir")
                            .takes_value(true)
                            .help("The path to the project directory to initialize (PWD will be used by default if not given)")
                            .value_name("DIR")
                        )
                    )
                    .subcommand(SubCommand::with_name("packages")
                        .display_order(7)
                        .about("Displays the IDs of all packages and package groups defined by the BOM")
                    )
                    .subcommand(SubCommand::with_name("create")
                        .display_order(10)
                        .about("Create a new project workspace from the project BOM")
                        .arg(Arg::with_name("path")
                            .help("The path to the new workspace directory")
                            .takes_value(true)
                            .value_name("PATH")
                            .required(true)
                        )
                    )
                    .subcommand(SubCommand::with_name("update")
                        .display_order(15)
                        .about("Update an existing project workspace per its current BOM")
                        .arg(Arg::with_name("force")
                            .short("f")
                            .long("force")
                            .required(false)
                            .takes_value(false)
                            .help("Force the update and potentially lose any local modifications")
                        )
                        .arg(Arg::with_name("links")
                            .short("l")
                            .long("links")
                            .required(false)
                            .takes_value(false)
                            .help("Update the workspace links")
                        )
                        .arg(Arg::with_name("packages")
                            .value_name("PACKAGES")
                            .takes_value(true)
                            .multiple(true)
                            .help("Packages and/or groups to be updated, run 'origen proj packages' to see a list of possible package IDs")
                            .required_unless("links")
                            .required(true)
                        )
                    )
                    .subcommand(SubCommand::with_name("mods")
                        .display_order(20)
                        .about("Display a list of modified files within the given package(s)")
                        .arg(Arg::with_name("packages")
                            .help("Package(s) to look for modifications in, use 'all' to see the modification to all packages")
                            .multiple(true)
                            .value_name("PACKAGES")
                            .required(true)
                        )
                    )
                    .subcommand(SubCommand::with_name("clean")
                        .display_order(20)
                        .about("Revert all local modifications within the given package(s)")
                        .arg(Arg::with_name("packages")
                            .help("Package(s) to revert local modifications in, use 'all' to clean all packages")
                            .multiple(true)
                            .value_name("PACKAGES")
                            .required(true)
                        )
                    )
                    .subcommand(SubCommand::with_name("tag")
                        .display_order(20)
                        .about("Apply the given tag to the current view of the given package(s)")
                        .arg(Arg::with_name("name")
                            .help("Name of the tag to be applied")
                            .takes_value(true)
                            .value_name("NAME")
                            .required(true)
                        )
                        .arg(Arg::with_name("packages")
                            .help("Package(s) to be tagged, use 'all' to tag all packages")
                            .multiple(true)
                            .takes_value(true)
                            .value_name("PACKAGES")
                            .required(true)
                        )
                        .arg(Arg::with_name("force")
                            .short("f")
                            .long("force")
                            .required(false)
                            .takes_value(false)
                            .help("Force the application of the tag even if there are local modifications")
                        )
                        .arg(Arg::with_name("message")
                            .short("m")
                            .long("message")
                            .required(false)
                            .takes_value(true)
                            .help("A message to be applied with the tag")
                        )
                    )
                    .subcommand(SubCommand::with_name("bom")
                        .display_order(25)
                        .about("View the active BOM in the current or given directory")
                        .arg(Arg::with_name("dir")
                            .takes_value(true)
                            .help("The path to a directory (PWD will be used by default if not given)")
                            .value_name("DIR")
                        )
                    )
            );
    }

    /************************************************************************************/
    /******************** Global and app commands ***************************************/
    /************************************************************************************/

    /************************************************************************************/
    /******************** Origen dev commands *******************************************/
    /************************************************************************************/
    if STATUS.is_origen_present || STATUS.is_app_present {
        let fmt_help = match STATUS.is_origen_present {
            true => "Nicely format all Rust and Python files",
            false => "Nicely format all of your application's Python files",
        };

        origen_commands.push(CommandHelp {
            name: "fmt".to_string(),
            help: fmt_help.to_string(),
            shortcut: None,
        });

        app = app
            //************************************************************************************/
            .subcommand(SubCommand::with_name("fmt").about(fmt_help));
    }

    if STATUS.is_origen_present {
        let build_help = "Build and deploy Origen";

        origen_commands.push(CommandHelp {
            name: "build".to_string(),
            help: build_help.to_string(),
            shortcut: None,
        });

        app = app
            //************************************************************************************/
            .subcommand(
                SubCommand::with_name("build").about(build_help)
                .arg(Arg::with_name("cli")
                        .long("cli")
                        .required(false)
                        .takes_value(false)
                        .help("Build the CLI (instead of the Python API)")
                )
                .arg(Arg::with_name("version")
                        .long("version")
                        .required(false)
                        .takes_value(true)
                        .value_name("VERSION")
                        .help("Set the version (of all components) to the given value before building")
                )
            );
    }

    /************************************************************************************/
    /******************** In application commands ***************************************/
    /************************************************************************************/
    if STATUS.is_app_present {
        /************************************************************************************/
        let i_help = "Start an Origen console to interact with the DUT";
        origen_commands.push(CommandHelp {
            name: "interactive".to_string(),
            help: i_help.to_string(),
            shortcut: Some("i".to_string()),
        });
        app = app.subcommand(
            SubCommand::with_name("interactive")
                .about(i_help)
                .visible_alias("i")
                .arg(
                    Arg::with_name("target")
                        .short("t")
                        .long("target")
                        .help("Override the default target currently set by the workspace")
                        .takes_value(true)
                        .use_delimiter(true)
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("TARGET"),
                )
                .arg(
                    Arg::with_name("mode")
                        .short("m")
                        .long("mode")
                        .help("Override the default execution mode currently set by the workspace")
                        .takes_value(true)
                        .value_name("MODE"),
                ),
        );

        /************************************************************************************/
        let g_help = "Generate patterns or test programs";
        origen_commands.push(CommandHelp {
            name: "generate".to_string(),
            help: g_help.to_string(),
            shortcut: Some("g".to_string()),
        });
        app = app.subcommand(
            SubCommand::with_name("generate")
                .about(g_help)
                .visible_alias("g")
                .arg(
                    Arg::with_name("files")
                        .help("The name of the file(s) to be generated")
                        .takes_value(true)
                        .value_name("FILES")
                        .multiple(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("target")
                        .short("t")
                        .long("target")
                        .help("Override the default target currently set by the workspace")
                        .takes_value(true)
                        .use_delimiter(true)
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("TARGET"),
                )
                .arg(
                    Arg::with_name("mode")
                        .short("m")
                        .long("mode")
                        .help("Override the default execution mode currently set by the workspace")
                        .takes_value(true)
                        .value_name("MODE"),
                )
                .arg(
                    Arg::with_name("output_dir")
                        .short("o")
                        .long("output-dir")
                        .help("Override the default output directory (<APP ROOT>/output)")
                        .takes_value(true)
                        .value_name("OUTPUT_DIR"),
                )
                .arg(
                    Arg::with_name("reference_dir")
                        .short("r")
                        .long("reference-dir")
                        .help("Override the default reference directory (<APP ROOT>/.ref)")
                        .takes_value(true)
                        .value_name("REFERENCE_DIR"),
                ),
        );

        /************************************************************************************/
        let c_help = "Compile templates";
        origen_commands.push(CommandHelp {
            name: "compile".to_string(),
            help: c_help.to_string(),
            shortcut: Some("c".to_string()),
        });
        app = app.subcommand(
            SubCommand::with_name("compile")
                .about(c_help)
                .visible_alias("c")
                .arg(
                    Arg::with_name("files")
                        .help("The name of the file(s) to be generated")
                        .takes_value(true)
                        .value_name("FILES")
                        .multiple(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("target")
                        .short("t")
                        .long("target")
                        .help("Override the default target currently set by the workspace")
                        .takes_value(true)
                        .use_delimiter(true)
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("TARGET"),
                )
                .arg(
                    Arg::with_name("mode")
                        .short("m")
                        .long("mode")
                        .help("Override the default execution mode currently set by the workspace")
                        .takes_value(true)
                        .value_name("MODE"),
                ),
        );

        /************************************************************************************/
        let t_help = "Set/view the default target";
        origen_commands.push(CommandHelp {
            name: "target".to_string(),
            help: t_help.to_string(),
            shortcut: Some("t".to_string()),
        });
        app = app.subcommand(
            SubCommand::with_name("target")
                .about(t_help)
                .visible_alias("t")
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Activates the given target(s)")
                        .visible_alias("a")
                        .arg(
                            Arg::with_name("targets")
                                .help("Targets to be activated")
                                .takes_value(true)
                                .value_name("TARGETS")
                                .multiple(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .about("Deactivates the given target(s)")
                        .visible_alias("r")
                        .arg(
                            Arg::with_name("targets")
                                .help("Targets to be deactivated")
                                .takes_value(true)
                                .value_name("TARGETS")
                                .multiple(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("set")
                        .about("Activates the given target(s) while deactivating all others")
                        .visible_alias("s")
                        .arg(
                            Arg::with_name("targets")
                                .help("Targets to be set")
                                .takes_value(true)
                                .value_name("TARGETS")
                                .multiple(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("default")
                        .about("Activates the default target(s) while deactivating all others")
                        .visible_alias("d"),
                )
                .subcommand(
                    SubCommand::with_name("view")
                        .about("Views the currently activated target(s)")
                        .visible_alias("v"),
                ),
        );

        /************************************************************************************/
        let mode_help = "Set/view the default execution mode";
        origen_commands.push(CommandHelp {
            name: "mode".to_string(),
            help: mode_help.to_string(),
            shortcut: Some("m".to_string()),
        });
        app = app.subcommand(
            SubCommand::with_name("mode")
                .about(mode_help)
                .visible_alias("m")
                .arg(
                    Arg::with_name("mode")
                        .help("The name of the mode to be set as the default mode")
                        .takes_value(true)
                        .value_name("MODE"),
                ),
        );

        /************************************************************************************/
        let setup_help = "Setup your application's Python environment in a new workspace, this will install dependencies per the poetry.lock file";
        origen_commands.push(CommandHelp {
            name: "setup".to_string(),
            help: setup_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(SubCommand::with_name("setup").about(setup_help));

        /************************************************************************************/
        let update_help = "Update your application's Python dependencies according to the latest pyproject.toml file";
        origen_commands.push(CommandHelp {
            name: "update".to_string(),
            help: update_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(SubCommand::with_name("update").about(update_help));

        /************************************************************************************/
        let save_ref_help = "Save a reference version of the given file, this will be automatically checked for differences the next time it is generated";
        origen_commands.push(CommandHelp {
            name: "save_ref".to_string(),
            help: save_ref_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            SubCommand::with_name("save_ref")
                .about(save_ref_help)
                .arg(
                    Arg::with_name("files")
                        .help("The name of the file(s) to be saved")
                        .takes_value(true)
                        .value_name("FILES")
                        .multiple(true)
                        .required_unless_one(&["new", "changed"]),
                )
                .arg(
                    Arg::with_name("new")
                        .long("new")
                        .required(false)
                        .takes_value(false)
                        .help("Update all NEW file references from the last generate run"),
                )
                .arg(
                    Arg::with_name("changed")
                        .long("changed")
                        .required(false)
                        .takes_value(false)
                        .help("Update all CHANGED file references from the last generate run"),
                ),
        );
    }

    // This is used to justify the command names in the help
    let mut name_width = origen_commands
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap();

    let mut app_command_defs = AppCommands::new(Path::new("/"));
    let cmds;
    if STATUS.is_app_present {
        app_command_defs = AppCommands::new(&origen::app().unwrap().root);
        app_command_defs.parse_commands();
        // Need to hold this in a long-lived immutable reference for referencing in clap args
        cmds = app_command_defs.commands.clone();

        if let Some(width) = app_command_defs.max_name_width() {
            if width > name_width {
                name_width = width;
            }
        }
        for command in &app_command_defs.command_helps {
            app_commands.push(command.clone());
        }
        // This defines the application commands, some crazy references here since the string
        // args to clap need the right lifetime
        // For each command
        for i in 0..cmds.len() {
            let cmd = build_command(&cmds[i]);
            app = app.subcommand(cmd);
        }
    }

    // Clap is great, but its generated help doesn't give the flexibility needed to handle things
    // like app and plugin command additions, so we make our own

    let mut help_message = format!(
        "Origen, The Semiconductor Developer's Kit

{}

USAGE:
    origen [FLAGS] [COMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Terminal verbosity level e.g. -v, -vv, -vvv

CORE COMMANDS:
",
        version
    );

    for command in &origen_commands {
        help_message += &command.render(name_width);
    }

    if !app_commands.is_empty() {
        help_message += "\nAPP COMMANDS:\n";
        for command in &app_commands {
            help_message += &command.render(name_width);
        }
    }

    help_message += "\nSee 'origen <command> -h' for more information on a specific command.";

    app = app.help(help_message.as_str());

    let matches = app.get_matches();

    let _ = LOGGER.set_verbosity(matches.occurrences_of("verbose") as u8);

    match matches.subcommand_name() {
        Some("setup") => commands::setup::run(),
        Some("update") => commands::update::run(),
        Some("fmt") => commands::fmt::run(),
        Some("build") => commands::build::run(matches.subcommand_matches("build").unwrap()),
        Some("proj") => commands::proj::run(matches.subcommand_matches("proj").unwrap()),
        Some("interactive") => {
            log_trace!("Launching interactive session");
            let m = matches.subcommand_matches("interactive").unwrap();
            commands::interactive::run(
                if let Some(targets) = m.values_of("target") {
                    Some(targets.collect())
                } else {
                    Option::None
                },
                &m.value_of("mode"),
            );
        }
        Some("generate") => {
            let m = matches.subcommand_matches("generate").unwrap();
            commands::launch(
                "generate",
                if let Some(targets) = m.values_of("target") {
                    Some(targets.collect())
                } else {
                    Option::None
                },
                &m.value_of("mode"),
                Some(m.values_of("files").unwrap().collect()),
                m.value_of("output_dir"),
                m.value_of("reference_dir"),
            );
        }
        Some("compile") => {
            let m = matches.subcommand_matches("compile").unwrap();
            commands::launch(
                "compile",
                if let Some(targets) = m.values_of("target") {
                    Some(targets.collect())
                } else {
                    Option::None
                },
                &m.value_of("mode"),
                Some(m.values_of("files").unwrap().collect()),
                m.value_of("output_dir"),
                m.value_of("reference_dir"),
            );
        }
        Some("target") => {
            let m = matches.subcommand_matches("target").unwrap();
            let subm = m.subcommand();
            if let Some(s) = subm.1 {
                commands::target::run(
                    Some(subm.0),
                    match s.values_of("targets") {
                        Some(targets) => Some(targets.collect()),
                        None => None,
                    },
                )
            } else {
                commands::target::run(None, None);
            }
        }
        Some("mode") => {
            let matches = matches.subcommand_matches("mode").unwrap();
            commands::mode::run(matches.value_of("mode"));
        }
        Some("save_ref") => {
            let matches = matches.subcommand_matches("save_ref").unwrap();
            commands::save_ref::run(matches);
        }
        // To get here means the user has typed "origen -v", which officially means
        // verbosity level 1 with no command, but this is what they really mean
        None => {
            if STATUS.is_app_present {
                // Run a short command line operation to get the Origen version back from the Python domain
                let cmd = "from origen.boot import run_cmd; run_cmd('_version_');";
                let mut origen_version = "".to_string();

                let res = python::run_with_callbacks(
                    cmd,
                    Some(&mut |line| {
                        origen_version += line;
                    }),
                    None,
                );

                if let Err(e) = res {
                    log_error!("{}", e);
                    log_error!("Couldn't boot app to determine the in-application Origen version");
                    origen_version = "Uknown".to_string();
                }

                let app_version = match origen::app().unwrap().version() {
                    Err(e) => {
                        log_error!("{}", e);
                        "Error".to_string()
                    }
                    Ok(v) => format!("{}", v),
                };

                println!(
                    "App:    {}\nOrigen: {}\nCLI:    {}",
                    app_version, origen_version, STATUS.origen_version
                );
            } else {
                println!("Origen: {}", STATUS.origen_version);
            }
        }
        _ => {
            // To get here we must be dealing with a command added by an app/plugin
            app_command_defs.dispatch(&matches);
        }
    }
}

fn build_command(cmd_def: &app_commands::Command) -> App {
    let mut cmd = SubCommand::with_name(&cmd_def.name).about(cmd_def.help.as_str());
    if cmd_def.alias.is_some() {
        cmd = cmd.visible_alias(cmd_def.alias.as_ref().unwrap().as_str());
    }
    if cmd_def.arg.is_some() {
        // For each arg
        for j in 0..cmd_def.arg.as_ref().unwrap().len() {
            let arg_def = &cmd_def.arg.as_ref().unwrap()[j];
            let mut arg = Arg::with_name(&arg_def.name).help(&arg_def.help);
            // If this is an arg without a switch
            if arg_def.switch.is_some() && !arg_def.switch.unwrap() {
                // Do nothing?
            } else {
                if arg_def.long.is_some() {
                    arg = arg.long(&arg_def.long.as_ref().unwrap());
                } else {
                    arg = arg.long(&arg_def.name);
                }
                if arg_def.short.is_some() {
                    arg = arg.short(arg_def.short.as_ref().unwrap())
                }
            }
            if arg_def.takes_value.is_some() {
                arg = arg.takes_value(arg_def.takes_value.unwrap())
            }
            if arg_def.multiple.is_some() {
                arg = arg.multiple(arg_def.multiple.unwrap())
            }
            if arg_def.required.is_some() {
                arg = arg.required(arg_def.required.unwrap())
            }
            if arg_def.value_name.is_some() {
                arg = arg.value_name(arg_def.value_name.as_ref().unwrap())
            } else {
                arg = arg.value_name(arg_def.upcased_name.as_ref().unwrap())
            }
            if arg_def.use_delimiter.is_some() {
                arg = arg.use_delimiter(arg_def.use_delimiter.unwrap())
            }
            if arg_def.hidden.is_some() {
                arg = arg.hidden(arg_def.hidden.unwrap())
            }

            cmd = cmd.arg(arg);
        }
    }
    if let Some(subcommands) = &cmd_def.subcommand {
        for c in subcommands {
            let subcmd = build_command(&c);
            cmd = cmd.subcommand(subcmd);
        }
    }
    cmd
}
