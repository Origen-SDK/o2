#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate origen_metal;

mod framework;
mod commands;
mod python;

use clap::{Arg, Command};
use indexmap::map::IndexMap;
use origen::{Result, LOGGER, STATUS};
use origen_metal as om;
use std::iter::FromIterator;
use std::process::exit;
use framework::{Extensions, Plugins, AuxCmds, AppCmds, CmdHelps};
use framework::plugins::{PL_MGR_CMD_NAME, PL_CMD_NAME, run_pl_mgr, run_pl};
use framework::{
    VERBOSITY_OPT_NAME, VERBOSITY_OPT_SHORT_NAME,
    VERBOSITY_KEYWORDS_OPT_NAME, VERBOSITY_KEYWORDS_OPT_LONG_NAME
};
use clap::error::ErrorKind as ClapErrorKind;
use commands::_prelude::clap_arg_actions::*;

static VERBOSITY_HELP_STR: &str = "Terminal verbosity level e.g. -v, -vv, -vvv";
static VERBOSITY_KEYWORD_HELP_STR: &str = "Keywords for verbose listeners";

#[derive(Clone)]
pub struct CommandHelp {
    name: String,
    help: String,
    shortcut: Option<String>,
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// This is the entry point for the Origen CLI tool
fn main() -> Result<()> {
    let verbosity_re = regex::Regex::new(r"-([vV]+)").unwrap();

    // Intercept the 'origen exec' command immediately to prevent further parsing of it, this
    // is so that something like 'origen exec pytest -v' will apply '-v' as an argument to pytest
    // and not to origen
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "exec" {
        args = args.drain(2..).collect();
        if args.len() > 0 && (args[0] == "-h" || args[0] == "--help" || args[0] == "help") {
            // Just fall through to display the help in this case
        } else {
            // Apply any leading -vvv to origen, any -v later in the args will be applied to the
            // 3rd party command
            if args.len() > 0 && verbosity_re.is_match(&args[0]) {
                let captures = verbosity_re.captures(&args[0]).unwrap();
                let x = captures.get(1).unwrap().as_str();
                let verbosity = x.chars().count() as u8;
                origen::initialize(
                    Some(verbosity),
                    vec![],
                    None,
                    Some(built_info::PKG_VERSION.to_string()),
                    None,
                    None,
                );
                args = args.drain(1..).collect();
            }
            // Command is not actually available outside an app, so just fall through
            // to generate the appropriate error
            if STATUS.is_app_present {
                if args.len() > 0 {
                    let cmd = args[0].clone();
                    args = args.drain(1..).collect();
                    let cmd_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    commands::exec::run(&cmd, cmd_args);
                } else {
                    std::process::exit(0);
                }
            }
        }
    }

    // Set the verbosity immediately, this is to allow log statements to work in really
    // low level stuff, e.g. when building the STATUS
    let mut verbosity: u8 = 0;
    for arg in std::env::args() {
        if let Some(captures) = verbosity_re.captures(&arg) {
            let x = captures.get(1).unwrap().as_str();
            verbosity = x.chars().count() as u8;
        }
    }
    let exe = match std::env::current_exe() {
        Ok(p) => Some(format!("{}", p.display())),
        Err(e) => {
            log_error!("{}", e);
            None
        }
    };
    origen::initialize(
        Some(verbosity),
        vec![],
        exe,
        Some(built_info::PKG_VERSION.to_string()),
        None,
        None,
    );

    let version = match STATUS.is_app_present {
        true => format!("Origen CLI: {}", STATUS.origen_version.to_string()),
        false => format!("Origen: {}", STATUS.origen_version.to_string()),
    };

    // The main help message is going to be automatically generated to allow us to handle and clearly
    // separate commands added by the app and plugins.
    // When a command is added below it must also be added to these vectors.
    let mut origen_commands: Vec<CommandHelp> = vec![];
    let mut helps = CmdHelps::new();
    let app_cmds: Option<AppCmds>;
    let mut extensions = Extensions::new();
    let plugins = match Plugins::new(&mut extensions) {
        Ok(pl) => pl,
        Err(e) => {
            log_error!("Failed to collect plugins. Encountered error: {}", e);
            None
        }
    };
    let aux_cmds = AuxCmds::new(&mut extensions)?;

    if let Some(app) = &STATUS.app.as_ref() {
        app_cmds = Some(AppCmds::new(app, &mut extensions)?);
    } else {
        app_cmds = None;
    }

    // Structures to hold command aliases and replacements
    // Clap does not want to own the values and, in the case of replacements
    // cannot be checked (easily) due to borrowing from one command to update another
    // Easier to just store things here and have clap reference them.
    let mut top_app_replacements: Vec<[&str; 3]> = vec![];
    let mut top_app_cmd_aliases: IndexMap<String, Vec<String>> = IndexMap::new();
    let mut top_pl_replacements: Vec<[&str; 3]> = vec![];
    let mut top_pl_cmd_aliases: IndexMap<String, IndexMap<String, Vec<String>>> = IndexMap::new();
    let mut top_aux_replacements: Vec<[&str; 3]> = vec![];
    let mut top_aux_cmd_aliases: IndexMap<String, IndexMap<String, Vec<String>>> = IndexMap::new();
    let mut after_help_str = "".to_string();

    let mut app = Command::new("")
        .arg_required_else_help(true)
        .disable_version_flag(true)
        .before_help("Origen, The Semiconductor Developer's Kit")
        .version(&*version)
        .arg(
            Arg::new("verbose")
                .long(VERBOSITY_OPT_NAME)
                .short(VERBOSITY_OPT_SHORT_NAME)
                .action(clap::builder::ArgAction::Count)
                .global(true)
                .help(VERBOSITY_HELP_STR),
        )
        .arg(
            Arg::new("verbosity_keywords")
                .long(VERBOSITY_KEYWORDS_OPT_NAME)
                .visible_alias(VERBOSITY_KEYWORDS_OPT_LONG_NAME)
                // .short('k')
                .multiple(true)
                .action(AppendArgs)
                .global(true)
                .help(VERBOSITY_KEYWORD_HELP_STR),
        );

    /************************************************************************************/
    /******************** Global only commands ******************************************/
    /************************************************************************************/
    if !STATUS.is_app_present {
        //************************************************************************************/
        let proj_help = "Manage multi-repository project areas and workspaces";
        origen_commands.push(CommandHelp {
            name: "proj".to_string(),
            help: proj_help.to_string(),
            shortcut: None,
        });

        app = app
            .subcommand(
                Command::new("proj")
                    .display_order(1)
                    .about(proj_help)
                    .arg_required_else_help(true)
                    .subcommand(Command::new("init")
                        .display_order(5)
                        .about("Initialize a new project directory (create an initial project BOM)")
                        .arg(Arg::new("dir")
                            .action(SetArg)
                            .help("The path to the project directory to initialize (PWD will be used by default if not given)")
                            .value_name("DIR")
                        )
                    )
                    .subcommand(Command::new("packages")
                        .display_order(7)
                        .about("Displays the IDs of all packages and package groups defined by the BOM")
                    )
                    .subcommand(Command::new("create")
                        .display_order(10)
                        .about("Create a new project workspace from the project BOM")
                        .arg(Arg::new("path")
                            .help("The path to the new workspace directory")
                            .action(SetArg)
                            .value_name("PATH")
                            .required(true)
                        )
                    )
                    .subcommand(Command::new("update")
                        .display_order(15)
                        .about("Update an existing project workspace per its current BOM")
                        .arg(Arg::new("force")
                            .short('f')
                            .long("force")
                            .required(false)
                            .action(SetArgTrue)
                            .help("Force the update and potentially lose any local modifications")
                        )
                        .arg(Arg::new("links")
                            .short('l')
                            .long("links")
                            .required(false)
                            .action(SetArgTrue)
                            .help("Update the workspace links")
                        )
                        .arg(Arg::new("packages")
                            .value_name("PACKAGES")
                            .action(AppendArgs)
                            .multiple(true)
                            .help("Packages and/or groups to be updated, run 'origen proj packages' to see a list of possible package IDs")
                            .required_unless("links")
                            .required(true)
                        )
                    )
                    .subcommand(Command::new("mods")
                        .display_order(20)
                        .about("Display a list of modified files within the given package(s)")
                        .arg(Arg::new("packages")
                            .help("Package(s) to look for modifications in, use 'all' to see the modification to all packages")
                            .action(AppendArgs)
                            .multiple(true)
                            .value_name("PACKAGES")
                            .required(true)
                        )
                    )
                    .subcommand(Command::new("clean")
                        .display_order(20)
                        .about("Revert all local modifications within the given package(s)")
                        .arg(Arg::new("packages")
                            .help("Package(s) to revert local modifications in, use 'all' to clean all packages")
                            .action(AppendArgs)
                            .multiple(true)
                            .value_name("PACKAGES")
                            .required(true)
                        )
                    )
                    .subcommand(Command::new("tag")
                        .display_order(20)
                        .about("Apply the given tag to the current view of the given package(s)")
                        .arg(Arg::new("name")
                            .help("Name of the tag to be applied")
                            .action(SetArg)
                            .value_name("NAME")
                            .required(true)
                        )
                        .arg(Arg::new("packages")
                            .help("Package(s) to be tagged, use 'all' to tag all packages")
                            .multiple(true)
                            .action(AppendArgs)
                            .value_name("PACKAGES")
                            .required(true)
                        )
                        .arg(Arg::new("force")
                            .short('f')
                            .long("force")
                            .required(false)
                            .action(SetArgTrue)
                            .help("Force the application of the tag even if there are local modifications")
                        )
                        .arg(Arg::new("message")
                            .short('m')
                            .long("message")
                            .required(false)
                            .action(SetArg)
                            .help("A message to be applied with the tag")
                        )
                    )
                    .subcommand(Command::new("bom")
                        .display_order(25)
                        .about("View the active BOM in the current or given directory")
                        .arg(Arg::new("dir")
                            .action(SetArg)
                            .help("The path to a directory (PWD will be used by default if not given)")
                            .value_name("DIR")
                        )
                    )
            );

        //************************************************************************************/
        let new_help = "Create a new Origen application";
        origen_commands.push(CommandHelp {
            name: "new".to_string(),
            help: new_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            Command::new("new").about(new_help).arg(
                Arg::new("name")
                    .help("The lowercased and underscored name of the new application")
                    .action(SetArg)
                    .required(true)
                    .number_of_values(1)
                    .value_name("NAME"),
            )
            .arg(Arg::new("setup")
                .help("Don't create the new app's virtual environment after building (need to manually run 'origen env setup' within the new app workspace before using it in that case)")
                .long("no-setup")
                .required(false)
                .action(SetArgTrue)
            ),
        );
    }

    framework::plugins::add_helps(&mut helps, plugins.as_ref());
    framework::aux_cmds::add_helps(&mut helps, &aux_cmds);
    commands::eval::add_helps(&mut helps);
    commands::credentials::add_helps(&mut helps);
    commands::interactive::add_helps(&mut helps);

    if STATUS.is_app_present {
        commands::app::add_helps(&mut helps, app_cmds.as_ref().unwrap());
        commands::target::add_helps(&mut helps);
    }

    helps.apply_exts(&extensions);

    /************************************************************************************/
    /******************** Global and app commands ***************************************/
    /************************************************************************************/

    // app = mailer::add_commands(app, &mut origen_commands)?;
    app = commands::credentials::add_commands(app, &helps, &extensions)?;
    app = commands::eval::add_commands(app, &helps, &extensions)?;
    app = commands::interactive::add_commands(app, &helps, &extensions)?;
    app = framework::plugins::add_commands(app, &helps, plugins.as_ref(), &extensions)?;
    app = framework::aux_cmds::add_commands(app, &helps, &aux_cmds, &extensions)?;

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
            .subcommand(Command::new("fmt").about(fmt_help));
    }

    if STATUS.is_origen_present || STATUS.is_app_in_origen_dev_mode {
        let (app_, help) = commands::build::define(app);
        app = app_;
        origen_commands.push(help);
    }

    /************************************************************************************/
    /******************** In application commands ***************************************/
    /************************************************************************************/
    if STATUS.is_app_present {
        app = commands::app::add_commands(app, &helps, app_cmds.as_ref().unwrap(), &extensions)?;

        /************************************************************************************/
        let new_help = "Generate a new block, flow, pattern, etc. for your application";
        origen_commands.push(CommandHelp {
            name: "new".to_string(),
            help: new_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            Command::new("new")
            .about(new_help)
            .arg_required_else_help(true)
            .subcommand(Command::new("dut")
                .display_order(5)
                .about("Create a new top-level (DUT) block, see 'origen new dut -h' for more info")
                .long_about(
"This generator creates a top-level (DUT) block and all of the associated resources for it, e.g. a
reg file, controller, target, timesets, pins, etc.

The NAME of the DUT should be given in lower case, optionally prefixed by parent DUT name(s) separated
by a forward slash.

Any parent DUT(s) will be created if they don't exist, but they will not be modified if they do.

Examples:
  origen new dut                # Creates <app_name>/blocks/dut/...
  origen new dut falcon         # Creates <app_name>/blocks/dut/derivatives/falcon/...
  origen new dut dsp/falcon     # Creates <app_name>/blocks/dut/derivatives/dsp/derivatives/falcon/...")
                .arg(Arg::new("name")
                    .action(SetArg)
                    .required(false)
                    .help("The name of the new DUT")
                    .value_name("NAME")
                )
            )
            .subcommand(Command::new("block")
                .display_order(5)
                .about("Create a new block, see 'origen new block -h' for more info")
                .long_about(
"This generator creates a block (e.g. to represent RAM, ATD, Flash, DAC, etc.) and all of the associated
resources for it, e.g. a reg file, controller, timesets, etc.

The NAME should be given in lower case (e.g. flash/flash2kb, adc/adc16), optionally with
additional parent sub-block names after the initial type.

Alternatively, a reference to an existing BLOCK can be added, in which case a nested block will be created
within that block's 'blocks/' directory, rather than a primary top-level block.

Any parent block(s) will be created if they don't exist, but they will not be modified if they do.

Examples:
  origen new block dac                  # Creates <app_name>/blocks/dac/...
  origen new block adc/adc8bit          # Creates <app_name>/blocks/adc/derivatives/adc8bit/...
  origen new block adc/adc16bit         # Creates <app_name>/blocks/adc/derivatives/adc16bit/...
  origen new block nvm/flash/flash2kb   # Creates <app_name>/blocks/nvm/derivatives/flash/derivatives/flash2kb/...

  # Example of creating a nested sub-block
  origen new block bist --parent nvm/flash   # Creates <app_name>/blocks/nvm/derivatives/flash/blocks/bist/...")
                .arg(Arg::new("name")
                    .action(SetArg)
                    .required(true)
                    .help("The name of the new block, including its parents if applicable")
                    .value_name("NAME")
                )
                .arg(
                    Arg::new("parent")
                        .short('p')
                        .long("parent")
                        .help("Create the new block nested within this existing block")
                        .action(SetArg)
                        .required(false)
                        .value_name("PARENT")
                )
            )
        );

        /************************************************************************************/
        let g_help = "Generate patterns or test programs";
        origen_commands.push(CommandHelp {
            name: "generate".to_string(),
            help: g_help.to_string(),
            shortcut: Some("g".to_string()),
        });
        app = app.subcommand(
            Command::new("generate")
                .about(g_help)
                .visible_alias("g")
                .arg(
                    Arg::new("files")
                        .help("The name of the file(s) to be generated")
                        .action(AppendArgs)
                        .value_name("FILES")
                        .multiple(true)
                        .required(true),
                )
                .arg(
                    Arg::new("target")
                        .short('t')
                        .long("target")
                        .help("Override the default target currently set by the workspace")
                        .action(AppendArgs)
                        .use_delimiter(true)
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("TARGET"),
                )
                .arg(
                    Arg::new("mode")
                        .short('m')
                        .long("mode")
                        .help("Override the default execution mode currently set by the workspace")
                        .action(SetArg)
                        .value_name("MODE"),
                )
                .arg(
                    Arg::new("output_dir")
                        .short('o')
                        .long("output-dir")
                        .help("Override the default output directory (<APP ROOT>/output)")
                        .action(SetArg)
                        .value_name("OUTPUT_DIR"),
                )
                .arg(
                    Arg::new("reference_dir")
                        .short('r')
                        .long("reference-dir")
                        .help("Override the default reference directory (<APP ROOT>/.ref)")
                        .action(SetArg)
                        .value_name("REFERENCE_DIR"),
                )
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .short('d')
                        .help("Enable Python caller tracking for debug (takes longer to execute)")
                        .action(SetArgTrue),
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
            Command::new("compile")
                .about(c_help)
                .visible_alias("c")
                .arg(
                    Arg::new("files")
                        .help("The name of the file(s) to be generated")
                        .action(AppendArgs)
                        .value_name("FILES")
                        .multiple(true)
                        .required(true),
                )
                .arg(
                    Arg::new("target")
                        .short('t')
                        .long("target")
                        .help("Override the default target currently set by the workspace")
                        .action(AppendArgs)
                        .use_delimiter(true)
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("TARGET"),
                )
                .arg(
                    Arg::new("mode")
                        .short('m')
                        .long("mode")
                        .help("Override the default execution mode currently set by the workspace")
                        .action(SetArg)
                        .value_name("MODE"),
                ),
        );

        app = commands::target::add_commands(app, &helps, &extensions)?;

        /************************************************************************************/
        let t_help = "Create, Build, and View Web Documentation";
        origen_commands.push(CommandHelp {
            name: "web".to_string(),
            help: t_help.to_string(),
            shortcut: Some("w".to_string()),
        });
        app = app.subcommand(
            Command::new("web")
                .about(t_help)
                .arg_required_else_help(true)
                .visible_alias("w")
                .subcommand(
                    Command::new("build") // What I think this command should be called
                        .about("Builds the web documentation")
                        .visible_alias("b")
                        .visible_alias("compile") // If coming from O1
                        .visible_alias("html") // If coming from Sphinx and using quickstart's Makefile
                        .arg(
                            Arg::new("view")
                                .long("view")
                                .help("Launch your web browser after the build")
                                .action(SetArgTrue),
                        )
                        .arg(
                            Arg::new("clean")
                                .long("clean")
                                .help(
                                    "Clean up directories from previous builds and force a rebuild",
                                )
                                .action(SetArgTrue),
                        )
                        .arg(
                            Arg::new("release")
                                .long("release")
                                .short('r')
                                .help("Release (deploy) the resulting web pages")
                                .action(SetArgTrue),
                        )
                        .arg(
                            Arg::new("archive")
                                .long("archive")
                                .short('a')
                                .help("Archive the resulting web pages after building")
                                .action(SetArg)
                                .multiple(false)
                                .min_values(0),
                        )
                        .arg(
                            Arg::new("as-release")
                                .long("as-release")
                                .help("Build webpages with release checks")
                                .action(SetArgTrue),
                        )
                        .arg(
                            Arg::new("release-with-warnings")
                                .long("release-with-warnings")
                                .help("Release webpages even if warnings persists")
                                .action(SetArgTrue),
                        )
                        .arg(
                            Arg::new("no-api")
                                .long("no-api")
                                .help("Skip building the API")
                                .action(SetArgTrue),
                        )
                        .arg(
                            Arg::new("sphinx-args")
                                .long("sphinx-args")
                                .help(
                                    "Additional arguments to pass to the 'sphinx-build' command
  Argument will passed as a single string and appended to the build command
  E.g.: 'origen web build --sphinx-args \"-q -D my_config_define=1\"'
     -> 'sphinx-build <source_dir> <output_dir> -q -D my_config_define=1'",
                                )
                                .action(SetArg)
                                .multiple(false)
                                .allow_hyphen_values(true),
                        ), // .arg(Arg::new("pdf")
                           //     .long("pdf")
                           //     .help("Create a PDF of resulting web pages")
                           //     .action(SetArgTrue)
                           // )
                )
                .subcommand(
                    Command::new("view")
                        .about("Launches your web browser to view previously built documentation")
                        .visible_alias("v"),
                )
                .subcommand(
                    Command::new("clean")
                        .about("Cleans the output directory and all cached files"),
                ),
        );

        /************************************************************************************/
        let mailer_help =
            "Command-line-interface to Origen's mailer for quick emailing or shell-scripting";
        origen_commands.push(CommandHelp {
            name: "mailer".to_string(),
            help: mailer_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            Command::new("mailer")
                .about(mailer_help)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("send")
                        .about("Quickly send an email")
                        .arg(
                            Arg::new("body")
                                .help("Email message body")
                                .long("body")
                                .action(SetArg)
                                .required(true)
                                .value_name("BODY")
                                .index(1),
                        )
                        .arg(
                            Arg::new("subject")
                                .help("Email subject line")
                                .long("subject")
                                .short('s')
                                .action(SetArg)
                                .value_name("SUBJECT"),
                        )
                        .arg(
                            Arg::new("to")
                                .help("Recipient list")
                                .long("to")
                                .short('t')
                                .action(AppendArgs)
                                .required(true)
                                .multiple(true)
                                .value_name("TO"),
                        ),
                )
                .subcommand(
                    Command::new("test")
                        .about("Send a test email")
                        .arg(
                            Arg::new("to")
                                .help(
                                    "Recipient list. If omitted, will be sent to the current user",
                                )
                                .long("to")
                                .short('t')
                                .action(AppendArgs)
                                .required(false)
                                .multiple(true)
                                .value_name("TO"),
                        ),
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
            Command::new("mode")
                .about(mode_help)
                .visible_alias("m")
                .arg(
                    Arg::new("mode")
                        .help("The name of the mode to be set as the default mode")
                        .action(SetArg)
                        .value_name("MODE"),
                ),
        );

        /************************************************************************************/
        let env_help = "Manage your application's Origen/Python environment (dependencies, etc.)";
        origen_commands.push(CommandHelp {
            name: "env".to_string(),
            help: env_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(Command::new("env").about(env_help)
            .arg_required_else_help(true)
            .subcommand(
                Command::new("setup")
                    .about("Setup your application's Python environment for the first time in a new workspace, this will install dependencies per the poetry.lock file")
                    .arg(Arg::new("origen")
                            .long("origen")
                            .help("The path to a local version of Origen to use (to develop Origen)")
                            .action(SetArg)
                        ),
            )
            .subcommand(
                Command::new("update")
                    .about("Update your application's Python dependencies according to the latest pyproject.toml file"),
            )
        );

        /************************************************************************************/
        let exec_help = "Execute a command within your application's Origen/Python environment (e.g. origen exec pytest)";
        origen_commands.push(CommandHelp {
            name: "exec".to_string(),
            help: exec_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            Command::new("exec")
                .about(exec_help)
                .arg_required_else_help(true)
                .allow_hyphen_values(true)
                .arg(
                    Arg::new("cmd")
                        .help("The command to be run")
                        .action(SetArg)
                        .required(true)
                        .value_name("COMMAND"),
                )
                .arg(
                    Arg::new("args")
                        .help("Arguments to be passed to the command")
                        .action(AppendArgs)
                        .allow_hyphen_values(true)
                        .multiple(true)
                        .number_of_values(1)
                        .required(false)
                        //.last(true)
                        .value_name("ARGS"),
                ),
        );

        /************************************************************************************/
        let save_ref_help = "Save a reference version of the given file, this will be automatically checked for differences the next time it is generated";
        origen_commands.push(CommandHelp {
            name: "save_ref".to_string(),
            help: save_ref_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            Command::new("save_ref")
                .about(save_ref_help)
                .arg(
                    Arg::new("files")
                        .help("The name of the file(s) to be saved")
                        .action(SetArg)
                        .value_name("FILES")
                        .multiple(true)
                        .required_unless_one(&["new", "changed"]),
                )
                .arg(
                    Arg::new("new")
                        .long("new")
                        .required(false)
                        .action(SetArgTrue)
                        .help("Update all NEW file references from the last generate run"),
                )
                .arg(
                    Arg::new("changed")
                        .long("changed")
                        .required(false)
                        .action(SetArgTrue)
                        .help("Update all CHANGED file references from the last generate run"),
                ),
        );
    }

    let mut all_cmds_and_aliases = vec![];
    for subc in app.get_subcommands() {
        all_cmds_and_aliases.push(subc.get_name().to_string());
        for a in subc.get_all_aliases() {
            all_cmds_and_aliases.push(a.to_string());
        }
    }

    if let Some(a_cmds) = app_cmds.as_ref() {
        for top_cmd in a_cmds.top_commands.iter() {
            // TODO test that aliases vs. command names at the same level are safe (clap should fail earlier for this)
            match app.try_get_matches_from_mut(["origen", top_cmd]) {
                Ok(_) => {
                    top_app_cmd_aliases.insert(top_cmd.to_string(), vec!(top_cmd.to_string()));
                    top_app_replacements.push(["app", "commands", top_cmd]);
                },
                Err(e) => {
                    match e.kind {
                        ClapErrorKind::DisplayHelp |
                        ClapErrorKind::DisplayHelpOnMissingArgumentOrSubcommand |
                        ClapErrorKind::DisplayVersion |
                        ClapErrorKind::UnknownArgument => {
                            top_app_cmd_aliases.insert(top_cmd.to_string(), vec!(top_cmd.to_string()));
                            top_app_replacements.push(["app", "commands", top_cmd]);
                        },
                        _ => {}
                    }
                },
            }
            let current_top_cmd_aliases = app.find_subcommand("app").unwrap().find_subcommand("commands").unwrap().find_subcommand(top_cmd).unwrap().get_all_aliases().map( |a| a.to_string()).collect::<Vec<String>>();
            for a in current_top_cmd_aliases.iter() {
                match app.try_get_matches_from_mut(["origen", a]) {
                    Ok(_) => {
                        if let Some(aliases) = top_app_cmd_aliases.get_mut(top_cmd) {
                            aliases.push(a.to_string());
                        } else {
                            top_app_cmd_aliases.insert(top_cmd.to_string(), vec!(a.to_string()));
                        }
                    },
                    Err(e) => {
                        match e.kind {
                            ClapErrorKind::DisplayHelp |
                            ClapErrorKind::DisplayHelpOnMissingArgumentOrSubcommand |
                            ClapErrorKind::DisplayVersion |
                            ClapErrorKind::UnknownArgument => {
                                if let Some(aliases) = top_app_cmd_aliases.get_mut(top_cmd) {
                                    aliases.push(a.to_string());
                                } else {
                                    top_app_cmd_aliases.insert(top_cmd.to_string(), vec!(a.to_string()));
                                }
                            },
                            _ => {}
                        }
                    },
                }
            }
        }

        let mut strs = vec!();
        if !top_app_cmd_aliases.is_empty() {
            let mut len = 0;
            for (n, aliases) in top_app_cmd_aliases.iter() {
                for a in aliases.iter() {
                    top_app_replacements.push(["app", "commands", a]);
                }

                let s = aliases.join(", ");
                let l = s.len();
                if l > len {
                    len = l;
                }
                strs.push((s, l, n))
            }
            for r in top_app_replacements.iter() {
                app = app.replace(r[2], r);
            }
            after_help_str += "APP COMMAND SHORTCUTS:\nThe following shortcuts to application commands are available:\n";
            for s in strs.iter() {
                after_help_str += &format!("    {s}{:<w$} => {c}\n", "", w=(len - s.1), s=s.0, c=s.2);
            }
            after_help_str += "\n";
        }
    }

    if let Some(pls) = plugins.as_ref() {
        for (n, pl) in pls.plugins.iter() {
            for top_cmd in pl.top_commands.iter() {
                if !all_cmds_and_aliases.contains(top_cmd) {
                    if let Some(cmd_aliases) = top_pl_cmd_aliases.get_mut(n) {
                        cmd_aliases.insert(top_cmd.to_string(), vec!(top_cmd.to_string()));
                    } else {
                        let mut pl_aliases = IndexMap::new();
                        pl_aliases.insert(top_cmd.to_string(), vec!(top_cmd.to_string()));
                        top_pl_cmd_aliases.insert(n.to_string(), pl_aliases);
                        all_cmds_and_aliases.push(top_cmd.to_string());
                    }
                }

                let current_top_cmd_aliases = app.find_subcommand("plugin").unwrap().find_subcommand(n).unwrap().find_subcommand(top_cmd).unwrap().get_all_aliases().map( |a| a.to_string()).collect::<Vec<String>>();
                for a in current_top_cmd_aliases.iter() {
                    if !all_cmds_and_aliases.contains(a) {
                        if let Some(pl_aliases) = top_pl_cmd_aliases.get_mut(n) {
                            if let Some(cmd_aliases) = pl_aliases.get_mut(top_cmd) {
                                cmd_aliases.push(a.to_string());
                            } else {
                                pl_aliases.insert(top_cmd.to_string(), vec!(a.to_string()));
                            }
                        } else {
                            let mut pl_aliases = IndexMap::new();
                            pl_aliases.insert(top_cmd.to_string(), vec!(a.to_string()));
                            top_pl_cmd_aliases.insert(n.to_string(), pl_aliases);
                        }
                        all_cmds_and_aliases.push(a.to_string());
                    }
                }
            }
        }

        let mut strs = vec!();
        if !top_pl_cmd_aliases.is_empty() {
            let mut len = 0;
            for (pln, pl_aliases) in top_pl_cmd_aliases.iter() {
                for (cmdn, cmda) in pl_aliases {
                    top_pl_replacements.push(["plugin", pln, cmdn]);

                    let s = cmda.join(", ");
                    let l = s.len();
                    if l > len {
                        len = l;
                    }
                    strs.push((s, l, format!("{} {}", pln, cmdn)))
                }
            }

            for r in top_pl_replacements.iter() {
                app = app.replace(r[2], r);
            }

            after_help_str += "PLUGIN COMMAND SHORTCUTS:\nThe following shortcuts to plugin commands are available:\n";
            for s in strs.iter() {
                after_help_str += &format!("    {s}{:<w$} => {c}\n", "", w=(len - s.1), s=s.0, c=s.2);
            }
            after_help_str += "\n";
        }
    }

    for (n, ns) in aux_cmds.namespaces.iter() {
        for top_cmd in ns.top_commands.iter() {
            if !all_cmds_and_aliases.contains(top_cmd) {
                if let Some(cmd_aliases) = top_aux_cmd_aliases.get_mut(n) {
                    cmd_aliases.insert(top_cmd.to_string(), vec!(top_cmd.to_string()));
                } else {
                    let mut ns_aliases = IndexMap::new();
                    ns_aliases.insert(top_cmd.to_string(), vec!(top_cmd.to_string()));
                    top_aux_cmd_aliases.insert(n.to_string(), ns_aliases);
                }
                all_cmds_and_aliases.push(top_cmd.to_string());
            }

            let current_top_cmd_aliases = app.find_subcommand("auxillary_commands").unwrap().find_subcommand(n).unwrap().find_subcommand(top_cmd).unwrap().get_all_aliases().map( |a| a.to_string()).collect::<Vec<String>>();
            for a in current_top_cmd_aliases.iter() {
                if !all_cmds_and_aliases.contains(a) {
                    if let Some(ns_aliases) = top_aux_cmd_aliases.get_mut(n) {
                        if let Some(cmd_aliases) = ns_aliases.get_mut(top_cmd) {
                            cmd_aliases.push(a.to_string());
                        } else {
                            ns_aliases.insert(top_cmd.to_string(), vec!(a.to_string()));
                        }
                    } else {
                        let mut ns_aliases = IndexMap::new();
                        ns_aliases.insert(top_cmd.to_string(), vec!(a.to_string()));
                        top_aux_cmd_aliases.insert(n.to_string(), ns_aliases);
                    }
                    all_cmds_and_aliases.push(a.to_string());
                }
            }
        }
    }
    if !top_aux_cmd_aliases.is_empty() {
        let mut strs = vec!();
        let mut len = 0;
        for (auxn, aux_aliases) in top_aux_cmd_aliases.iter() {
            for (cmdn, cmda) in aux_aliases {
                top_aux_replacements.push(["auxillary_commands", auxn, cmdn]);

                let s = cmda.join(", ");
                let l = s.len();
                if l > len {
                    len = l;
                }
                strs.push((s, l, format!("{} {}", auxn, cmdn)))
            }
        }

        for r in top_aux_replacements.iter() {
            app = app.replace(r[2], r);
        }

        after_help_str += "AUX COMMAND SHORTCUTS:\nThe following shortcuts to auxillary commands are available:\n";
        for s in strs.iter() {
            after_help_str += &format!("    {s}{:<w$} => {c}\n", "", w=(len - s.1), s=s.0, c=s.2);
        }
        after_help_str += "\n";
    }

    after_help_str += "See 'origen <command> -h' for more information on a specific command.";
    app = app.after_help(&*after_help_str);

    let matches = app.get_matches_mut();

    let _ = LOGGER.set_verbosity(*matches.get_one::<u8>("verbose").unwrap());
    if let Some(keywords) = matches.get_many::<String>("verbosity_keywords") {
        let _ = LOGGER.set_verbosity_keywords(keywords.map(|k| k.to_string()).collect());
    }

    macro_rules! run_cmd_match_case {
        ($cmd:ident, $cmd_name:ident) => {
            commands::$cmd::run(matches.subcommand_matches(commands::$cmd::$cmd_name).unwrap(), &app, &extensions, plugins.as_ref())?
        };
        ($cmd:ident) => {
            commands::$cmd::run(matches.subcommand_matches(commands::$cmd::BASE_CMD).unwrap(), &app, &extensions, plugins.as_ref())?
        }
    }

    match matches.subcommand_name() {
        Some(commands::app::BASE_CMD) => commands::app::run(matches.subcommand_matches(commands::app::BASE_CMD).unwrap(), &app, &extensions, plugins.as_ref(), &app_cmds.as_ref().unwrap())?,
        Some("env") => commands::env::run(matches.subcommand_matches("env").unwrap()),
        Some("fmt") => commands::fmt::run()?,
        Some("new") => commands::new::run(matches.subcommand_matches("new").unwrap()),
        Some("build") => commands::build::run(matches.subcommand_matches("build").unwrap())?,
        Some("proj") => commands::proj::run(matches.subcommand_matches("proj").unwrap()),
        Some(commands::eval::BASE_CMD) => run_cmd_match_case!(eval),
        Some(commands::interactive::BASE_CMD) => run_cmd_match_case!(interactive),
        Some(commands::aux_cmds::BASE_CMD) => commands::aux_cmds::run(matches.subcommand_matches(commands::aux_cmds::BASE_CMD).unwrap(), &app, &extensions, plugins.as_ref(), &aux_cmds)?,
        Some("generate") => {
            let m = matches.subcommand_matches("generate").unwrap();
            commands::launch(
                "generate",
                if let Some(targets) = m.get_many::<String>("target") {
                    Some(targets.map(|t| t.as_str()).collect())
                } else {
                    Option::None
                },
                &m.get_one::<&str>("mode").map(|s| *s),
                Some(m.get_many::<String>("files").unwrap().map(|t| t.as_str()).collect()),
                m.get_one::<&str>("output_dir").map(|s| *s),
                m.get_one::<&str>("reference_dir").map(|s| *s),
                m.contains_id("debug"),
                None,
            );
        }
        Some("compile") => {
            let m = matches.subcommand_matches("compile").unwrap();
            commands::launch(
                "compile",
                if let Some(targets) = m.get_many::<String>("target") {
                    Some(targets.map(|t| t.as_str()).collect())
                } else {
                    Option::None
                },
                &m.get_one::<&str>("mode").map(|s| *s),
                Some(m.get_many::<String>("files").unwrap().map(|t| t.as_str()).collect()),
                m.get_one::<&str>("output_dir").map(|s| *s),
                m.get_one::<&str>("reference_dir").map(|s| *s),
                false,
                None,
            );
        }
        Some(commands::target::BASE_CMD) => commands::target::run(matches.subcommand_matches(commands::target::BASE_CMD).unwrap())?,
        Some("web") => {
            let cmd = matches.subcommand_matches("web").unwrap();
            let subcmd = cmd.subcommand().unwrap();
            let sub = subcmd.1;
            match subcmd.0 {
                "build" => {
                    let mut args = IndexMap::new();
                    if sub.contains_id("view") {
                        args.insert("view", "True".to_string());
                    }
                    if sub.contains_id("clean") {
                        args.insert("clean", "True".to_string());
                    }
                    if sub.contains_id("no-api") {
                        args.insert("no-api", "True".to_string());
                    }
                    if sub.contains_id("as-release") {
                        args.insert("as-release", "True".to_string());
                    }
                    if sub.contains_id("release-with-warnings") {
                        args.insert("release-with-warnings", "True".to_string());
                    }
                    if sub.contains_id("release") {
                        args.insert("release", "True".to_string());
                    }
                    if sub.contains_id("archive") {
                        if let Some(archive) = sub.get_one::<&str>("archive") {
                            args.insert("archive", format!("'{}'", archive));
                        } else {
                            args.insert("archive", "True".to_string());
                        }
                    }
                    if let Some(s_args) = sub.get_one::<&str>("sphinx-args") {
                        // Recall that this comes in as a single argument, potentially quoted to mimic multiple,
                        // but a single argument from the perspective here nonetheless
                        args.insert("sphinx-args", format!("'{}'", s_args));
                    }
                    commands::launch(
                        "web:build",
                        if let Some(targets) = cmd.get_many::<String>("target") {
                            Some(targets.map(|t| t.as_str()).collect())
                        } else {
                            Option::None
                        },
                        &None,
                        None,
                        None,
                        None,
                        false,
                        Some(args),
                    )
                }
                "view" => commands::launch("web:view", None, &None, None, None, None, false, None),
                "clean" => {
                    commands::launch("web:clean", None, &None, None, None, None, false, None)
                }
                _ => {}
            }
        }
        Some("mailer") => {
            let cmd = matches.subcommand_matches("mailer").unwrap();
            let subcmd = cmd.subcommand().unwrap();
            let sub = subcmd.1;
            match subcmd.0 {
                "send" => {
                    let mut args = IndexMap::new();
                    if let Some(t) = sub.get_many::<String>("to") {
                        let r = t.map(|x| format!("\"{}\"", x)).collect::<Vec<String>>();
                        args.insert("to", format!("[{}]", r.join(",")));
                    }
                    if let Some(s) = sub.get_one::<&str>("subject") {
                        args.insert("subject", format!("\"{}\"", s));
                    }
                    if let Some(b) = sub.get_one::<&str>("body") {
                        args.insert("body", format!("\"{}\"", b));
                    }

                    commands::launch(
                        "mailer:send",
                        if let Some(targets) = cmd.get_many::<String>("target") {
                            Some(targets.map(|t| t.as_str()).collect())
                        } else {
                            Option::None
                        },
                        &None,
                        None,
                        None,
                        None,
                        false,
                        Some(args),
                    )
                }
                "test" => {
                    let mut args = IndexMap::new();
                    if let Some(t) = sub.get_many::<String>("to") {
                        let r = t.map(|x| format!("\"{}\"", x)).collect::<Vec<String>>();
                        args.insert("to", format!("[{}]", r.join(",")));
                    }
                    commands::launch(
                        "mailer:test",
                        if let Some(targets) = cmd.get_many::<String>("target") {
                            Some(targets.map(|t| t.as_str()).collect())
                        } else {
                            Option::None
                        },
                        &None,
                        None,
                        None,
                        None,
                        false,
                        Some(args),
                    )
                }
                _ => {}
            }
        }
        Some("credentials") => run_cmd_match_case!(credentials),
        Some("mode") => {
            let matches = matches.subcommand_matches("mode").unwrap();
            commands::mode::run(matches.get_one::<&str>("mode").map(|s| *s));
        }
        Some("save_ref") => {
            let matches = matches.subcommand_matches("save_ref").unwrap();
            commands::save_ref::run(matches);
        }
        Some(PL_MGR_CMD_NAME) => run_pl_mgr(matches.subcommand_matches(PL_MGR_CMD_NAME).unwrap(), plugins.as_ref())?,
        Some(PL_CMD_NAME) => run_pl(matches.subcommand_matches(PL_CMD_NAME).unwrap(), &app, &extensions, plugins.as_ref())?,
        Some(invalid_cmd) => {
            // This case shouldn't happen as clap should've previously kicked out on any invalid command
            unreachable!("Uncaught invalid command encountered: '{}'", invalid_cmd);
        }
        None => {
            // To get here means the user has typed "origen -v", which officially means
            // verbosity level 1 with no command, but this is what they really mean
            let mut max_len = 0;
            let mut versions: IndexMap<String, (bool, bool, String)> = IndexMap::new();
            if STATUS.is_app_present {
                let cmd = "from origen.boot import run_cmd; run_cmd('_version_');";
                let mut output_lines = "".to_string();

                let res = python::run_with_callbacks(
                    cmd,
                    Some(&mut |line| {
                        output_lines += &format!("{}\n", line);
                    }),
                    None,
                );
                output_lines.pop();

                match res {
                    Ok(_) => {
                        let lines = output_lines.split("\n").collect::<Vec<&str>>();
                        if lines.len() == 0 || lines.len() == 1 {
                            log_error!(
                                "Unable to parse in-application version output. Expected newlines:"
                            );
                            log_error!("{}", output_lines);
                        } else {
                            let mut phase = 0;
                            let mut current = "".to_string();
                            let mut is_private = false;
                            let mut is_okay = false;
                            let mut ver_or_message = "".to_string();
                            for l in lines {
                                if phase == 0 {
                                    let ver = parse_version_token(l);
                                    current = ver.0;
                                    is_private = ver.1;
                                    if !is_private && current.len() > max_len {
                                        max_len = current.len();
                                    }
                                    phase += 1;
                                } else if phase == 1 {
                                    match origen::utility::status_to_bool(l) {
                                        Ok(stat) => is_okay = stat,
                                        Err(e) => {
                                            log_error!("{}", e.msg);
                                            log_error!("Unable to parse version information");
                                            break;
                                        }
                                    }
                                    phase += 1;
                                } else if phase == 2 {
                                    match l.chars().next() {
                                        Some(t) => {
                                            if t == '\t' {
                                                ver_or_message += &l[1..];
                                            } else {
                                                versions.insert(
                                                    current.to_string(),
                                                    (
                                                        is_okay,
                                                        is_private,
                                                        ver_or_message.to_string(),
                                                    ),
                                                );
                                                let ver = parse_version_token(l);
                                                current = ver.0;
                                                is_private = ver.1;
                                                if !is_private && current.len() > max_len {
                                                    max_len = current.len();
                                                }
                                                ver_or_message = "".to_string();
                                                phase = 1;
                                            }
                                        }
                                        None => {
                                            log_error!("Unable to parse in-application version output - unexpected empty line:");
                                            log_error!("{}", output_lines);
                                        }
                                    }
                                } else {
                                    log_error!("Unable to parse in-application version output:");
                                    log_error!("{}", output_lines);
                                }
                            }

                            if phase == 2 {
                                versions.insert(
                                    current.clone(),
                                    (is_okay, is_private, ver_or_message.clone()),
                                );
                            } else {
                                log_error!("Unable to parse in-application version output - unexpected format:");
                                log_error!("{}", output_lines);
                            }
                        }
                    }
                    Err(e) => {
                        log_error!(
                            "Couldn't boot app to determine the in-application Origen version"
                        );
                        log_error!("Received Error:");
                        log_error!("");
                        log_error!("{}", e);
                        if output_lines != "" {
                            log_error!("");
                            log_error!("The following was also captured:");
                            log_error!("");
                            log_error!("{}", output_lines);
                        }
                        exit(1);
                    }
                }
            } else {
                versions.insert(
                    "Origen".to_string(),
                    (true, false, STATUS.origen_version.to_string()),
                );
                versions.insert(
                    "CLI".to_string(),
                    (true, false, built_info::PKG_VERSION.to_string()),
                );
                max_len = 6; // 'Origen'
            }

            for (n, v) in versions.iter() {
                if v.0 == true {
                    if v.1 == true {
                        log_debug!("{}: {}", n, v.2);
                    } else {
                        println!("{}: {}{}", n, " ".repeat(max_len - n.len()), v.2);
                    }
                } else {
                    log_error!("Errors encountered retrieving version info for '{}':", n);
                    log_error!("{}", v.2);
                }
            }
        }
    }
    Ok(())
}

fn parse_version_token(input: &str) -> (String, bool) {
    let chars = input.chars().collect::<Vec<char>>();
    if chars.len() > 2 {
        if chars[0] == '_' && chars[1] == ' ' {
            (String::from_iter(chars[2..].iter()), true)
        } else {
            (input.to_string(), false)
        }
    } else {
        (input.to_string(), false)
    }
}
