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
use indexmap::map::IndexMap;
use origen::utility::version::to_pep440;
use origen::{LOGGER, STATUS};
use std::path::Path;

static VERBOSITY_HELP_STR: &str = "Terminal verbosity level e.g. -v, -vv, -vvv";
static VERBOSITY_KEYWORD_HELP_STR: &str = "Keywords for verbose listeners";

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
                origen::initialize(Some(verbosity), vec!(), None);
                args = args.drain(1..).collect();
            }
            // Commmand is not actually available outside an app, so just fall through
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
    origen::initialize(Some(verbosity), vec!(), exe);

    let version = match STATUS.is_app_present {
        true => format!(
            "Origen CLI: {}",
            to_pep440(&STATUS.origen_version.to_string()).unwrap_or("Error".to_string())
        ),
        false => format!(
            "Origen: {}",
            to_pep440(&STATUS.origen_version.to_string()).unwrap_or("Error".to_string())
        ),
    };
    if STATUS.app.is_some() {
        origen::core::application::config::Config::check_defaults(
            &STATUS.app.as_ref().unwrap().root,
        );
    }

    let mut app = App::new("")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::DisableVersion)
        .before_help("Origen, The Semiconductor Developer's Kit")
        .after_help("See 'origen <command> -h' for more information on a specific command.")
        .version(&*version)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help(VERBOSITY_HELP_STR)
        )
        .arg(
            Arg::with_name("verbosity_keywords")
                .short("k")
                .multiple(true)
                .takes_value(true)
                .global(true)
                .help(VERBOSITY_KEYWORD_HELP_STR)
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
        //************************************************************************************/
        let proj_help = "Manage multi-repository project areas and workspaces";
        origen_commands.push(CommandHelp {
            name: "proj".to_string(),
            help: proj_help.to_string(),
            shortcut: None,
        });

        app = app
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

        //************************************************************************************/
        let new_help = "Create a new Origen application";
        origen_commands.push(CommandHelp {
            name: "new".to_string(),
            help: new_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            SubCommand::with_name("new").about(new_help).arg(
                Arg::with_name("name")
                    .help("The lowercased and underscored name of the new application")
                    .takes_value(true)
                    .required(true)
                    .number_of_values(1)
                    .value_name("NAME"),
            )
            .arg(Arg::with_name("setup")
                .help("Don't create the new app's virtual environment after building (need to manually run 'origen env setup' within the new app workspace before using it in that case)")
                .long("no-setup")
                .required(false)
                .takes_value(false)
            ),
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

    if STATUS.is_origen_present || STATUS.is_app_in_origen_dev_mode {
        let build_help = match STATUS.is_origen_present {
            true => "Build and publish Origen, builds the pyapi Rust package by default",
            false => "Build Origen",
        };

        origen_commands.push(CommandHelp {
            name: "build".to_string(),
            help: build_help.to_string(),
            shortcut: None,
        });

        //************************************************************************************/
        let mut sub = SubCommand::with_name("build").about(build_help).arg(
            Arg::with_name("cli")
                .long("cli")
                .required(false)
                .takes_value(false)
                .display_order(1)
                .help("Build the CLI (instead of the Python API)"),
        );

        if STATUS.is_origen_present {
            sub = sub
                .arg(
                    Arg::with_name("release")
                        .long("release")
                        .required(false)
                        .takes_value(false)
                        .display_order(1)
                        .help("Build a release version (applied by default with --publish and only applicable to Rust builds)"),
                )
                .arg(
                    Arg::with_name("target")
                        .long("target")
                        .required(false)
                        .takes_value(true)
                        .display_order(1)
                        .help("The Rust h/ware target (passed directly to Cargo build)"),
                )
                .arg(
                    Arg::with_name("python")
                        .long("python")
                        .required(false)
                        .takes_value(false)
                        .display_order(1)
                        .help("Build the pure Python package (instead of the Python API)"),
                )
                .arg(
                    Arg::with_name("publish")
                        .long("publish")
                        .required(false)
                        .takes_value(false)
                        .display_order(1)
                        .help("Publish packages (e.g. to PyPI) after building"),
                )
                .arg(
                    Arg::with_name("dry_run")
                        .long("dry-run")
                        .required(false)
                        .takes_value(false)
                        .display_order(1)
                        .help("Use with --publish to perform a full dry run of the publishable build without actually publishing it"),
                )
                .arg(
                    Arg::with_name("version")
                        .long("version")
                        .required(false)
                        .takes_value(true)
                        .value_name("VERSION")
                        .display_order(1)
                        .help("Set the version (of all components) to the given value"),
                );
        }

        app = app.subcommand(sub);
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
        let new_help = "Generate a new block, flow, pattern, etc. for your application";
        origen_commands.push(CommandHelp {
            name: "new".to_string(),
            help: new_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            SubCommand::with_name("new")
            .about(new_help)
            .setting(AppSettings::ArgRequiredElseHelp)
            .subcommand(SubCommand::with_name("dut")
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
                .arg(Arg::with_name("name")
                    .takes_value(true)
                    .required(false)
                    .help("The name of the new DUT")
                    .value_name("NAME")
                )
            )
            .subcommand(SubCommand::with_name("block")
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
                .arg(Arg::with_name("name")
                    .takes_value(true)
                    .required(true)
                    .help("The name of the new block, including its parents if applicable")
                    .value_name("NAME")
                )
                .arg(
                    Arg::with_name("parent")
                        .short("p")
                        .long("parent")
                        .help("Create the new block nested within this existing block")
                        .takes_value(true)
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
                )
                .arg(
                    Arg::with_name("debug")
                        .long("debug")
                        .short("d")
                        .help("Enable Python caller tracking for debug (takes longer to execute)")
                        .takes_value(false),
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
                .arg(
                    Arg::with_name("full-paths")
                        .long("full-paths")
                        .short("f")
                        .help("Display targets' full paths")
                        .takes_value(false),
                )
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
                        )
                        .arg(
                            Arg::with_name("full-paths")
                                .long("full-paths")
                                .short("f")
                                .help("Display targets' full paths")
                                .takes_value(false),
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
                        )
                        .arg(
                            Arg::with_name("full-paths")
                                .long("full-paths")
                                .short("f")
                                .help("Display targets' full paths")
                                .takes_value(false),
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
                        )
                        .arg(
                            Arg::with_name("full-paths")
                                .long("full-paths")
                                .short("f")
                                .help("Display targets' full paths")
                                .takes_value(false),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("default")
                        .about("Activates the default target(s) while deactivating all others")
                        .visible_alias("d")
                        .arg(
                            Arg::with_name("full-paths")
                                .long("full-paths")
                                .short("f")
                                .help("Display targets' full paths")
                                .takes_value(false),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("view")
                        .about("Views the currently activated target(s)")
                        .visible_alias("v")
                        .arg(
                            Arg::with_name("full-paths")
                                .long("full-paths")
                                .short("f")
                                .help("Display targets' full paths")
                                .takes_value(false),
                        ),
                ),
        );

        /************************************************************************************/
        let t_help = "Create, Build, and View Web Documentation";
        origen_commands.push(CommandHelp {
            name: "web".to_string(),
            help: t_help.to_string(),
            shortcut: Some("w".to_string()),
        });
        app = app.subcommand(
            SubCommand::with_name("web")
                .about(t_help)
                .setting(AppSettings::ArgRequiredElseHelp)
                .visible_alias("w")
                .subcommand(
                    SubCommand::with_name("build") // What I think this command should be called
                        .about("Builds the web documentation")
                        .visible_alias("b")
                        .visible_alias("compile") // If coming from O1
                        .visible_alias("html") // If coming from Sphinx and using quickstart's Makefile
                        .arg(
                            Arg::with_name("view")
                                .long("view")
                                .help("Launch your web browser after the build")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("clean")
                                .long("clean")
                                .help(
                                    "Clean up directories from previous builds and force a rebuild",
                                )
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("release")
                                .long("release")
                                .short("r")
                                .help("Release (deploy) the resulting web pages")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("archive")
                                .long("archive")
                                .short("a")
                                .help("Archive the resulting web pages after building")
                                .takes_value(true)
                                .multiple(false)
                                .min_values(0),
                        )
                        .arg(
                            Arg::with_name("as-release")
                                .long("as-release")
                                .help("Build webpages with release checks")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("release-with-warnings")
                                .long("release-with-warnings")
                                .help("Release webpages even if warnings persists")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("no-api")
                                .long("no-api")
                                .help("Skip building the API")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("sphinx-args")
                                .long("sphinx-args")
                                .help(
                                    "Additional arguments to pass to the 'sphinx-build' command
  Argument will passed as a single string and appended to the build command
  E.g.: 'origen web build --sphinx-args \"-q -D my_config_define=1\"'
     -> 'sphinx-build <source_dir> <output_dir> -q -D my_config_define=1'",
                                )
                                .takes_value(true)
                                .multiple(false)
                                .allow_hyphen_values(true),
                        ), // .arg(Arg::with_name("pdf")
                           //     .long("pdf")
                           //     .help("Create a PDF of resulting web pages")
                           //     .takes_value(false)
                           // )
                )
                .subcommand(
                    SubCommand::with_name("view")
                        .about("Launches your web browser to view previously built documentation")
                        .visible_alias("v"),
                )
                .subcommand(
                    SubCommand::with_name("clean")
                        .about("Cleans the output directory and all cached files"),
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
        let app_help = "Commands for packaging and releasing your application";
        origen_commands.push(CommandHelp {
            name: "app".to_string(),
            help: app_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(
            SubCommand::with_name("app")
                .about(app_help)
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("package")
                        .about("Build the app into a Python package (a wheel)"),
                ),
        );

        /************************************************************************************/
        let env_help = "Manage your application's Origen/Python environment (dependencies, etc.)";
        origen_commands.push(CommandHelp {
            name: "env".to_string(),
            help: env_help.to_string(),
            shortcut: None,
        });
        app = app.subcommand(SubCommand::with_name("env").about(env_help)
            .setting(AppSettings::ArgRequiredElseHelp)
            .subcommand(
                SubCommand::with_name("setup")
                    .about("Setup your application's Python environment for the first time in a new workspace, this will install dependencies per the poetry.lock file")
                    .arg(Arg::with_name("origen")
                            .long("origen")
                            .help("The path to a local version of Origen to use (to develop Origen)")
                            .takes_value(true),
                    ),
            )
            .subcommand(
                SubCommand::with_name("update")
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
            SubCommand::with_name("exec")
                .about(exec_help)
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::DisableVersion)
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(
                    Arg::with_name("cmd")
                        .help("The command to be run")
                        .takes_value(true)
                        .required(true)
                        .value_name("COMMAND"),
                )
                .arg(
                    Arg::with_name("args")
                        .help("Arguments to be passed to the command")
                        .takes_value(true)
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
        // This defines the application commands
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
    -h, --help                Prints help information
    -v                        {}
    -vk, --verbosity_keywords {}

CORE COMMANDS:
",
        version,
        VERBOSITY_HELP_STR,
        VERBOSITY_KEYWORD_HELP_STR
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
    if let Some(keywords) = matches.values_of("verbosity_keywords") {
        let _ = LOGGER.set_verbosity_keywords(keywords.map( |k| k.to_string()).collect());
    }

    match matches.subcommand_name() {
        Some("app") => commands::app::run(matches.subcommand_matches("app").unwrap()),
        Some("env") => commands::env::run(matches.subcommand_matches("env").unwrap()),
        Some("fmt") => commands::fmt::run(),
        Some("new") => commands::new::run(matches.subcommand_matches("new").unwrap()),
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
                m.is_present("debug"),
                None,
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
                false,
                None,
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
                    s.is_present("full-paths"),
                )
            } else {
                commands::target::run(None, None, m.is_present("full-paths"));
            }
        }
        Some("web") => {
            let cmd = matches.subcommand_matches("web").unwrap();
            let subcmd = cmd.subcommand();
            let sub = subcmd.1.unwrap();
            match subcmd.0 {
                "build" => {
                    let mut args = IndexMap::new();
                    if sub.is_present("view") {
                        args.insert("view", "True".to_string());
                    }
                    if sub.is_present("clean") {
                        args.insert("clean", "True".to_string());
                    }
                    if sub.is_present("no-api") {
                        args.insert("no-api", "True".to_string());
                    }
                    if sub.is_present("as-release") {
                        args.insert("as-release", "True".to_string());
                    }
                    if sub.is_present("release-with-warnings") {
                        args.insert("release-with-warnings", "True".to_string());
                    }
                    if sub.is_present("release") {
                        args.insert("release", "True".to_string());
                    }
                    if sub.is_present("archive") {
                        if let Some(archive) = sub.value_of("archive") {
                            args.insert("archive", format!("'{}'", archive));
                        } else {
                            args.insert("archive", "True".to_string());
                        }
                    }
                    if let Some(s_args) = sub.value_of("sphinx-args") {
                        // Recall that this comes in as a single argument, potentially quoted to mimic multiple,
                        // but a single argument from the perspective here nonetheless
                        args.insert("sphinx-args", format!("'{}'", s_args));
                    }
                    commands::launch(
                        "web:build",
                        if let Some(targets) = cmd.values_of("target") {
                            Some(targets.collect())
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
                    origen_version = "Unknown".to_string();
                }

                let app_version = match origen::app().unwrap().version() {
                    Err(e) => {
                        log_error!("{}", e);
                        "Error".to_string()
                    }
                    Ok(v) => format!("{}", v),
                };

                if STATUS.is_app_in_origen_dev_mode {
                    println!(
                        "App:    {}\nOrigen: {} (from {})\nCLI:    {}",
                        to_pep440(&app_version).unwrap_or("Error".to_string()),
                        to_pep440(&origen_version).unwrap_or("Error".to_string()),
                        STATUS.origen_wksp_root.display(),
                        to_pep440(&STATUS.origen_version.to_string())
                            .unwrap_or("Error".to_string())
                    );
                } else {
                    println!(
                        "App:    {}\nOrigen: {}\nCLI:    {}",
                        to_pep440(&app_version).unwrap_or("Error".to_string()),
                        to_pep440(&origen_version).unwrap_or("Error".to_string()),
                        to_pep440(&STATUS.origen_version.to_string())
                            .unwrap_or("Error".to_string())
                    );
                }
            } else {
                println!(
                    "Origen: {}",
                    to_pep440(&STATUS.origen_version.to_string()).unwrap_or("Error".to_string())
                );
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
