use super::fmt::cd;
use clap::ArgMatches;
use origen::core::file_handler::File;
use origen::{Result, STATUS};
use regex::Regex;
use semver::Version;
use sha2::{Digest, Sha256};
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
        return;
    }

    // Build the latest CLI
    if matches.is_present("cli") {
        cd(&STATUS
            .origen_wksp_root
            .join("rust")
            .join("origen")
            .join("cli"));
        display!("");
        let mut args = vec!["build"];
        if matches.is_present("release") || matches.is_present("publish") {
            args.push("--release");
        }
        Command::new("cargo")
            .args(&args)
            .status()
            .expect("failed to execute process");
        display!("");

    // Builds the top-level 'origen' Python package, no rust involvement
    } else if matches.is_present("python") {
        let wheel_dir = &STATUS.origen_wksp_root.join("python").join("dist");
        // Make sure we are not about to upload any stale/old artifacts
        if wheel_dir.exists() {
            std::fs::remove_dir_all(&wheel_dir).expect("Couldn't delete existing wheel dir");
        }
        cd(&STATUS.origen_wksp_root.join("python"));

        dependency_on_pyapi(true).expect("Couldn't enable dependency on origen_pyapi");

        Command::new("poetry")
            .args(&["build", "--no-interaction", "--format", "wheel"])
            .status()
            .expect("failed to build origen for release");

        fix_wheel_version(wheel_dir);
        cd(&STATUS.origen_wksp_root.join("python"));

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

    // Build the PyAPI by default
    } else {
        // A release build will also build the origen_pyapi Python package and optionally
        // publish is to PyPI
        if matches.is_present("release") || matches.is_present("publish") {
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

        // A standard (non-release) build
        } else {
            // The default build will compile the latest PyAPI and copy it into
            // the example app's Python env

            // If this command is launched within a different test app, then the build will apply
            // to that app instead.
            let mut app = STATUS.origen_wksp_root.join("test_apps").join("python_app");
            let apps_dir = STATUS.origen_wksp_root.join("test_apps");
            if let Ok(pwd) = std::env::current_dir() {
                if pwd.starts_with(&apps_dir) {
                    app = pwd;
                    while app.parent().unwrap() != apps_dir {
                        app.pop();
                    }
                }
            }

            cd(&app);
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

/// Poetry is too opinionated about the versioning and wants to call a pre-release version
/// a release candidate. This fixes the generated version by putting it back to the original
/// Origen version within the wheel package.
fn fix_wheel_version(dist_dir: &Path) {
    if STATUS.origen_version.pre.is_empty() {
        return;
    }
    let paths = std::fs::read_dir(dist_dir).unwrap();

    for path in paths {
        let wheel_file_name;
        let old_version;
        let new_version = STATUS.origen_version.to_string();
        let underscored_new_version = new_version.replace("-", "_");

        // Get the mangled version from the whl file name
        if let Some(file) = path.unwrap().path().file_name() {
            if let Some(file) = file.to_str() {
                if !file.ends_with(".whl") {
                    continue;
                }
                wheel_file_name = file.to_string();
                let re = regex::Regex::new(r"origen-(\d+\.\d+\.\d+.+\d+)-py3.*").unwrap();
                let captures = re.captures(&wheel_file_name).unwrap();
                old_version = captures.get(1).unwrap().as_str().to_string();
            } else {
                continue;
            }
        } else {
            continue;
        }

        cd(dist_dir);

        // Unzip the wheel and replace all occurrences of the mangled version
        Command::new("unzip")
            .arg(&wheel_file_name)
            .status()
            .expect("failed to unzip wheel file");

        std::fs::remove_file(&wheel_file_name).expect("Couldn't delete the original wheel file");

        let new_wheel_file_name = wheel_file_name.replace(&old_version, &underscored_new_version);

        let old_info_dir_name = format!("origen-{}.dist-info", &old_version);
        let new_info_dir_name = format!("origen-{}.dist-info", &underscored_new_version);

        std::fs::rename(&old_info_dir_name, &new_info_dir_name).expect("couldn't rename info file");

        let metadata_file = Path::new(&new_info_dir_name).join("METADATA");
        let record_file = Path::new(&new_info_dir_name).join("RECORD");

        // Update the package version in the METADATA file
        let mut contents = std::fs::read_to_string(&metadata_file).expect("Couldn't read METADATA");
        contents = contents.replace(
            &format!("Version: {}", &old_version),
            &format!("Version: {}", &new_version),
        );
        File::create(metadata_file.clone()).write(&contents);

        // Update the dir names and hash of the METADATA file in the RECORD file
        let mut contents = std::fs::read_to_string(&record_file).expect("Couldn't read RECORD");
        contents = contents.replace(
            &format!("origen-{}.dist", &old_version),
            &format!("origen-{}.dist", &underscored_new_version),
        );

        let sha = hash(&metadata_file);
        let new_meta_line = format!(
            "origen-{}.dist-info/METADATA,sha256={},{}",
            &underscored_new_version, sha.0, sha.1
        );

        let lines: Vec<&str> = contents
            .split("\n")
            .into_iter()
            .map(|line| {
                if line.contains("dist-info/METADATA") {
                    &new_meta_line
                } else {
                    line
                }
            })
            .collect();

        File::create(record_file).write(&lines.join("\n"));

        // Finally, zip up the new wheel and clean up
        Command::new("zip")
            .args(&["-r", &new_wheel_file_name, "origen", &new_info_dir_name])
            .status()
            .expect("failed to zip wheel file");

        std::fs::remove_dir_all("origen").expect("Couldn't delete origen dir");
        std::fs::remove_dir_all(&new_info_dir_name)
            .expect(&format!("Couldn't delete {} dir", &new_info_dir_name));
    }
}

fn hash(file: &Path) -> (String, usize) {
    let contents =
        std::fs::read_to_string(file).expect(&format!("Couldn't read {}", file.display()));
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let hash = hasher.finalize();
    let b = base64_url::encode(&hash);
    (b, contents.as_bytes().len())
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
