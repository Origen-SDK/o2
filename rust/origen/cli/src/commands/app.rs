use clap::ArgMatches;
use indexmap::IndexMap;

fn _run(cmd: &str, proc_cmd: &ArgMatches, args: Option<IndexMap<&str, String>>) {
    super::launch(
        cmd,
        if let Some(targets) = proc_cmd.values_of("target") {
            Some(targets.collect())
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
}

pub fn run(cmd: &ArgMatches) {
    let subcmd = cmd.subcommand();
    let sub = subcmd.1.unwrap();
    match subcmd.0 {
        "package" => {
            _run("app:package", sub, None);
        }
        "publish" => {
            let mut args = IndexMap::new();
            if sub.is_present("dry-run") {
                args.insert("dry-run", "True".to_string());
            }
            _run("app:publish", sub, Some(args));
        }
        "run_publish_checks" => {
            _run("app:run_publish_checks", sub, None);
        }

        _ => unreachable!(),
    }
}
