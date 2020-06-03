#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate origen;

mod commands;
mod python;

use clap::{App, AppSettings, Arg, SubCommand};
use origen::{LOGGER, STATUS};

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
                .arg(Arg::with_name("output_dir")
                    .short("o")
                    .long("output-dir")
                    .help("Override the default output directory (<APP ROOT>/output)")
                    .takes_value(true)
                    .value_name("OUTPUT_DIR")
                )
                .arg(Arg::with_name("reference_dir")
                    .short("r")
                    .long("reference-dir")
                    .help("Override the default reference directory (<APP ROOT>/.ref)")
                    .takes_value(true)
                    .value_name("REFERENCE_DIR")
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
           .subcommand(SubCommand::with_name("setup")
                .about("Setup your application's Python environment"),
           )
    }

    let matches = app.get_matches();

    let _ = LOGGER.set_verbosity(matches.occurrences_of("verbose") as u8);

    match matches.subcommand_name() {
        Some("setup") => commands::setup::run(),
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
        // To get here means the user has typed "origen -v", which officially means
        // verbosity level 1 with no command, but this is what they really mean
        None => {
            if STATUS.is_app_present {
                // Run a short command line operation to get the Origen version back from the Python domain
                let cmd = "from origen.boot import __origen__; __origen__('_version_');";
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

                let app_version = "TBD";

                println!(
                    "App:    {}\nOrigen: {}\nCLI:    {}",
                    app_version, origen_version, STATUS.origen_version
                );
            } else {
                println!("Origen: {}", STATUS.origen_version);
            }
        }
        _ => unreachable!(),
    }
}
