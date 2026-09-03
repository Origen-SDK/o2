use super::_prelude::*;

pub const BASE_CMD: &'static str = "web";

gen_core_cmd_funcs!(
    BASE_CMD,
    "Build and view application documentation",
    { |cmd: App| { cmd.arg_required_else_help(true).visible_alias("w") } },
    core_subcmd!("build", "Build the application documentation", {
        |cmd: App| {
            cmd.visible_alias("b")
                .visible_alias("compile")
                .visible_alias("html")
                .arg(
                    Arg::new("view")
                        .long("view")
                        .help("Open the generated documentation after a successful build")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("clean")
                        .long("clean")
                        .help("Clean generated documentation before building")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("no-api")
                        .long("no-api")
                        .help("Skip Python and Rust API documentation generation")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("release")
                        .long("release")
                        .short('r')
                        .help("Release the generated documentation")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("archive")
                        .long("archive")
                        .short('a')
                        .help("Archive the generated documentation under the given ID")
                        .action(SetArg)
                        .value_name("ARCHIVE_ID"),
                )
                .arg(
                    Arg::new("as-release")
                        .long("as-release")
                        .help("Build with release checks without publishing")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("release-with-warnings")
                        .long("release-with-warnings")
                        .help("Allow a release build to complete with warnings")
                        .requires("release")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("sphinx-args")
                        .long("sphinx-args")
                        .help("Additional arguments passed to sphinx-build")
                        .action(SetArg)
                        .allow_hyphen_values(true)
                        .value_name("ARGS"),
                )
        }
    }),
    core_subcmd!("view", "Open previously generated documentation", {
        |cmd: App| cmd.visible_alias("v")
    }),
    core_subcmd!(
        "serve",
        "Build, watch, and serve application documentation",
        {
            |cmd: App| {
                cmd.visible_alias("s")
                .arg(
                    Arg::new("host")
                        .long("host")
                        .help("Host interface for the documentation server; auto binds all interfaces and advertises this machine's hostname")
                        .action(SetArg)
                        .default_value("auto")
                        .value_name("HOST"),
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .short('p')
                        .help("Port for the documentation server")
                        .action(SetArg)
                        .default_value("8000")
                        .value_name("PORT"),
                )
                .arg(
                    Arg::new("open")
                        .long("open")
                        .help("Open the documentation in the default browser")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("fast")
                        .long("fast")
                        .help("Skip AutoAPI, Rustdoc, and documentation subprojects")
                        .action(SetArgTrue),
                )
                .arg(
                    Arg::new("sphinx-args")
                        .long("sphinx-args")
                        .help("Additional arguments passed to sphinx-autobuild")
                        .action(SetArg)
                        .allow_hyphen_values(true)
                        .value_name("ARGS"),
                )
            }
        }
    ),
    core_subcmd!("clean", "Remove generated documentation", {
        |cmd: App| cmd.visible_alias("c")
    })
);

gen_simple_run_func!();
