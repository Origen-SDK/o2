use super::{CmdHelps, Extensions};
use origen::Result;
use clap::Command;

pub fn add_core_subc_helps(helps: &mut CmdHelps, base_name: &str, cmd: &str, cmd_help: &str, extendable: bool, subcmds: &[SubCmd]) {
    let n = format!("{}.{}", base_name, cmd);
    let h = helps.add_core_cmd(&n).set_help_msg(cmd_help);
    if !extendable {
        h.set_as_not_extendable();
    }
    for subc in subcmds {
        add_core_subc_helps(helps, &n, subc.name, subc.help, subc.extendable, subc.subcmds);
    }
}

pub fn add_core_subcs<'a>(helps: &'a CmdHelps, exts: Option<&'a Extensions>, cmd: Command<'a>, base: Vec<&str>, subcmd: &SubCmd) -> Result<Command<'a>> {
    let mut n = base.clone();
    n.push(subcmd.name);
    let mut subc = helps.core_subc(&n);
    for s in subcmd.subcmds {
        subc = add_core_subcs(helps, exts, subc, n.clone(), s)?;
    }
    if let Some(setup) = subcmd.proc {
        subc = setup(subc);
    }
    if subcmd.include_app_opts {
        subc = super::add_all_app_opts(subc);
    }
    // add exts
    if let Some(e) = exts {
        subc = e.apply_to_core_cmd(&n.join("."), subc);
    }
    Ok(cmd.subcommand(subc))
}

pub struct SubCmd<'a> {
    pub name: &'static str,
    pub help: &'static str,
    pub subcmds: &'a [SubCmd<'a>],
    pub proc: Option<&'a dyn Fn(Command) -> Command>,
    pub include_app_opts: bool,
    pub extendable: bool,
}
#[macro_export]
macro_rules! core_subcmd {
    ($($args:tt)+) => {{
        $crate::_core_subcmd!(true, true, $($args)*)
    }}
}

#[macro_export]
macro_rules! _core_subcmd {
    ($include_app_opts:expr, $extendable:expr, $name:expr, $help:expr, $proc:tt) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[],
            proc: Some(&$proc),
            include_app_opts: $include_app_opts,
            extendable: $extendable,
        }
    }};

    ($name:expr, $help:expr, $proc:tt, $($subcmd:expr ), *) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[$($subcmd),*],
            proc: Some(&$proc),
            include_app_opts: $include_app_opts,
            extendable: true,
        }
    }};

    ($name:expr, $help:expr) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[],
            proc: None,
            include_app_opts: $include_app_opts,
            extendable: true,
        }
    }};

    ($name:expr, $help:expr, $($subcmd:expr ), *) => {{
        $crate::framework::core_cmds::SubCmd {
            name: $name,
            help: $help,
            subcmds: &[$($subcmd),*],
            proc: None,
            include_app_opts: $include_app_opts,
            extendable: true,
        }
    }};
}

#[macro_export]
macro_rules! core_subcmd__no_exts__no_app_opts {
    ($($args:tt)+) => {{
        $crate::_core_subcmd!(false, false, $($args)*)
    }}
}

#[macro_export]
macro_rules! gen_core_cmd_funcs {
    ($base_name:expr, $cmd_help:expr, $proc:tt) => {
        gen_core_cmd_funcs!($base_name, $cmd_help, $proc,);
    };
    ($base_name:expr, $cmd_help:expr, $proc:tt, $($subcmd:expr ), *) => {
        pub (crate) fn add_helps(helps: &mut $crate::CmdHelps) {
            helps.add_core_cmd($base_name).set_help_msg($cmd_help);
            $(
                $crate::framework::core_cmds::add_core_subc_helps(helps, $base_name, $subcmd.name, $subcmd.help, $subcmd.extendable, $subcmd.subcmds);
            )*
        }

        pub (crate) fn add_commands<'a>(app: clap::Command<'a>, helps: &'a $crate::CmdHelps, exts: &'a $crate::Extensions) -> origen::Result<clap::Command<'a>> {
            let mut cmd = helps.core_cmd($base_name);
            cmd = $proc(cmd);
            $(
                cmd = $crate::framework::core_cmds::add_core_subcs(helps, Some(exts), cmd, vec!($base_name), &$subcmd)?;
            )*
            cmd = crate::framework::add_all_app_opts(cmd);
            cmd = exts.apply_to_core_cmd($base_name, cmd);
            Ok(app.subcommand(cmd))
        }
    };
}

#[macro_export]
macro_rules! gen_core_cmd_funcs__no_exts__no_app_opts {
    ($base_name:expr, $cmd_help:expr, $proc:tt, $($subcmd:expr ), *) => {
        pub (crate) fn add_helps(helps: &mut $crate::CmdHelps) {
            helps.add_core_cmd($base_name).set_help_msg($cmd_help).set_as_not_extendable();
            $(
                $crate::framework::core_cmds::add_core_subc_helps(helps, $base_name, $subcmd.name, $subcmd.help, $subcmd.extendable, $subcmd.subcmds);
            )*
        }

        pub (crate) fn add_commands<'a>(app: clap::Command<'a>, helps: &'a $crate::CmdHelps, exts: &'a $crate::Extensions) -> origen::Result<clap::Command<'a>> {
            let mut cmd = helps.core_cmd($base_name);
            cmd = $proc(cmd);
            $(
                cmd = $crate::framework::core_cmds::add_core_subcs(helps, None, cmd, vec!($base_name), &$subcmd)?;
            )*
            Ok(app.subcommand(cmd))
        }
    };

    ($base_name:expr, $cmd_help:expr, $proc:tt) => {
        pub (crate) fn add_helps(helps: &mut $crate::CmdHelps) {
            helps.add_core_cmd($base_name).set_help_msg($cmd_help).set_as_not_extendable();
        }

        pub (crate) fn add_commands<'a>(app: clap::Command<'a>, helps: &'a $crate::CmdHelps, exts: &'a $crate::Extensions) -> origen::Result<clap::Command<'a>> {
            let mut cmd = helps.core_cmd($base_name);
            cmd = $proc(cmd);
            Ok(app.subcommand(cmd))
        }
    };
}