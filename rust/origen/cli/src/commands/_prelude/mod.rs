pub mod clap_arg_actions;

pub use clap::{Arg, Command};
pub type App = Command;
pub use super::launch_as;
pub use crate::framework::core_cmds::SubCmd;
pub use crate::framework::{CmdHelps, CmdSrc, Extensions, Plugins};
pub use crate::origen_fe_available;
pub use crate::{output_dir_opt, ref_dir_opt};
pub use crate::{req_sv_arg, sv_opt};
pub use indexmap::IndexMap;
pub use origen::Result;

pub type RunInput<'a> = &'a clap::ArgMatches;

pub use crate::{
    core_subcmd, core_subcmd__no_exts__no_app_opts, gen_core_cmd_funcs,
    gen_core_cmd_funcs__no_exts__no_app_opts, gen_simple_run_func, print_subcmds_available_msg,
    unreachable_invalid_subc,
};
pub use clap_arg_actions::*;
