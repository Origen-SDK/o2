use origen::core::application::target;
use super::_prelude::*;

pub const BASE_CMD: &'static str = "target";
pub const FULL_PATHS_OPT: &'static str = "full-paths";
pub const TARGETS_OPT: &'static str = "targets";

macro_rules! full_paths_opt {
    () => {
        Arg::new(FULL_PATHS_OPT)
            .long(FULL_PATHS_OPT)
            .visible_alias("full_paths")
            .short('f')
            .help("Display targets' full paths")
            .action(SetArgTrue)
    }
}

macro_rules! targets_arg {
    ($help:expr) => {
        Arg::new(TARGETS_OPT)
            .help($help)
            .action(AppendArgs)
            .value_name("TARGETS")
            .multiple(true)
            .required(true)
    }
}

macro_rules! tnames {
    ($cmd:expr) => {
        $cmd.get_many::<String>(TARGETS_OPT).unwrap().map(|t| t.as_str()).collect::<Vec<&str>>()
    }
}

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Set/view the default target",
    { |cmd: App<'a>| { cmd.arg(full_paths_opt!()).visible_alias("t") }},
    core_subcmd__no_exts__no_app_opts!("add", "Activates the given target(s)", { |cmd: App| {
        cmd.visible_alias("a")
        .arg(targets_arg!("Targets to be activated"))
        .arg(full_paths_opt!())
    }}),
    core_subcmd__no_exts__no_app_opts!("clear", "Deactivates any and all current targets", { |cmd: App| {
        cmd.visible_alias("c")
    }}),
    core_subcmd__no_exts__no_app_opts!("remove", "Deactivates the given target(s)", { |cmd: App| {
        cmd.visible_alias("r")
        .arg(targets_arg!("Targets to be deactivated"))
        .arg(full_paths_opt!())
    }}),
    core_subcmd__no_exts__no_app_opts!("set", "Activates the given target(s) while deactivating all others", { |cmd: App| {
        cmd.visible_alias("s")
        .arg(targets_arg!("Targets to be set"))
        .arg(full_paths_opt!())
    }}),
    core_subcmd__no_exts__no_app_opts!("default", "Activates the default target(s) while deactivating all others", { |cmd: App| {
        cmd.visible_alias("d")
        .arg(full_paths_opt!())
    }}),
    core_subcmd__no_exts__no_app_opts!("view", "Views the currently activated target(s)", { |cmd: App| {
        cmd.visible_alias("v")
        .arg(full_paths_opt!())
    }})
);

macro_rules! view {
    ($invocation:expr) => {
        view_targets(*$invocation.get_one::<bool>(FULL_PATHS_OPT).unwrap())
    }
}

fn view_targets(fp: bool) -> Result<()> {
    if let Some(targets) = target::get(fp) {
        println!("The targets currently enabled are:");
        println!("{}", targets.join("\n"))
    } else {
        println!("No targets have been enabled and this workspace does not enable any default targets")
    }
    Ok(())
}

pub(crate) fn run(invocation: &clap::ArgMatches) -> origen::Result<()> {
    if let Some((n, subcmd)) = invocation.subcommand() {
        match n {
            "add" => {
                target::add(tnames!(subcmd));
            }
            "clear" => {
                target::clear();
                view_targets(false)?;
                return Ok(())
            }
            "default" => {
                target::reset();
            }
            "remove" => {
                target::remove(tnames!(subcmd));
            }
            "set" => {
                target::set(tnames!(subcmd));
            }
            "view" => {
                view!(subcmd)?;
                return Ok(());
            }
            _ => unreachable_invalid_subc!(n)
        }
        // Show the effect after running the command
        view!(subcmd)
    } else {
        view!(invocation)?;
        println!();
        print_subcmds_available_msg!();
        Ok(())
    }
}
