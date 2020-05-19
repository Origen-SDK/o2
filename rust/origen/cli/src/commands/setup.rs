extern crate time;

use crate::python::{poetry_version, MIN_PYTHON_VERSION, PYTHON_CONFIG};
use online::online;
use origen::core::term::*;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const POETRY_INSTALLER: &str =
    "https://raw.githubusercontent.com/sdispater/poetry/1.0.0b7/get-poetry.py";

pub fn run() {
    print!("Is a suitable Python available? ... ");
    if PYTHON_CONFIG.available {
        greenln("YES");
    } else {
        redln("NO");
        println!("");
        println!(
            "Could not find Python > {} available, please install this and try again.",
            MIN_PYTHON_VERSION
        );
        std::process::exit(1);
    }

    print!("Is the internet accessible?     ... ");
    if online(None).is_ok() {
        greenln("YES");
    } else {
        redln("NO");
        println!("");
        println!("In future, Origen would now talk you through what files to get and where to put them, but that's not implemented yet, sorry!");
        std::process::exit(1);
    }

    let mut attempts = 0;
    while attempts < 2 {
        print!("Is a suitable Poetry available? ... ");
        let version = poetry_version();
        if version.is_some() && version.unwrap().major == 1 {
            greenln("YES");
            attempts = 2;
        } else {
            redln("NO");
            println!("");
            attempts = attempts + 1;

            let tmp_dir =
                PathBuf::from("/tmp").join(format!("origen-{}", time::now().to_timespec().nsec));
            fs::create_dir_all(format!("{}", tmp_dir.display())).expect("Couldn't create tmp dir");

            // Get the Poetry installer script
            let get_poetry_file = tmp_dir.join("get-poetry.py");
            let mut resp =
                reqwest::get(POETRY_INSTALLER).expect("Failed to fetch Poetry install file");
            let mut out = fs::File::create(format!("{}", get_poetry_file.display()))
                .expect("Failed to create Poetry install file");
            io::copy(&mut resp, &mut out).expect("Failed to copy content to Poetry install file");

            // Modify the script to handle the case where the python command is not 'python'
            let data =
                fs::read_to_string(&get_poetry_file).expect("Unable to read Poetry install file");
            let new_data = data.replace(
                "/bin/env python",
                &format!("/bin/env {}", PYTHON_CONFIG.command),
            );
            fs::write(&get_poetry_file, new_data).expect("Unable to write Poetry install file");

            // Install Poetry
            Command::new(&PYTHON_CONFIG.command)
                .arg(get_poetry_file)
                .arg("--yes")
                .status()
                .expect("Something went wrong install Poetry");

            if poetry_version().unwrap().major != 1 {
                // Have to use --preview here to get a 1.0.0 pre version, can only use versions for
                // official releases
                Command::new(&PYTHON_CONFIG.poetry_command)
                    .arg("self:update")
                    .arg("--preview")
                    .status()
                    .expect("Something wend wrong updating Poetry");
            }
            println!("");
        }
    }

    print!("Are the app's deps. installed?  ... ");

    let status = Command::new(&PYTHON_CONFIG.poetry_command)
        .arg("install")
        .arg("--no-root")
        .status();

    if status.is_ok() {
        greenln("YES");
        std::process::exit(0);
    } else {
        redln("NO");
        std::process::exit(1);
    }
}
