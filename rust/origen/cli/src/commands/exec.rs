use super::_prelude::*;
use crate::python::{add_origen_env, PYTHON_CONFIG};
use std::borrow::Cow;

pub const BASE_CMD: &'static str = "exec";
pub const EXEC_HELP: &'static str =
    "Execute a command within your Origen/Python environment (e.g. origen exec pytest)";

pub(crate) fn run_pre_phase(invocation: &clap::ArgMatches) -> Result<i32> {
    let mut cmd = PYTHON_CONFIG.uv_command()?;
    cmd.arg("run");
    cmd.arg("--no-sync");
    cmd.arg("--no-editable");
    cmd.arg("--");
    let cmd_name = invocation.get_one::<String>("cmd").unwrap();

    cmd.arg(cmd_name);
    if let Some(args) = invocation.get_many::<String>("args") {
        cmd.args(args.filter(|arg| arg.as_str() != "--"));
    }
    log_debug!(
        "Running Command (from Exec): {} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<Cow<'_, str>>>()
            .join(" ")
    );
    add_origen_env(&mut cmd);

    let res = cmd.status()?;
    Ok(if res.success() { 0 } else { 1 })
}

pub fn add_prephase_cmds<'a>(cmd: App) -> App {
    cmd.subcommand(config_exec_cmd(Command::new(BASE_CMD)).disable_help_flag(true))
}

pub fn config_exec_cmd<'a>(cmd: App) -> App {
    cmd.trailing_var_arg(true)
        .arg(
            Arg::new("cmd")
                .help("The command to be run")
                .action(SetArg)
                .required(true)
                .value_name("COMMAND"),
        )
        .arg(
            Arg::new("args")
                .help("Arguments to be passed to the command")
                .action(AppendArgs)
                .allow_hyphen_values(true)
                .num_args(1..)
                .required(false)
                .value_name("ARGS"),
        )
}

gen_core_cmd_funcs__no_exts__no_app_opts!(BASE_CMD, EXEC_HELP, { |cmd: App| config_exec_cmd(cmd) });
