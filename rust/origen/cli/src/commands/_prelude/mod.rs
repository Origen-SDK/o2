pub mod clap_arg_actions;

pub use clap::{App, AppSettings, Arg, Command};
pub use origen::Result;
pub use indexmap::IndexMap;
pub use super::launch_as;
pub use crate::framework::{
    Extensions, Plugins, add_verbosity_opts,
    CmdHelps, CmdHelp, CmdSrc,
};
pub use crate::framework::core_cmds::SubCmd;
pub use crate::{output_dir_opt, ref_dir_opt};

// TODO clap4.0 remove after update to next clap version
pub type RunInput<'a> = &'a clap::ArgMatches;

pub use clap_arg_actions::*;
pub use crate::{
    gen_core_cmd_funcs, gen_core_cmd_funcs__no_exts__no_app_opts,
    core_subcmd, core_subcmd__no_exts__no_app_opts,
    gen_simple_run_func,
    print_subcmds_available_msg, unreachable_invalid_subc,
};