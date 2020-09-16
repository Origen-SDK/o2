use super::fmt::cd;
use clap::ArgMatches;
use origen::core::file_handler::File;
use origen::utility::file_utils::{copy, symlink};
use origen::utility::version::{to_pep440, to_semver};
use origen::{Result, STATUS};
use regex::Regex;
use semver::Version;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

pub fn run(matches: &ArgMatches) {
    if let Some(v) = matches.value_of("version") {
        let mut version_bad = false;
        let version;
        match to_semver(v) {
            Ok(ver) => version = ver,
            Err(_e) => {
                version = v.to_string();
                version_bad = true;
            }
        }
        if version_bad || Version::parse(&version).is_err() {
            display_redln!(
                "Invalid version: '{}', must be a semantic version like 1.2.3 or 1.2.3.dev4 (1.2.3-dev4 also accepted)",
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
            &version,
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("origen")
                .join("Cargo.toml"),
            &version,
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("pyapi")
                .join("Cargo.toml"),
            &version,
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("python")
                .join("pyproject.toml"),
            &to_pep440(&version).unwrap(),
        )
        .expect("Couldn't write version");
        return;
    }

    // Build the latest CLI, this can be requested from an Origen workspace or an app workspace that is
    // locally referencing an Origen workspace
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

    // Builds the top-level 'origen' Python package, no rust involvement, only available within an Origen workspace
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
        // publish it to PyPI, only available within an Origen workspace
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
            let mut maturin_args = vec![
                "build",
                "--no-sdist", // Local building of the pyapi will not be supported
                "--release",
            ];
            let python_interpreter;
            if let Ok(ver) = std::env::var("PYTHON_INTERPRETER") {
                maturin_args.push("--interpreter");
                python_interpreter = format!("{}", ver);
                maturin_args.push(&python_interpreter);
            } else if let Ok(ver) = std::env::var("PYTHON_VERSION") {
                maturin_args.push("--interpreter");
                python_interpreter = format!("python{}", ver);
                maturin_args.push(&python_interpreter);
            }
            Command::new("maturin")
                .args(&maturin_args)
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

        // A standard (non-release) build, this can be requested from an Origen workspace or an app workspace that
        // is locally referencing an Origen workspace
        } else {
            let pyapi_dir = STATUS.origen_wksp_root.join("rust").join("pyapi");
            cd(&pyapi_dir);
            display!("");
            Command::new("cargo")
                .args(&["build"])
                .status()
                .expect("failed to execute process");
            if cfg!(windows) {
                let link = pyapi_dir.join("target").join("_origen.pyd");
                let target = pyapi_dir.join("target").join("debug").join("_origen.dll");
                if link.exists() {
                    std::fs::remove_file(&link).expect(&format!(
                        "Couldn't delete existing _origen.dll at '{}'",
                        link.display()
                    ));
                }
                // Copy rather than link the file for now to avoid any issues with symlinks not working in user env
                copy(&target, &link).expect(&format!(
                    "Couldn't copy file from '{}' to '{}",
                    target.display(),
                    link.display()
                ));
            } else {
                let link = pyapi_dir.join("target").join("_origen.so");
                let target = pyapi_dir.join("target").join("debug").join("lib_origen.so");
                if link.exists() {
                    std::fs::remove_file(&link).expect(&format!(
                        "Couldn't delete existing _origen.so at '{}'",
                        link.display()
                    ));
                }
                symlink(&target, &link).expect(&format!(
                    "Couldn't create symlink from '{}' to '{}",
                    link.display(),
                    target.display()
                ));
            }
            display!("");
        }
    }
}

/// This is no longer used, decided to accept the version that Poetry wants to use, however
/// keeping it around for a while in case that decision changes.
///
/// Poetry is too opinionated about the versioning and wants to call a pre-release version
/// a release candidate. This fixes the generated version by putting it back to the original
/// Origen version within the wheel package.
fn _fix_wheel_version(dist_dir: &Path) {
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

        let sha = _hash(&metadata_file);
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

fn _hash(file: &Path) -> (String, usize) {
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
