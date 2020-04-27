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
           .subcommand(SubCommand::with_name("web")
                .about("Create, Build, and View Web Documentation")
                .setting(AppSettings::ArgRequiredElseHelp)
                .visible_alias("w")
                .subcommand(SubCommand::with_name("build") // What I think this command should be called
                    .about("Builds the web documentation")
                    .visible_alias("b")
                    .visible_alias("compile") // What O1 thinks it should be called
                    .visible_alias("html") // What sphinx thinks it should be called
                    .arg(Arg::with_name("view")
                        .long("view")
                        .short("v")
                        .help("Launch your web browswer after the build")
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
"Additional arguments to pass to the 'sphinx-build' commmand
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
            Some("web") => {
                let cmd = matches.subcommand_matches("web").unwrap();
                //let subcommand = matches.get_matches();
                let subcmd = cmd.subcommand();
                let sub = subcmd.1.unwrap();
                match subcmd.0 {
                    "build" => {
                        let mut args = "from origen.boot import __origen__; __origen__('web:build', args={".to_string();
                        if sub.is_present("view") {
                            args.push_str("'view': True, ");
                        }
                        if sub.is_present("clean") {
                            args.push_str("'clean': True, ");
                        }
                        if sub.is_present("no-api") {
                            args.push_str("'no-api': True, ");
                        }
                        // if sub.is_present("pdf") {
                        //     args.push_str("'pdf': True, ");
                        // }
                        if sub.is_present("release") {
                            args.push_str("'release': True, ");
                        }
                        if sub.is_present("archive") {
                            if let Some(archive) = sub.value_of("archive") {
                                args.push_str(&format!("'archive': '{}', ", archive));
                            } else {
                                args.push_str(&format!("'archive': True"));
                            }
                        }
                        if let Some(s_args) = sub.value_of("sphinx-args") {
                            // Recall that this comes in as a single argument, potentially quoted to mimic multiple,
                            // but a single argument from the perspective here nonetheless
                            args.push_str(&format!("'sphinx-args': '{}', ", s_args));
                        }
                        args.push_str("}");
                        args.push_str(");");
                        python::run(&args);
                    },
                    "view" => {
                        commands::launch(
                            "web:view",
                            None,
                            &None,
                            None
                        )
                    },
                    "clean" => {
                        commands::launch(
                            "web:clean",
                            None,
                            &None,
                            None
                        )
                    }
                    _ => {}
                }
            }
            // Should never hit these
            None => {}
            _ => {}
        }
    }
}
