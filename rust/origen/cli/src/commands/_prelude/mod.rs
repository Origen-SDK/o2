pub mod clap_arg_actions;

pub use clap::{App, AppSettings, Arg, Command};
pub use origen::Result;
pub use super::super::CommandHelp;
pub use indexmap::IndexMap;
pub use super::{launch, launch_simple, launch_as};
pub use crate::framework::{Extensions, Plugins};
pub use crate::framework::{CmdHelps, CmdHelp, CmdSrc};

// FOR_PR remove this
pub type RunInput<'a> = &'a clap::ArgMatches;

pub use clap_arg_actions::*;
pub use crate::{gen_core_cmd_funcs, core_subcmd, gen_simple_run_func};