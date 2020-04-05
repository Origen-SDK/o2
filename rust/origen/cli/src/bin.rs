#[macro_use]
extern crate lazy_static;

mod commands;
mod python;

use clap::{App, AppSettings, Arg, SubCommand};
use origen::STATUS;

// This is the entry point for the Origen CLI tool
fn main() {
    let about = format!("CLI {}", STATUS.origen_version);
    let mut app = App::new("Origen, The Semiconductor Developer's Kit")
        .setting(AppSettings::ArgRequiredElseHelp)
        .after_help("See 'origen <command> -h' for more information on a specific command.")
        .about(&*about)
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Display the Origen and application version"),
        );

    /************************************************************************************/
    /******************** Global commands ***********************************************/
    /************************************************************************************/
    //app = app
    //    //************************************************************************************/
    //    .subcommand(
    //        SubCommand::with_name("check")
    //            .about("Check if your environment is setup correctly to run Origen"),
    //    );

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
           .subcommand(SubCommand::with_name("setup")
                .about("Setup your application's Python environment"),
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("utility")
                .about("Various utility functions and helpers")
                .subcommand(SubCommand::with_name("sources")
                    .about("Locates source files for generation or compilation")
                    .arg(Arg::with_name("pattern")
                        .help("Locate pattern source only")
                    // <Add more options here>
                    )
                )
           )

      }

    let matches = app.get_matches();

    if matches.is_present("version") {
        commands::version::run();
    } else {
        match matches.subcommand_name() {
            Some("setup") => commands::setup::run(),
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
                            None => None
                        }
                    )
                } else {
                    commands::target::run(None, None);
                }
            }
            Some("mode") => {
                let matches = matches.subcommand_matches("mode").unwrap();
                commands::mode::run(matches.value_of("mode"));
            }
            // Should never hit these
            None => {}
            _ => {}
        }
    }
}
