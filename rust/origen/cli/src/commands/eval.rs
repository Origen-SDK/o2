use clap::{App, AppSettings, Arg, Command, ArgMatches};
use super::_prelude::*;
use origen::Result;
use super::super::CommandHelp;
use indexmap::IndexMap;
use super::launch2;
use crate::{Extensions, Plugins};

pub const CMD_NAME: &'static str = "eval";
pub const BASE_CMD: &'static str = "eval";

gen_core_cmd_funcs!(
    BASE_CMD,
    "Evaluates statements in an Origen context",
    { |cmd: App<'a>| {
        cmd.visible_alias("e")
        .arg(
            Arg::new("code")
                .help("Statements to evaluate")
                .action(AppendArgs)
                .value_name("CODE")
                .multiple(true)
                .required(true)
        )
    }}
);

gen_simple_run_func!();
