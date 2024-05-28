use super::_prelude::*;

pub const BASE_CMD: &'static str = "plugins";

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Interface with the Origen plugin manager",
    { |cmd: App<'a>| {
        cmd.visible_alias("pl_mgr").visible_alias("pls").arg_required_else_help(true)
    }},
    core_subcmd__no_exts__no_app_opts!("list", "List the available plugins", { |cmd: App| {
        cmd.visible_alias("ls")
    }})
);

pub fn run(cmd: RunInput, plugins: Option<&Plugins>) -> Result<()> {
    if let Some(subcmd) = cmd.subcommand() {
        match subcmd.0 {
            "list" => {
                if let Some(pls) = plugins {
                    if pls.is_empty() {
                        displayln!("There are no available plugins!");
                    } else {
                        displayln!("Available plugins:\n");
                        for (name, _) in pls.plugins.iter() {
                            displayln!("{}", name);
                        }
                    }
                } else {
                    displayln!("The plugin manager is not available or there was an error populating plugins!");
                }
            },
            _ => unreachable_invalid_subc!(subcmd.0)
        }
    }
    Ok(())
}
