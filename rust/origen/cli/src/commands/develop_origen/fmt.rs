use crate::python::PYTHON_CONFIG;
use crate::Result;
use origen::core::term::*;
use origen::STATUS;
use origen_metal::utils::file::cd;
use std::env;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::commands::_prelude::*;
pub const BASE_CMD: &'static str = "fmt";

pub (crate) fn fmt_cmd<'a>() -> SubCmd<'a> {
    core_subcmd__no_exts__no_app_opts!(
        BASE_CMD,
        "Nicely format all Rust and Python files",
        { |cmd| { cmd } }
    )
}

pub(crate) fn run() -> Result<()> {
    let orig_dir = env::current_dir().expect("Couldn't read the PWD");

    if STATUS.is_origen_present {
        starting("rust/origen ... ");
        cd(&STATUS.origen_wksp_root.join("rust").join("origen"))?;

        cargo_fmt();

        starting("rust/pyapi ... ");
        cd(&STATUS.origen_wksp_root.join("rust").join("pyapi"))?;

        cargo_fmt();

        starting("rust/origen_metal ... ");
        cd(&STATUS.origen_wksp_root.join("rust").join("origen_metal"))?;

        cargo_fmt();

        starting("rust/pyapi_metal ... ");
        cd(&STATUS.origen_wksp_root.join("rust").join("pyapi_metal"))?;

        cargo_fmt();
    }

    let root = match STATUS.is_origen_present {
        true => {
            starting("python_app ... ");
            STATUS.origen_wksp_root.join("test_apps").join("python_app")
        }
        false => {
            starting("formatting ... ");
            origen::app().unwrap().root.clone()
        }
    };

    cd(&root)?;

    py_fmt(&root);

    if STATUS.is_origen_present {
        starting("python/origen ... ");
        let dir = &STATUS.origen_wksp_root.join("python").join("origen");
        py_fmt(&dir);

        starting("python/origen_metal ... ");
        let dir = &STATUS.origen_wksp_root.join("python").join("origen_metal");
        py_fmt(&dir);
    }

    let _ = env::set_current_dir(&orig_dir);
    Ok(())
}

fn starting(job: &str) {
    print!("{}", job);
    let _ = stdout().flush();
}

// Returns true if no problems
fn py_fmt(dir: &Path) {
    let res = PYTHON_CONFIG
        .poetry_command()
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
