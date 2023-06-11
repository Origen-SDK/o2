use super::_prelude::*;

pub const BASE_CMD: &'static str = "credentials";

gen_core_cmd_funcs!(
    BASE_CMD,
    "Set or clear user credentials",
    { |cmd: App<'a>| { cmd.setting(AppSettings::ArgRequiredElseHelp) }},
    core_subcmd!("set", "Set the current user's password", { |cmd: App| {
        cmd.arg(
            Arg::new("all")
                .help("Set the password for all datasets")
                .action(SetArgTrue)
                .required(false)
                .long("all")
                .short('a'),
        ).arg(
            Arg::new("datasets")
                .help("Specify the dataset to set the password for")
                .action(AppendArgs)
                .required(false)
                .multiple(true)
                .conflicts_with("all")
                .long("dataset")
                .short('d'),
        )
    }}),
    core_subcmd!("clear", "Clear the user's password", { |cmd: App| {
        cmd.arg(
            Arg::new("all")
                .help("Clear the password for all datasets")
                .action(SetArgTrue)
                .required(false)
                .long("all")
                .short('a'),
        ).arg(
            Arg::new("datasets")
                .help("Specify the dataset to clear the password for")
                .action(AppendArgs)
                .required(false)
                .conflicts_with("all")
                .multiple(true)
                .long("datasets")
                .short('d'),
        )
    }})
);

gen_simple_run_func!();
