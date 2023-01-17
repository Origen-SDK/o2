use super::_prelude::*;

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
