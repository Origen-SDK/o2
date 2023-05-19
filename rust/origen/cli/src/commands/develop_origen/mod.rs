mod build;
mod fmt;

use origen::Result;
use super::_prelude::*;
pub const BASE_CMD: &'static str = "develop_origen";

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Commands to assist with Origen core development",
    { |cmd: App<'a>| { 
        cmd.arg_required_else_help(true).visible_alias("origen")
    }},
    build::build_cmd(),
    fmt::fmt_cmd()
);

pub(crate) fn run(mut invocation: &clap::ArgMatches) -> Result<()> {
    let (n, subcmd) = invocation.subcommand().unwrap();
    match n {
        build::BASE_CMD => build::run(subcmd),
        fmt::BASE_CMD => fmt::run(),
        _ => unreachable_invalid_subc!(n)
    }
}