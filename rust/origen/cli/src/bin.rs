#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate origen;

mod commands;
mod python;

use clap::{App, AppSettings, Arg, SubCommand};
use indexmap::map::IndexMap;
use origen::{LOGGER, STATUS};

// This is the entry point for the Origen CLI tool
fn main() {
    let version = match STATUS.is_app_present {
        false => STATUS.origen_version.to_string(),
        true => format!(
            "CLI:    {}\n Origen: {}\n App:    {}",
            STATUS.origen_version, "TBD", "TBD"
        ),
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

    /************************************************************************************/
    /******************** Global only commands ******************************************/
    /************************************************************************************/
    if !STATUS.is_app_present {
        app = app
            //************************************************************************************/
            .subcommand(
                SubCommand::with_name("proj")
                    .display_order(1)
                    .about("Manage multi-repository project areas and workspaces")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .subcommand(SubCommand::with_name("init")
                        .display_order(1)
                        .about("Initialize a new project directory (create an initial project BOM)")
                        .arg(Arg::with_name("dir")
                            .help("The path to the project directory to initialize (PWD will be used by default if not given)")
                            .value_name("DIR")
                        )
                    )
                    .subcommand(SubCommand::with_name("create")
                        .display_order(2)
                        .about("Create a new project workspace from the project BOM")
                        .arg(Arg::with_name("path")
                            .help("The path to the new workspace directory")
                            .value_name("PATH")
                            .required(true)
                        )
                    )
                    .subcommand(SubCommand::with_name("update")
                        .display_order(3)
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
                            .help("Update the workspace links only, leaving all packages unmodified")
                        )
                        .arg(Arg::with_name("exclude")
                            .short("e")
                            .long("exclude")
                            .help("Exclude the given package reference, supply either a package ID or a path to a workspace directory")
                            .takes_value(true)
                            .multiple(true)
                            .number_of_values(1)
                            .value_name("PACKAGE")
                        )
                        .arg(Arg::with_name("packages")
                            .value_name("PACKAGES")
                            .multiple(true)
                            .help("The packages to be updated, supply either package IDs or paths to workspace directories")
                        )
                    )
                    .subcommand(SubCommand::with_name("bom")
                        .display_order(4)
                        .about("View the active BOM in the current or given directory")
                        .arg(Arg::with_name("dir")
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
        let msg = match STATUS.is_origen_present {
            true => "Nicely format all Rust and Python files",
            false => "Nicely format all of your application's Python files",
        };
        app = app
            //************************************************************************************/
            .subcommand(SubCommand::with_name("fmt").about(msg));
    }

    /************************************************************************************/
    /******************** In application commands ***************************************/
    /************************************************************************************/
    if STATUS.is_app_present {
        app = app

           /************************************************************************************/
           .subcommand(SubCommand::with_name("interactive")
                .about("Start an Origen console to interact with the DUT")
                .visible_alias("i")
                .arg(Arg::with_name("target")
                    .short("t")
                    .long("target")
                    .help("Override the default target currently set by the workspace")
                    .takes_value(true)
                    .use_delimiter(true)
                    .multiple(true)
                    .number_of_values(1)
                    .value_name("TARGET")
                )
                .arg(Arg::with_name("mode")
                    .short("m")
                    .long("mode")
                    .help("Override the default execution mode currently set by the workspace")
                    .takes_value(true)
                    .value_name("MODE")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("generate")
                .about("Generate patterns or test programs")
                .visible_alias("g")
                .arg(Arg::with_name("files")
                    .help("The name of the file(s) to be generated")
                    .takes_value(true)
                    .value_name("FILES")
                    .multiple(true)
                    .required(true)
                )
                .arg(Arg::with_name("target")
                    .short("t")
                    .long("target")
                    .help("Override the default target currently set by the workspace")
                    .takes_value(true)
                    .use_delimiter(true)
                    .multiple(true)
                    .number_of_values(1)
                    .value_name("TARGET")
                )
                .arg(Arg::with_name("mode")
                    .short("m")
                    .long("mode")
                    .help("Override the default execution mode currently set by the workspace")
                    .takes_value(true)
                    .value_name("MODE")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("compile")
                .about("Compile templates")
                .visible_alias("c")
                .arg(Arg::with_name("files")
                    .help("The name of the file(s) to be generated")
                    .takes_value(true)
                    .value_name("FILES")
                    .multiple(true)
                    .required(true)
                )
                .arg(Arg::with_name("target")
                    .short("t")
                    .long("target")
                    .help("Override the default target currently set by the workspace")
                    .takes_value(true)
                    .use_delimiter(true)
                    .multiple(true)
                    .number_of_values(1)
                    .value_name("TARGET")
                )
                .arg(Arg::with_name("mode")
                    .short("m")
                    .long("mode")
                    .help("Override the default execution mode currently set by the workspace")
                    .takes_value(true)
                    .value_name("MODE")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("target")
                .about("Set/view the default target")
                .visible_alias("t")
                .subcommand(SubCommand::with_name("add")
                    .about("Activates the given target(s)")
                    .visible_alias("a")
                    .arg(Arg::with_name("targets")
                        .help("Targets to be activated")
                        .takes_value(true)
                        .value_name("TARGETS")
                        .multiple(true)
                        .required(true)
                    )
                )
                .subcommand(SubCommand::with_name("remove")
                    .about("Deactivates the given target(s)")
                    .visible_alias("r")
                    .arg(Arg::with_name("targets")
                        .help("Targets to be deactivated")
                        .takes_value(true)
                        .value_name("TARGETS")
                        .multiple(true)
                        .required(true)
                    )
                )
                .subcommand(SubCommand::with_name("set")
                    .about("Activates the given target(s) while deactivating all others")
                    .visible_alias("s")
                    .arg(Arg::with_name("targets")
                        .help("Targets to be set")
                        .takes_value(true)
                        .value_name("TARGETS")
                        .multiple(true)
                        .required(true)
                    )
                )
                .subcommand(SubCommand::with_name("default")
                    .about("Activates the default target(s) while deactivating all others")
                    .visible_alias("d")
                )
                .subcommand(SubCommand::with_name("view")
                    .about("Views the currently activated target(s)")
                    .visible_alias("v")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("mode")
                .about("Set/view the default execution mode")
                .visible_alias("m")
                .arg(Arg::with_name("mode")
                    .help("The name of the mode to be set as the default mode")
                    .takes_value(true)
                    .value_name("MODE")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("web")
                .about("Create, Build, and View Web Documentation")
                .setting(AppSettings::ArgRequiredElseHelp)
                .visible_alias("w")
                .subcommand(SubCommand::with_name("build") // What I think this command should be called
                    .about("Builds the web documentation")
                    .visible_alias("b")
                    .visible_alias("compile") // If coming from O1
                    .visible_alias("html") // If coming from Sphinx and using quickstart's Makefile
                    .arg(Arg::with_name("view")
                        .long("view")
                        .help("Launch your web browser after the build")
                        .takes_value(false)
                    )
                    .arg(Arg::with_name("clean")
                        .long("clean")
                        .help("Clean up directories from previous builds and force a rebuild")
                        .takes_value(false)
                    )
                    .arg(Arg::with_name("release")
                        .long("release")
                        .short("r")
                        .help("Release (deploy) the resulting web pages")
                        .takes_value(false)
                    )
                    .arg(Arg::with_name("archive")
                        .long("archive")
                        .short("a")
                        .help(
"Archive the resulting web pages after building"
                    )
                        .takes_value(true)
                        .multiple(false)
                        .min_values(0)
                    )
                    .arg(Arg::with_name("no-api")
                        .long("no-api")
                        .help("Skip building the API")
                        .takes_value(false)
                    )
                    .arg(Arg::with_name("sphinx-args")
                        .long("sphinx-args")
                        .help(
"Additional arguments to pass to the 'sphinx-build' command
  Argument will passed as a single string and appended to the build command
  E.g.: 'origen web build --sphinx-args \"-q -D my_config_define=1\"'
     -> 'sphinx-build <source_dir> <output_dir> -q -D my_config_define=1'"
                        )
                        .takes_value(true)
                        .multiple(false)
                        .allow_hyphen_values(true)
                    )
                    // .arg(Arg::with_name("pdf")
                    //     .long("pdf")
                    //     .help("Create a PDF of resulting web pages")
                    //     .takes_value(false)
                    // )
                )
                .subcommand(SubCommand::with_name("view")
                    .about("Launches your web browser to view previously built documentation")
                    .visible_alias("v")
                )
                .subcommand(SubCommand::with_name("clean")
                    .about("Cleans the output directory and all cached files")
                )
            )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("setup")
                .about("Setup your application's Python environment in a new workspace, this will install dependencies per the poetry.lock file"),
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("update")
                .about("Update your application's Python dependencies according to the latest pyproject.toml file"),
           )
    }

    let matches = app.get_matches();

    let _ = LOGGER.set_verbosity(matches.occurrences_of("verbose") as u8);

    match matches.subcommand_name() {
        Some("setup") => commands::setup::run(),
        Some("update") => commands::update::run(),
        Some("fmt") => commands::fmt::run(),
        Some("proj") => commands::proj::run(matches.subcommand_matches("proj").unwrap()),
        Some("interactive") => {
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
                )
            } else {
                commands::target::run(None, None);
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
                    // if sub.is_present("pdf") {
                    //     args.push_str("'pdf': True, ");
                    // }
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
                        Some(args)
                    )
                },
                "view" => {
                    commands::launch(
                        "web:view",
                        None,
                        &None,
                        None,
                        None
                    )
                },
                "clean" => {
                    commands::launch(
                        "web:clean",
                        None,
                        &None,
                        None,
                        None
                    )
                }
                _ => {}
            }
        }
        Some("mode") => {
            let matches = matches.subcommand_matches("mode").unwrap();
            commands::mode::run(matches.value_of("mode"));
        }
        // To get here means the user has typed "origen -v", which officially means
        // verbosity level 1 with no command, but this is what they really mean
        None => println!(" {}", version),
        _ => unreachable!(),
    }
}
