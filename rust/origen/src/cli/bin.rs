extern crate clap;

mod commands;

use clap::{App, Arg, SubCommand};
use core::CONFIG;

// This is the entry point for the Origen CLI tool
fn main() {
    if !CONFIG.is_app_present {
        /************************************************************************************/
        /******************** Global commands ***********************************************/
        /************************************************************************************/

        let matches = App::new("Origen, The Semiconductor Developer's Kit")
            .about(&*format!("CLI {}", CONFIG.origen_version))
            .arg(
                Arg::with_name("version")
                    .short("v")
                    .long("version")
                    .help("Display the Origen version"),
            )
            .get_matches();

        if matches.is_present("version") {
            commands::version::main();
        }

    /************************************************************************************/
    /******************** In application commands ***************************************/
    /************************************************************************************/
    } else {
        let matches = App::new("Origen, The Semiconductor Developer's Kit")
           .about(&*format!("CLI {}", CONFIG.origen_version))
           .arg(Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Display the Origen and application version")
           )

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
                    .help("The name of the file from target/ to be set as the default target")
                    .takes_value(true)
                    .value_name("TARGET")
                )
           )

           /************************************************************************************/
           .subcommand(SubCommand::with_name("environment")
                .about("Set/view the default environment")
                .visible_alias("e")
                .arg(Arg::with_name("environment")
                    .help("The name of the file from environment/ to be set as the default environment")
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

           .get_matches();

        if matches.is_present("version") {
            commands::version::main();
        } else {
            // You can also match on a subcommand's name
            match matches.subcommand_name() {
                Some("version") => commands::version::main(),
                None => println!("No subcommand was used"),
                _ => println!("Some other subcommand was used"),
            }
        }
    }
}
