use clap::{ArgMatches, Command};
use indexmap::IndexMap;
use crate::framework::AppCmds;
use crate::commands::_prelude::*;
use crate::framework::app_cmds::add_commands as add_app_user_commands;
use crate::framework::app_cmds::add_helps as add_app_cmd_helps;

pub const BASE_CMD: &'static str = "app";

pub (crate) fn add_helps(helps: &mut CmdHelps, app_cmds: &AppCmds) {
    helps.add_core_cmd(BASE_CMD).set_help_msg("Manage and interface with the application");
    add_app_cmd_helps(helps, app_cmds);
}

pub (crate) fn add_commands<'a>(app: App<'a>, helps: &'a CmdHelps, app_cmds: &'a AppCmds, exts: &'a Extensions, ) -> Result<App<'a>> {
    let mut app_subc = helps.core_cmd(BASE_CMD)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Initialize the application's revision control")
        )
        .subcommand(
            Command::new("status")
                .about("Show any local changes")
                .arg(Arg::new("modified")
                    .long("modified")
                    .action(SetArgTrue)
                    .help("Show tracked, modified files")
                )
                .arg(Arg::new("untracked")
                    .long("untracked")
                    .action(SetArgTrue)
                    .help("Show untracked files")
                )
        )
        .subcommand(
            Command::new("checkin")
                .about("Check in the given pathspecs")
                .arg(Arg::new("pathspecs")
                    .help("The paths to be checked in")
                    .action(AppendArgs)
                    .value_name("PATHSPECS")
                    .multiple(true)
                )
                .arg(Arg::new("all")
                    .long("all")
                    .short('a')
                    .action(SetArgTrue)
                    .conflicts_with("pathspecs")
                    .help("Check in all changes in the workspace")
                )
                .arg(Arg::new("dry-run")
                    .long("dry-run")
                    .action(SetArgTrue)
                    .conflicts_with("pathspecs")
                    .help("Perform a dry-run only")
                )
                .arg(Arg::new("message")
                    .long("message")
                    .short('m')
                    .action(SetArg)
                    .required(true)
                    .help("Message to provide with the check-in operation")
                )
        )
        .subcommand(
            Command::new("package")
                .about("Build the app into publishable package (e.g., a 'python wheel')"),
        )
        .subcommand(Command::new("run_publish_checks")
            .about("Run production-ready and publish-ready checks")
        )
        .subcommand(Command::new("publish")
            .about("Publish (release) the app")
            .arg(Arg::new("dry-run")
                .long("dry-run")
                .action(SetArgTrue)
                .help("Runs through the entire process except the uploading and mailer steps")
            )
            .arg(Arg::new("version")
                .long("version")
                .action(SetArg)
                .value_name("VERSION")
                .help("Publish with the given version increment")
            )
            .arg(Arg::new("release-note")
                .long("release-note")
                .action(SetArg)
                .value_name("NOTE")
                .help("Publish with the given release note")
            )
            .arg(Arg::new("release-title")
                .long("release-title")
                .action(SetArg)
                .value_name("TITLE")
                .help("Publish with the given release title")
            )
            .arg(Arg::new("no-release-title")
                .long("no-release-title")
                .action(SetArgTrue)
                .help("Indicate no release title will be provided")
                .conflicts_with("release-title")
            )
        );
    app_subc = add_app_user_commands(app_subc, helps, app_cmds, exts)?;
    Ok(app.subcommand(app_subc))
}

fn _run(cmd: &str, proc_cmd: &ArgMatches, args: Option<IndexMap<&str, String>>) -> Result<()>{
    super::launch(
        cmd,
        if let Some(targets) = proc_cmd.get_many::<String>("target") {
            Some(targets.map(|t| t.as_str()).collect())
        } else {
            Option::None
        },
        &None,
        None,
        None,
        None,
        false,
        args,
    );
    Ok(())
}

pub(crate) fn run(cmd: &ArgMatches, mut app: &App, exts: &Extensions, plugins: Option<&Plugins>, app_cmds: &AppCmds) -> origen::Result<()> {
    let subcmd = cmd.subcommand().unwrap();
    let sub = subcmd.1;
    match subcmd.0 {
        "init" => {
            _run("app:init", sub, None)
        }
        "status" => {
            let mut args = IndexMap::new();
            if sub.contains_id("modified") {
                args.insert("modified", "True".to_string());
            }
            if sub.contains_id("untracked") {
                args.insert("untracked", "True".to_string());
            }
            _run("app:status", sub, Some(args))
        }
        "checkin" => {
            let mut args = IndexMap::new();
            if sub.contains_id("all") {
                args.insert("all", "True".to_string());
            }
            if sub.contains_id("dry-run") {
                args.insert("dry-run", "True".to_string());
            }
            if let Some(pathspecs) = sub.get_many::<String>("pathspecs") {
                let p = pathspecs
                    .map(|ps| format!("\"{}\"", ps))
                    .collect::<Vec<String>>();
                args.insert("pathspecs", format!("[{}]", p.join(",")));
            }
            args.insert(
                "msg",
                format!("\"{}\"", sub.get_one::<String>("message").unwrap()),
            );
            _run("app:checkin", sub, Some(args))
        }
        "package" => {
            _run("app:package", sub, None)
        }
        "publish" => {
            let mut args = IndexMap::new();
            if sub.contains_id("dry-run") {
                args.insert("dry-run", "True".to_string());
            }
            if let Some(v) = sub.get_one::<&str>("version") {
                args.insert("version", format!("\"{}\"", v.to_string()));
            }
            if let Some(r) = sub.get_one::<&str>("release-note") {
                args.insert("release-note", format!("\"{}\"", r.to_string()));
            }
            if let Some(t) = sub.get_one::<&str>("release-title") {
                args.insert("release-title", format!("\"{}\"", t.to_string()));
            }
            if sub.contains_id("no-release-title") {
                args.insert("no-release-title", "True".to_string());
            }
            _run("app:publish", sub, Some(args))
        }
        "run_publish_checks" => {
            _run("app:run_publish_checks", sub, None)
        }
        "commands" => {
            if let Some(subc) = cmd.subcommand() {
                let mut overrides = IndexMap::new();
        
                let mut matches = cmd;
                let mut path_pieces: Vec<String> = vec!();
                app = app.find_subcommand(BASE_CMD).unwrap();
                matches = matches.subcommand_matches("commands").unwrap();
                app = app.find_subcommand("commands").unwrap();
                while matches.subcommand_name().is_some() {
                    let n = matches.subcommand_name().unwrap();
                    matches = matches.subcommand_matches(&n).unwrap();
                    app = app.find_subcommand(n).unwrap();
                    path_pieces.push(n.to_string());
                }
                let path = path_pieces.join(".");
                launch_as("_dispatch_app_cmd_", Some(&path_pieces), matches, app, exts.get_app_ext(&path), plugins, Some(
                    {
                        overrides.insert("dispatch_root".to_string(), Some(format!("r'{}'", app_cmds.cmds_root()?.display())));
                        overrides
                    }
                ), None);
                Ok(())
            } else {
                // This case shouldn't happen as any non-valid command should be
                // caught previously by clap and a non-command invocation should
                // print the help message.
                unreachable!("Expected an APP command but none was found!");
            }
        }

        _ => unreachable!(),
    }
}
