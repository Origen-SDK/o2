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
                .required_unless_present("scripts")
        )
        .arg(
            Arg::new("scripts")
                .help("Evaluate from script files")
                .long("scripts")
                .short('s')
                .visible_alias("files")
                .visible_short_alias('f')
                .action(AppendArgs)
                .value_name("SCRIPTS")
                .multiple(true)
                .use_value_delimiter(true)
                .require_value_delimiter(true)
        )
    }}
);

gen_simple_run_func!();
