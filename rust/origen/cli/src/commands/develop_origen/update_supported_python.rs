use std::path::Path;
use std::fs::File;
use std::io::Write;
use crate::STATUS;
use crate::commands::_prelude::*;
pub const BASE_CMD: &'static str = "update_supported_python";

pub (crate) fn update_supported_python_cmd<'a>() -> SubCmd<'a> {
    core_subcmd__no_exts__no_app_opts!(
        BASE_CMD,
        "Update supported python versions in all pyproject files",
        { |cmd| { 
            cmd.arg(Arg::new("min_version")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::new("non_inclusive_max_version")
                .takes_value(true)
                .required(true)
            )
        } }
    )
}

pub(crate) fn run(invocation: &clap::ArgMatches) -> Result<()> {
    let min = invocation.get_one::<String>("min_version").unwrap();
    let max = invocation.get_one::<String>("non_inclusive_max_version").unwrap();
    let version_str = format!("python = \">={min},<{max}\"");
    let mv_min = min.split('.').collect::<Vec<&str>>()[1].parse::<u8>()?;
    let max_parts = max.split('.').collect::<Vec<&str>>();
    let mv_max;
    if max_parts.len() == 2 {
        mv_max = max_parts[1].parse::<u8>()? - 1;
    } else {
        mv_max = max_parts[1].parse::<u8>()?;
    }

    // Update pyprojects
    // origen/metal pyproject
    let p = &STATUS.origen_wksp_root.join("python");
    update_pyproject(&p.join("origen_metal"), &version_str)?;
    update_pyproject(&p.join("origen"),&version_str)?;

    // Various app pyprojects, outside of no-workspace
    let p = &STATUS.origen_wksp_root.join("test_apps");
    update_pyproject(&p.join("pl_ext_cmds"),&version_str)?;
    update_pyproject(&p.join("python_app"),&version_str)?;
    update_pyproject(&p.join("python_no_app"),&version_str)?;
    update_pyproject(&p.join("python_plugin"),&version_str)?;
    update_pyproject(&p.join("python_plugin_no_cmds"),&version_str)?;
    update_pyproject(&p.join("python_plugin_the_second"),&version_str)?;
    update_pyproject(&p.join("test_apps_shared_test_helpers"),&version_str)?;

    // No-workspace pyprojects
    let p = &STATUS.origen_wksp_root.join("test_apps/no_workspace");
    update_pyproject(&p.join("user_install"),&version_str)?;
    update_pyproject(&p.join("templates"),&version_str)?;

    // Write rust CLI python settings
    let p = &STATUS.origen_wksp_root.join("rust/origen/cli/src/_generated/python.rs");
    println!("Creating python rs file {}", p.display());
    let mut f = File::create(p)?;
    f.write_fmt(format_args!("// THIS IS AN AUTO GENERATED FILE FROM CMD {}\n", BASE_CMD))?;
    f.write_fmt(format_args!("pub const MIN_PYTHON_VERSION: &str = \"{}\";\n", min))?;
    f.write(b"pub const PYTHONS: &[&str] = &[\n")?;
    f.write(b"    \"python\",\n")?;
    f.write(b"    \"python3\",\n")?;
    for v in mv_min..(mv_max+1) {
        f.write_fmt(format_args!("    \"python3.{}\",\n", v))?;
    }
    f.write(b"];\n")?;

    Ok(())
}

fn update_pyproject(path: &Path, version_str: &str) -> Result<()> {
    let n = path.join("pyproject.toml");
    println!("Updating pyproject: {}", n.display());

    let content = std::fs::read_to_string(&n)?;
    let mut lines: Vec<&str> = content.split("\n").collect();
    let i = lines.iter().position( |l| l.starts_with("python = ")).unwrap();
    lines[i] = version_str;

    // Write out file
    let mut f = File::create(n)?;
    f.write_all(lines.join("\n").as_bytes())?;

    Ok(())
}