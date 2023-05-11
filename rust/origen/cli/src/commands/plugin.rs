use super::_prelude::*;
use crate::framework::plugins::{add_pl_ns_helps, add_pl_ns_subcmds};

pub const BASE_CMD: &'static str = "plugin";

pub (crate) fn add_helps(helps: &mut CmdHelps, plugins: Option<&Plugins>) {
    helps.add_core_cmd(BASE_CMD).set_help_msg("Access added commands from individual plugins").set_as_not_extendable();
    add_pl_ns_helps(helps, plugins);
}

pub (crate) fn add_commands<'a>(mut app: App<'a>, helps: &'a CmdHelps, plugins: Option<&'a Plugins>, exts: &'a Extensions) -> Result<App<'a>> {
    let mut pl_sub = Command::new(BASE_CMD).visible_alias("pl").arg_required_else_help(true);
    pl_sub = helps.apply_core_cmd_helps(BASE_CMD, pl_sub);
    if let Some(pls) = plugins {
        pl_sub = add_pl_ns_subcmds(pl_sub, helps, pls, exts)?;
    }
    Ok(app.subcommand(pl_sub))
}

pub fn run(cmd: RunInput, app: &clap::App, exts: &crate::Extensions, plugins: Option<&Plugins>) -> Result<()> {
    if let Some(subcmd) = cmd.subcommand() {
        let sub = subcmd.1;
        plugins.unwrap().plugins.get(subcmd.0).unwrap().dispatch(sub, app, exts, plugins)
    } else {
        Ok(())
    }
}