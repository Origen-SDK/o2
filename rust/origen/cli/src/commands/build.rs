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
        cd(&STATUS.origen_wksp_root.join("example"));
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
        file.write_ln(line);
    }
    Ok(())
}
