use super::fmt::cd;
use crate::CommandHelp;
use clap::{App, Arg, ArgMatches, SubCommand};
use origen::core::file_handler::File;
use origen::utility::file_utils::{symlink, with_dir};
use origen::utility::version::Version;
use origen::{Result, STATUS};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

pub fn define<'a>(app: App<'a, 'a>) -> (App<'a, 'a>, CommandHelp) {
    let help = match STATUS.is_origen_present {
        true => "Build and publish Origen, builds the pyapi Rust package by default",
        false => "Build Origen",
    };

    let mut cmd = SubCommand::with_name("build").about(help).arg(
        Arg::with_name("cli")
            .long("cli")
            .required(false)
            .takes_value(false)
            .display_order(1)
            .help("Build the CLI (instead of the Python API)"),
    );

    if STATUS.is_origen_present {
        cmd = cmd
            .arg(
                Arg::with_name("release")
                    .long("release")
                    .required(false)
                    .takes_value(false)
                    .display_order(1)
                    .help("Build a release version (applied by default with --publish and only applicable to Rust builds)"),
            )
            .arg(
                Arg::with_name("target")
                    .long("target")
                    .required(false)
                    .takes_value(true)
                    .display_order(1)
                    .help("The Rust h/ware target (passed directly to Cargo build)"),
            )
            .arg(
                Arg::with_name("publish")
                    .long("publish")
                    .required(false)
                    .takes_value(false)
                    .display_order(1)
                    .help("Publish packages (e.g. to PyPI) after building"),
            )
            .arg(
                Arg::with_name("dry_run")
                    .long("dry-run")
                    .required(false)
                    .takes_value(false)
                    .display_order(1)
                    .help("Use with --publish to perform a full dry run of the publishable build without actually publishing it"),
            )
            .arg(
                Arg::with_name("version")
                    .long("version")
                    .required(false)
                    .takes_value(true)
                    .value_name("VERSION")
                    .display_order(1)
                    .help("Set the version (of all components) to the given value"),
            )
            .arg(
                Arg::with_name("metal")
                    .long("metal")
                    .takes_value(false)
                    .display_order(1)
                    .help("Build the metal_pyapi"),
            );
    }

    let help = CommandHelp {
        name: "build".to_string(),
        help: help.to_string(),
        shortcut: None,
    };

    (app.subcommand(cmd), help)
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    if let Some(v) = matches.value_of("version") {
        let mut version_bad = false;
        let version;
        match Version::new_semver(v) {
            Ok(ver) => version = ver.to_string(),
            Err(_e) => {
                version = v.to_string();
                version_bad = true;
            }
        }
        if version_bad || Version::new_semver(&version).is_err() {
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
            &version.to_string(),
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("origen")
                .join("Cargo.toml"),
            &version.to_string(),
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("rust")
                .join("pyapi")
                .join("Cargo.toml"),
            &version.to_string(),
        )
        .expect("Couldn't write version");
        write_version(
            &STATUS
                .origen_wksp_root
                .join("python")
                .join("origen")
                .join("pyproject.toml"),
            &Version::new_pep440(&version.to_string())
                .unwrap()
                .to_string(),
        )
        .expect("Couldn't write version");
        return Ok(());
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

    // Build the metal_pyapi
    } else if matches.is_present("metal") {
        let pyapi_dir = &STATUS.origen_wksp_root.join("rust").join("pyapi_metal");
        cd(&pyapi_dir);

        let mut args = vec!["build"];
        let mut target = "debug";
        let mut arch_target = None;

        if matches.is_present("release") {
            args.push("--release");
            target = "release";
        }
        if let Some(t) = matches.value_of("target") {
            args.push("--target");
            args.push(t);
            arch_target = Some(t);
        }

        Command::new("cargo")
            .args(&args)
            .status()
            .expect("failed to execute process");

        if cfg!(windows) {
            let link = &STATUS
                .origen_wksp_root
                .join("python")
                .join("origen_metal")
                .join("origen_metal")
                .join("_origen_metal.pyd");
            let target = match arch_target {
                None => pyapi_dir
                    .join("target")
                    .join(target)
                    .join("origen_metal.dll"),
                Some(t) => pyapi_dir
                    .join("target")
                    .join(t)
                    .join(target)
                    .join("origen_metal.dll"),
            };
            if link.exists() {
                std::fs::remove_file(&link).expect(&format!(
                    "Couldn't delete existing _origen_metal.pyd at '{}'",
                    link.display()
                ));
            }
            // Copy rather than link the file for now to avoid any issues with symlinks not working in user env
            std::fs::copy(&target, &link).expect(&format!(
                "Couldn't copy file from '{}' to '{}",
                target.display(),
                link.display()
            ));
        } else {
            let link = &STATUS
                .origen_wksp_root
                .join("python")
                .join("origen_metal")
                .join("origen_metal")
                .join("_origen_metal.so");
            let target = match arch_target {
                None => pyapi_dir
                    .join("target")
                    .join(target)
                    .join("liborigen_metal.so"),
                Some(t) => pyapi_dir
                    .join("target")
                    .join(t)
                    .join(target)
                    .join("liborigen_metal.so"),
            };
            if link.exists() {
                std::fs::remove_file(&link).expect(&format!(
                    "Couldn't delete existing _origen_metal.so at '{}'",
                    link.display()
                ));
            }
            symlink(&target, &link).expect(&format!(
                "Couldn't create symlink from '{}' to '{}",
                link.display(),
                target.display()
            ));
        }

    // Build the PyAPI by default
    } else {
        // A publish build will also build the origen_pyapi Python package and
        // publish it to PyPI, only available within an Origen workspace
        if matches.is_present("publish") {
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

            // Maturin picks up the version from Rust (pyapi) which is semver-compliant. Need to ensure the version
            // of the generated Python package is pep440 compliant
            let old = STATUS.origen_version.to_string();
            let new = Version::new_pep440(&old.to_string()).unwrap().to_string();
            if old != new {
                change_pyapi_wheel_version(&wheel_dir, &old, &new);
            }

            if matches.is_present("publish") && !matches.is_present("dry_run") {
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

        // A standard (non-published) build, this can be requested from an Origen workspace or an app workspace that
        // is locally referencing an Origen workspace
        } else {
            let pyapi_dir = STATUS.origen_wksp_root.join("rust").join("pyapi");
            cd(&pyapi_dir);
            display!("");

            let mut args = vec!["build"];
            let mut target = "debug";
            let mut arch_target = None;

            if matches.is_present("release") {
                args.push("--release");
                target = "release";
            }
            if let Some(t) = matches.value_of("target") {
                args.push("--target");
                args.push(t);
                arch_target = Some(t);
            }

            Command::new("cargo")
                .args(&args)
                .status()
                .expect("failed to execute process");

            if cfg!(windows) {
                let link = pyapi_dir.join("target").join("_origen.pyd");
                let target = match arch_target {
                    None => pyapi_dir.join("target").join(target).join("_origen.dll"),
                    Some(t) => pyapi_dir
                        .join("target")
                        .join(t)
                        .join(target)
                        .join("_origen.dll"),
                };
                if link.exists() {
                    std::fs::remove_file(&link).expect(&format!(
                        "Couldn't delete existing _origen.dll at '{}'",
                        link.display()
                    ));
                }
                // Copy rather than link the file for now to avoid any issues with symlinks not working in user env
                std::fs::copy(&target, &link).expect(&format!(
                    "Couldn't copy file from '{}' to '{}",
                    target.display(),
                    link.display()
                ));
            } else {
                let link = pyapi_dir.join("target").join("_origen.so");
                let target = match arch_target {
                    None => pyapi_dir.join("target").join(target).join("lib_origen.so"),
                    Some(t) => pyapi_dir
                        .join("target")
                        .join(t)
                        .join(target)
                        .join("lib_origen.so"),
                };
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
    Ok(())
}

fn change_pyapi_wheel_version(dist_dir: &Path, old_version: &str, new_version: &str) {
    if STATUS.origen_version.is_prerelease().unwrap() {
        return;
    }
    let underscored_old_version = old_version.replace("-", "_");

    let paths = std::fs::read_dir(dist_dir).unwrap();

    for path in paths {
        let wheel_file_name;

        // Only process wheel files in the given dir
        if let Some(file) = path.unwrap().path().file_name() {
            if let Some(file) = file.to_str() {
                if !file.ends_with(".whl") {
                    continue;
                }
                wheel_file_name = file.to_string();
            } else {
                continue;
            }
        } else {
            continue;
        }

        let _ = with_dir(dist_dir, || {
            // Unzip the wheel and replace all occurrences of the mangled version
            Command::new("unzip")
                .arg(&wheel_file_name)
                .status()
                .expect("failed to unzip wheel file");

            std::fs::remove_file(&wheel_file_name)
                .expect("Couldn't delete the original wheel file");

            let new_wheel_file_name =
                wheel_file_name.replace(&underscored_old_version, &new_version);

            let old_info_dir_name = format!("origen_pyapi-{}.dist-info", &underscored_old_version);
            let new_info_dir_name = format!("origen_pyapi-{}.dist-info", &new_version);

            std::fs::rename(&old_info_dir_name, &new_info_dir_name)
                .expect("couldn't rename info file");

            let metadata_file = Path::new(&new_info_dir_name).join("METADATA");
            let record_file = Path::new(&new_info_dir_name).join("RECORD");

            // Update the package version in the METADATA file
            let mut contents =
                std::fs::read_to_string(&metadata_file).expect("Couldn't read METADATA");
            contents = contents.replace(
                &format!("Version: {}", &old_version),
                &format!("Version: {}", &new_version),
            );
            File::create(metadata_file.clone()).write(&contents);

            // Update the dir names and hash of the METADATA file in the RECORD file
            let mut contents = std::fs::read_to_string(&record_file).expect("Couldn't read RECORD");
            contents = contents.replace(
                &format!("origen_pyapi-{}.dist", &underscored_old_version),
                &format!("origen_pyapi-{}.dist", &new_version),
            );

            let sha = hash(&metadata_file);
            let new_meta_line = format!(
                "origen_pyapi-{}.dist-info/METADATA,sha256={},{}",
                &new_version, sha.0, sha.1
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

            for entry in std::fs::read_dir(dist_dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if file_name.to_str().unwrap().starts_with("_origen") {
                            std::fs::remove_file(&path).expect("Couldn't delete _origen");
                        }
                    }
                }
            }
            std::fs::remove_dir_all(&new_info_dir_name)
                .expect(&format!("Couldn't delete {} dir", &new_info_dir_name));

            Ok(())
        });
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
