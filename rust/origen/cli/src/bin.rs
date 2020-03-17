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
                .arg(Arg::with_name("targets")
                    .help("The name of the file(s) from targets/ to be set, added, or removed to the default target(s)")
                    .takes_value(true)
                    .value_name("TARGETS")
                    .multiple(true)
                )
                .arg(Arg::with_name("add")
                    .help("Adds the given target(s) to the current default target(s)")
                    .short("a")
                    .long("add")
                    .takes_value(false)
                    .conflicts_with("remove")
                )
                .arg(Arg::with_name("remove")
                    .help("Removes the given target(s) to the current default target(s)")
                    .short("r")
                    .long("remove")
                    .takes_value(false)
                    .conflicts_with("add")
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
                let a;
                if m.value_of("add").is_some() {
                    a = Some("add");
                } else if m.value_of("remove").is_some() {
                    a = Some("remove");
                } else {
                    a = None;
                }
                //commands::target::run(matches.value_of("target"));
                if let Some(targets) = m.values_of("targets") {
                    commands::target::run(Some(targets.collect()), a);
                } else {
                    commands::target::run(None, a);
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
