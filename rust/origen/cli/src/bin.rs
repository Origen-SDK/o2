#[macro_use]
extern crate lazy_static;

mod commands;
mod python;

use clap::{App, Arg, SubCommand};
use origen::STATUS;

// This is the entry point for the Origen CLI tool
fn main() {
    let about = format!("CLI {}", STATUS.origen_version);
    let mut app = App::new("Origen, The Semiconductor Developer's Kit")
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
                    .value_name("TARGET")
                )
                .arg(Arg::with_name("environment")
                    .short("e")
                    .long("environment")
                    .help("Override the default environment currently set by the workspace")
                    .takes_value(true)
                    .value_name("ENVIRONMENT")
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
                )
                .arg(Arg::with_name("target")
                    .short("t")
                    .long("target")
                    .help("Override the default target currently set by the workspace")
                    .takes_value(true)
                    .value_name("TARGET")
                )
                .arg(Arg::with_name("environment")
                    .short("e")
                    .long("environment")
                    .help("Override the default environment currently set by the workspace")
                    .takes_value(true)
                    .value_name("ENVIRONMENT")
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
                )
                .arg(Arg::with_name("target")
                    .short("t")
                    .long("target")
                    .help("Override the default target currently set by the workspace")
                    .takes_value(true)
                    .value_name("TARGET")
                )
                .arg(Arg::with_name("environment")
                    .short("e")
                    .long("environment")
                    .help("Override the default environment currently set by the workspace")
                    .takes_value(true)
                    .value_name("ENVIRONMENT")
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
                .arg(Arg::with_name("target")
                    .help("The name of the file from targets/ to be set as the default target")
                    .takes_value(true)
                    .value_name("TARGET")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("environment")
                .about("Set/view the default environment")
                .visible_alias("e")
                .arg(Arg::with_name("environment")
                    .help("The name of the file from environments/ to be set as the default environment")
                    .takes_value(true)
                    .value_name("ENVIRONMENT")
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
        commands::version::main();
    } else {
        match matches.subcommand_name() {
            Some("setup") => commands::setup::main(),
            Some("interactive") => commands::interactive::main(),
            Some("target") => {
                let matches = matches.subcommand_matches("target").unwrap();
                if matches.is_present("target") {
                    let name = matches.value_of("target").unwrap();
                    commands::target::main(Some(name));
                } else {
                    commands::target::main(None);
                }
            }
            Some("environment") => {
                let matches = matches.subcommand_matches("environment").unwrap();
                if matches.is_present("environment") {
                    let name = matches.value_of("environment").unwrap();
                    commands::environment::main(Some(name));
                } else {
                    commands::environment::main(None);
                }
            }
            None => {}
            _ => {}
        }
    }
}
