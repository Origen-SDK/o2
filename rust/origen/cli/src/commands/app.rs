use super::fmt::cd;
use clap::ArgMatches;
use std::process::Command;
use indexmap::IndexMap;

pub fn run(cmd: &ArgMatches) {
    match cmd.subcommand_name() {
        Some("package") => {
            let wheel_dir = origen::app().unwrap().root.join("dist");
            // Make sure we are not about to upload any stale/old artifacts
            if wheel_dir.exists() {
                std::fs::remove_dir_all(&wheel_dir).expect("Couldn't delete existing dist dir");
            }
            cd(&origen::app().unwrap().root);

            Command::new("poetry")
                .args(&["build", "--no-interaction", "--format", "wheel"])
                .status()
                .expect("failed to build the application package for release");

            //if matches.is_present("publish") {
            //    let pypi_token =
            //        std::env::var("ORIGEN_PYPI_TOKEN").expect("ORIGEN_PYPI_TOKEN is not defined");

            //    let args: Vec<&str> = vec![
            //        "upload",
            //        //"-r",
            //        //"testpypi",
            //        "--username",
            //        "__token__",
            //        "--password",
            //        &pypi_token,
            //        "--non-interactive",
            //        "dist/*",
            //    ];

            //    Command::new("twine")
            //        .args(&args)
            //        .status()
            //        .expect("failed to publish origen");
            //}
        }
        Some("publish") => {
            // Will handle all the following in one go:
            // * Checking for changes in the current environment
            // * Checking in those changes
            // * Building the wheel
            // * Uploading ^
            // * Building the website
            // * Publishing ^
            // * Sending release emails
            let mut args = IndexMap::new();
            super::launch(
                "app:publish",
                if let Some(targets) = cmd.values_of("target") {
                    Some(targets.collect())
                } else {
                    Option::None
                },
                &None,
                None,
                None,
                None,
                false,
                Some(args),
            );
       }

        None | _ => unreachable!(),
    }
}
