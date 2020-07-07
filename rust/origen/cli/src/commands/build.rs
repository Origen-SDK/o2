use super::fmt::cd;
use clap::ArgMatches;
use origen::core::file_handler::File;
use origen::{Result, STATUS};
use regex::Regex;
use semver::Version;
use std::path::Path;
use std::process::Command;

pub fn run(matches: &ArgMatches) {
    if let Some(version) = matches.value_of("version") {
        if Version::parse(&version).is_err() {
            display_redln!(
                "Invalid version: '{}', must be a semantic version like 1.2.3 or 1.2.3-pre4",
                &version
            );
            std::process::exit(1);
        }
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("origen")
                .join("cli")
                .join("Cargo.toml"),
            version,
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("origen")
                .join("Cargo.toml"),
            version,
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("pyapi")
                .join("Cargo.toml"),
            version,
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("python")
                .join("pyproject.toml"),
            version,
        )
        .expect("Couldn't write version");
    }
    if matches.is_present("cli") {
        cd(&STATUS
            .origen_wksp_root
            .join("rust")
            .join("origen")
            .join("cli"));
        display!("");
        Command::new("cargo")
            .args(&["build"])
            .status()
            .expect("failed to execute process");
        display!("");
    } else {
        // A release build will do the following:
        //   1) Build a release version of the pyapi package and publish it to PyPI
        //   2) Build and publish the origen Python package
        if matches.is_present("release") {
            let wheel_dir = &STATUS
                .origen_wksp_root
                .join("rust")
                .join("pyapi")
                .join("target")
                .join("wheels");
            // Make sure we are not about to upload any stale/old artifacts
            if wheel_dir.exists() {
                std::fs::remove_dir_all(&wheel_dir).expect("Couldn't delete existing wheel dir");
            }
            cd(&STATUS.origen_wksp_root.join("rust").join("pyapi"));
            Command::new("maturin")
                .args(&[
                    "build",
                    "--no-sdist", // Local building of the pyapi will not be supported
                    "--release",
                    "--manifest-path",
                ])
                .status()
                .expect("failed to build pyapi for release");

            if matches.is_present("publish") {
                let pypi_token =
                    std::env::var("ORIGEN_PYPI_TOKEN").expect("ORIGEN_PYPI_TOKEN is not defined");

                let args: Vec<&str> = vec![
                    "upload",
                    //"-r",
                    //"testpypi",
                    "--username",
                    "__token__",
                    "--password",
                    &pypi_token,
                    "--non-interactive",
                    "target/wheels/*",
                ];

                Command::new("twine")
                    .args(&args)
                    .status()
                    .expect("failed to publish pyapi");
            }

            let wheel_dir = &STATUS.origen_wksp_root.join("python").join("dist");
            // Make sure we are not about to upload any stale/old artifacts
            if wheel_dir.exists() {
                std::fs::remove_dir_all(&wheel_dir).expect("Couldn't delete existing wheel dir");
            }
            cd(&STATUS.origen_wksp_root.join("python"));

            dependency_on_pyapi(true).expect("Couldn't enable dependency on origen_pyapi");

            Command::new("poetry")
                .args(&["build", "--no-interaction"])
                .status()
                .expect("failed to build origen for release");

            dependency_on_pyapi(false).expect("Couldn't disable dependency on origen_pyapi");

            if matches.is_present("publish") {
                let pypi_token =
                    std::env::var("ORIGEN_PYPI_TOKEN").expect("ORIGEN_PYPI_TOKEN is not defined");

                let args: Vec<&str> = vec![
                    "upload",
                    //"-r",
                    //"testpypi",
                    "--username",
                    "__token__",
                    "--password",
                    &pypi_token,
                    "--non-interactive",
                    "dist/*",
                ];

                Command::new("twine")
                    .args(&args)
                    .status()
                    .expect("failed to publish origen");
            }
        } else {
            // The default build will compile the latest PyAPI and copy it into
            // the example app's Python env
            cd(&STATUS.origen_wksp_root.join("test_apps").join("python_app"));
            display!("");
            Command::new("poetry")
                .args(&[
                    "run",
                    "maturin",
                    "develop",
                    "--manifest-path",
                    &format!(
                        "{}",
                        STATUS
                            .origen_wksp_root
                            .join("rust")
                            .join("pyapi")
                            .join("Cargo.toml")
                            .display()
                    ),
                ])
                .status()
                .expect("failed to execute process");
            display!("");
        }
    }
}

// Enables/disables the dependency on origen_pyapi for the main origen Python package
fn dependency_on_pyapi(enable: bool) -> Result<()> {
    let filepath = &STATUS
        .origen_wksp_root
        .join("python")
        .join("pyproject.toml");
    let contents = std::fs::read_to_string(&filepath)?;

    let new_content = match enable {
        true => contents.replace("#origen_pyapi", "origen_pyapi"),
        false => contents.replace("origen_pyapi", "#origen_pyapi"),
    };

    File::create(filepath.to_path_buf()).write(&new_content);
    Ok(())
}

fn write_version(filepath: &Path, version: &str) -> Result<()> {
    let content = std::fs::read_to_string(filepath)?;
    let mut lines: Vec<&str> = content.split("\n").collect();
    if let Some(last) = lines.last() {
        if last.to_string() == "" {
            lines.pop();
        }
    }
    let mut file = File::create(filepath.to_path_buf());
    let mut version_section_open = false;
    let mut version_written = false;

    lazy_static! {
        static ref VERSION_SECTION_HEADER: Regex =
            Regex::new(r"^\s*\[\s*(package|tool.poetry)\s*\]\s*$").unwrap();
        static ref SECTION_HEADER: Regex = Regex::new(r"^\s*\[\s*.*\s*\]\s*$").unwrap();
        static ref VERSION_LINE: Regex = Regex::new(r#"^\s*version\s*=\s*".*"\s*$"#).unwrap();
        static ref PYAPI_VERSION_LINE: Regex =
            Regex::new(r#"^\s*#?\s*origen_pyapi\s*=\s*".*"\s*$"#).unwrap();
    }

    for line in lines {
        if !version_written {
            if version_section_open && VERSION_LINE.is_match(&line) {
                file.write_ln(&format!("version = \"{}\"", version));
                version_written = true;
                continue;
            // If got to the end of the version section without writing the version
            } else if version_section_open && SECTION_HEADER.is_match(&line) {
                file.write_ln(&format!("version = \"{}\"", version));
                version_written = true;
            } else {
                if VERSION_SECTION_HEADER.is_match(&line) {
                    version_section_open = true;
                }
            }
        }
        if PYAPI_VERSION_LINE.is_match(&line) {
            file.write_ln(&format!("#origen_pyapi = \"{}\"", version));
        } else {
            file.write_ln(line);
        }
    }
    Ok(())
}
