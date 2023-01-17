use crate::commands::_prelude::*;

pub const BASE_CMD: &'static str = "interactive";

gen_core_cmd_funcs!(
    BASE_CMD,
    "Start an Origen console to interact with the DUT",
    { |cmd: App<'a>| {
        cmd.visible_alias("i")
    }}
);

crate::gen_simple_run_func!();
