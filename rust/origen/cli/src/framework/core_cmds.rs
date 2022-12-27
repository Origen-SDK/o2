use super::{CmdHelps, Extensions};
use origen::Result;
use clap::Command;

pub fn add_core_subc_helps(helps: &mut CmdHelps, base_name: &str, cmd: &str, cmd_help: &str, subcmds: &[SubCmd]) {
    let n = format!("{}.{}", base_name, cmd);
    helps.add_core_cmd(&n).set_help_msg(cmd_help);
    for subc in subcmds {
        add_core_subc_helps(helps, &n, subc.name, subc.help, subc.subcmds);
    }
}

pub fn add_core_subcs<'a>(helps: &'a CmdHelps, exts: &'a Extensions, cmd: Command<'a>, base: Vec<&str>, subcmd: &SubCmd) -> Result<Command<'a>> {
    let mut n = base.clone();
    n.push((subcmd.name));
    let mut subc = helps.core_subc(&n);
    for s in subcmd.subcmds {
        subc = add_core_subcs(helps, exts, subc, n.clone(), s)?;
    }
    if let Some(setup) = subcmd.proc {
        subc = setup(subc);
    }
    // add exts
    subc = exts.apply_to_core_cmd(&n.join("."), subc);
    Ok(cmd.subcommand(subc))
}

pub struct SubCmd<'a> {
    pub name: &'static str,
    pub help: &'static str,
    pub subcmds: &'a [SubCmd<'a>],
    pub proc: Option<&'a dyn Fn(Command) -> Command>,
}

#[macro_export]
macro_rules! core_subcmd {
    ($name:expr, $help:expr, $proc:tt) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[],
            proc: Some(&$proc),
        }
    }};

    ($name:expr, $help:expr, $proc:tt, $($subcmd:expr ), *) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[$($subcmd),*],
            proc: Some(&$proc),
        }
    }};

    ($name:expr, $help:expr) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[],
            proc: None,
        }
    }};

    ($name:expr, $help:expr, $($subcmd:expr ), *) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[$($subcmd),*],
            proc: None,
        }
    }};
}

#[macro_export]
macro_rules! gen_core_cmd_funcs {
    ($base_name:expr, $cmd_help:expr, $proc:tt) => {
        gen_core_cmd_funcs!($base_name, $cmd_help, $proc,);
    };
    ($base_name:expr, $cmd_help:expr, $proc:tt, $($subcmd:expr ), *) => {
    // ($base_name:expr, $cmd_help:expr, $subcmds:expr) => {
            // let mut cmd = helps.core_cmd(CMD_NAME);
        pub (crate) fn add_helps(helps: &mut $crate::CmdHelps) {
            helps.add_core_cmd($base_name).set_help_msg($cmd_help);
            // for subc in $subcmds {
            //     add_core_cmd_int!(helps, $base_name, $subcmd.0, $subcmd.1, $subcmd.2);
            // }
            $(
                // add_core_cmd_int!(helps, $base_name, $subcmd.0, $subcmd.1, $subcmd.2);
                $crate::framework::core_cmds::add_core_subc_helps(helps, $base_name, $subcmd.name, $subcmd.help, $subcmd.subcmds);
            )*
        }

        pub (crate) fn add_commands<'a>(app: clap::Command<'a>, helps: &'a $crate::CmdHelps, exts: &'a $crate::Extensions) -> origen::Result<clap::Command<'a>> {
            let mut cmd = helps.core_cmd($base_name);
            cmd = $proc(cmd);
            $(
                // add_core_cmd_int!(helps, $base_name, $subcmd.0, $subcmd.1, $subcmd.2);
                cmd = $crate::framework::core_cmds::add_core_subcs(helps, exts, cmd, vec!($base_name), &$subcmd)?;
            )*
            cmd = exts.apply_to_core_cmd($base_name, cmd);
            Ok(app.subcommand(cmd))
        }
    };
}