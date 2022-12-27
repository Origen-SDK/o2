// FOR_PR clean up and switch to macros to generate
use crate::commands::_prelude::*;
use origen::STATUS;
use std::fs;

pub const BASE_CMD: &'static str = "interactive";
pub const CMD_NAME: &'static str = "interactive";

pub (crate) fn add_commands<'a>(app: App<'a>, origen_commands: &mut Vec<CommandHelp>) -> Result<App<'a>> {
    let i_help = "Start an Origen console to interact with the DUT";
    origen_commands.push(CommandHelp {
        name: "interactive".to_string(),
        help: i_help.to_string(),
        shortcut: Some("i".to_string()),
    });
    let mut subc = Command::new("interactive").about(i_help).visible_alias("i");
    if STATUS.is_app_present {
        subc = subc.arg(
            Arg::new("target")
            .short('t')
            .long("target")
            .help("Override the default target currently set by the workspace")
            .action(AppendArgs)
            .use_delimiter(true)
            .multiple(true)
            .number_of_values(1)
            .value_name("TARGET"),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .help("Override the default execution mode currently set by the workspace")
                .action(SetArg)
                .value_name("MODE"),
        );
    }
    Ok(app.subcommand(subc))
}

crate::gen_simple_run_func!();
