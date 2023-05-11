use super::launch_as;
use crate::framework::{AuxCmds, build_path};
use crate::framework::aux_cmds::{add_aux_ns_subcmds, add_aux_ns_helps};
use indexmap::IndexMap;
use super::_prelude::*;
use crate::framework::helps::NOT_EXTENDABLE_MSG;

pub const BASE_CMD: &'static str = "auxillary_commands";

pub (crate) fn add_helps(helps: &mut CmdHelps, aux_cmds: &AuxCmds) {
    helps.add_core_cmd(BASE_CMD).set_help_msg("Interface with auxillary commands").set_as_not_extendable();
    add_aux_ns_helps(helps, aux_cmds);
}

pub (crate) fn add_commands<'a>(app: App<'a>, helps: &'a CmdHelps, aux_commands: &'a AuxCmds, exts: &'a Extensions) -> Result<App<'a>> {
    let mut aux_sub = helps.core_cmd(BASE_CMD).visible_alias("aux_cmds").arg_required_else_help(true);
    aux_sub = add_aux_ns_subcmds(&app, aux_sub, helps, aux_commands, exts)?;
    Ok(app.subcommand(aux_sub))
}

pub(crate) fn run(cmd: &clap::ArgMatches, mut app: &clap::App, exts: &crate::Extensions, plugins: Option<&crate::Plugins>, aux_cmds: &crate::AuxCmds) -> origen::Result<()> {
    if let Some(subc) = cmd.subcommand() {
        let ns_ins = aux_cmds.namespaces.get(subc.0).expect(&format!("Expected auxillary command namespace '{}' to be present, but was not found", subc.0));
        let path = build_path(&subc.1)?;

        let mut overrides = IndexMap::new();

        let mut matches = subc.1;
        let mut path_pieces: Vec<String> = vec!();
        app = app.find_subcommand(BASE_CMD).unwrap();
        app = app.find_subcommand(subc.0).unwrap();
        while matches.subcommand_name().is_some() {
            let n = matches.subcommand_name().unwrap();
            matches = matches.subcommand_matches(&n).unwrap();
            app = app.find_subcommand(n).unwrap();
            path_pieces.push(n.to_string());
        }

        launch_as("_dispatch_aux_cmd_", Some(&path_pieces), matches, app, exts.get_aux_ext(subc.0, &path), plugins, Some(
            {
                overrides.insert("dispatch_root".to_string(), Some(format!("r'{}'", ns_ins.root().display())));
                overrides.insert("dispatch_src".to_string(), Some(format!("r'{}'", subc.0)));
                overrides
            }
        ), None);
        Ok(())
    } else {
        // This case shouldn't happen as any non-valid command should be
        // caught previously by clap and a non-command invocation should
        // print the help message.
        unreachable!("Expected an AUX command but none was found!");
    }
}
