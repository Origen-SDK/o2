pub use crate::framework::aux_cmds::CMD_NAME;
use super::launch_as;
use crate::framework::build_path;
use indexmap::IndexMap;

pub(crate) fn run(cmd: &clap::ArgMatches, mut app: &clap::App, exts: &crate::Extensions, plugins: Option<&crate::Plugins>, aux_cmds: &crate::AuxCmds) -> origen::Result<()> {
    if let Some(subc) = cmd.subcommand() {
        let ns_ins = aux_cmds.namespaces.get(subc.0).expect(&format!("Expected auxillary command namespace '{}' to be present, but was not found", subc.0));
        let path = build_path(&subc.1)?;

        let mut overrides = IndexMap::new();

        let mut matches = subc.1;
        let mut path_pieces: Vec<String> = vec!();
        app = app.find_subcommand(CMD_NAME).unwrap();
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
