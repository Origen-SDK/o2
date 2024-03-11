mod build;
mod fmt;
mod update_supported_python;
mod publish;

use origen::Result;
use super::_prelude::*;
pub const BASE_CMD: &'static str = "develop_origen";

lazy_static! {
    static ref GH_OWNER: &'static str = "Origen-SDK";
    static ref GH_REPO: &'static str = "o2";
    static ref PUBLISH_BRANCH: &'static str = "master";
    static ref PUBLISH_WORKFLOW: &'static str = "publish.yml";
    static ref ORIGEN_OM_REQ_PATH: [&'static str; 4] = ["tool", "poetry", "dependencies", "origen_metal"];
}

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Commands to assist with Origen core development",
    { |cmd: App<'a>| { 
        cmd.arg_required_else_help(true).visible_alias("origen")
    }},
    build::build_cmd(),
    fmt::fmt_cmd(),
    update_supported_python::update_supported_python_cmd(),
    publish::publish_cmd()
);

pub(crate) fn run(invocation: &clap::ArgMatches) -> Result<()> {
    let (n, subcmd) = invocation.subcommand().unwrap();
    match n {
        build::BASE_CMD => build::run(subcmd),
        fmt::BASE_CMD => fmt::run(),
        update_supported_python::BASE_CMD => update_supported_python::run(subcmd),
        publish::BASE_CMD => publish::run(subcmd),
        _ => unreachable_invalid_subc!(n)
    }
}