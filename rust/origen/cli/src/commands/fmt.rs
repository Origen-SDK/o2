use crate::python::PYTHON_CONFIG;
use origen::core::term::*;
use origen::STATUS;
use std::env;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn run() {
    let orig_dir = env::current_dir().expect("Couldn't read the PWD");

    if STATUS.is_origen_present {
        starting("rust/origen ... ");
        cd(&STATUS.origen_wksp_root.join("rust").join("origen"));

        cargo_fmt();

        starting("rust/pyapi ... ");
        cd(&STATUS.origen_wksp_root.join("rust").join("pyapi"));

        cargo_fmt();
    }

    let root = match STATUS.is_origen_present {
        true => {
            starting("example ... ");
            STATUS.origen_wksp_root.join("example")
        }
        false => {
            starting("formatting ... ");
            origen::app().unwrap().root.clone()
        }
    };

    cd(&root);

    py_fmt(&root);

    if STATUS.is_origen_present {
        starting("python ... ");
        let dir = &STATUS.origen_wksp_root.join("python");
        py_fmt(&dir);
    }

    let _ = env::set_current_dir(&orig_dir);
}

fn starting(job: &str) {
    print!("{}", job);
    let _ = stdout().flush();
}

// Returns true if no problems
fn py_fmt(dir: &Path) {
    let res = Command::new(&PYTHON_CONFIG.poetry_command)
        .arg("run")
        .arg("yapf")
        .arg("--in-place")
        .arg("--recursive")
        .arg(&format!("{}", dir.display()))
        .status();

    if let Ok(stat) = res {
        if stat.success() {
            greenln("YES");
            return;
        }
    }
    redln("NO");
}

// Returns true if no problems
fn cargo_fmt() {
    let res = Command::new("cargo").arg("fmt").status();
    if let Ok(stat) = res {
        if stat.success() {
            greenln("YES");
            return;
        }
    }
    redln("NO");
}

pub fn cd(dir: &Path) {
    env::set_current_dir(&dir).expect(&format!("Couldn't cd to '{}'", dir.display()));
}
