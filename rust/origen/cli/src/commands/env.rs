//! Manage an application's UV environment.

use super::_prelude::*;
use crate::python::{python_version, uv_version, MIN_PYTHON_VERSION, PYTHON_CONFIG};
use origen::core::status::search_for;
use origen::core::term::*;
use semver::VersionReq;
use std::process::Command;

pub const BASE_CMD: &'static str = "env";
static MINIMUM_UV_VERSION: &str = "0.12.5";

/// Python versions below this cannot execute UV's Windows console-script
/// launchers. See `provision_with_pip`.
static MINIMUM_UV_LAUNCHER_PYTHON: &str = "3.8.0";

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Manage your application's Origen/Python environment",
    { |cmd: App| { cmd.arg_required_else_help(true) } },
    core_subcmd__no_exts__no_app_opts!(
        "setup",
        "Create or synchronize the UV environment from uv.lock",
        {
            |cmd: App| {
                cmd.arg(
                    Arg::new("origen")
                        .long("origen")
                        .help("Point this application at an Origen source checkout. Note: this permanently records the path in the application's pyproject.toml and uv.lock; remove it with 'uv remove origen'")
                        .action(SetArg),
                )
            }
        }
    ),
    core_subcmd__no_exts__no_app_opts!(
        "update",
        "Upgrade and synchronize the application's locked dependencies",
        { |cmd: App| { cmd } }
    )
);

pub fn run(invocation: &clap::ArgMatches) -> origen::Result<()> {
    require_python();
    require_uv();

    let app_root = &origen::app().unwrap().root;
    let pyproject = app_root.join("pyproject.toml");
    if !pyproject.exists() {
        display_redln!(
            "Application pyproject.toml was not found at {}",
            pyproject.display()
        );
        std::process::exit(1);
    }

    match invocation.subcommand_name() {
        Some("update") => {
            run_uv(app_root, &["lock", "--upgrade"])?;
            provision(app_root)?;
        }
        Some("setup") => {
            if let Some(path) = invocation
                .subcommand_matches("setup")
                .unwrap()
                .get_one::<String>("origen")
            {
                let path = std::path::Path::new(path)
                    .canonicalize()
                    .expect("The path supplied to --origen does not exist");
                let (found, root) = search_for(vec![".origen_dev_workspace"], false, &path);
                if !found {
                    display_redln!(
                        "An Origen source checkout was not found at {}",
                        path.display()
                    );
                    std::process::exit(1);
                }
                let package = root.join("python").join("origen");
                let package_str = package.to_string_lossy();
                displayln!(
                    "Adding '{}' as a path dependency. This edits pyproject.toml and uv.lock; run 'uv remove origen' to undo it before committing.",
                    package_str
                );
                run_uv(app_root, &["add", package_str.as_ref()])?;
            }
            provision(app_root)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// Install the locked environment.
///
/// Normally this is a plain `uv sync`. Windows with Python 3.7 takes a
/// different route; see `provision_with_pip` for why.
fn provision(app_root: &std::path::Path) -> origen::Result<()> {
    if needs_pip_provisioning() {
        provision_with_pip(app_root)
    } else {
        run_uv(app_root, &["sync", "--all-groups", "--no-editable"])
    }
}

/// UV writes Windows console scripts as a trampoline with a zip archive
/// appended. The trampoline runs `python.exe <the .exe itself>` and relies on
/// the interpreter executing that archive. CPython's old C `zipimport`, used up
/// to and including 3.7, rejects it and falls back to parsing the executable as
/// source, so *every* console script in such an environment is unusable --
/// `pytest`, and `origen` itself. CPython 3.8 replaced `zipimport` with the
/// pure-Python implementation, which reads it correctly.
///
/// pip's launchers are built differently and do work there, which is how this
/// combination behaved before O2 moved to UV. Retire this path when O2 drops
/// Python 3.7, or when UV's launcher becomes readable by it.
fn needs_pip_provisioning() -> bool {
    if !cfg!(windows) {
        return false;
    }
    let minimum = semver::Version::parse(MINIMUM_UV_LAUNCHER_PYTHON).unwrap();
    python_version().map_or(false, |version| version < minimum)
}

fn provision_with_pip(app_root: &std::path::Path) -> origen::Result<()> {
    displayln!(
        "Python {} on Windows cannot execute UV's console-script launchers, so this \
         environment will be installed with pip. The contents still come from uv.lock.",
        python_version().map_or("<unknown>".to_string(), |v| v.to_string())
    );

    let venv = app_root.join(".venv");
    // Pin the interpreter. Without it UV picks its own preferred Python for the
    // environment, which is not necessarily the one that was discovered here --
    // it may even download a different version - and the environment would then
    // not match the interpreter the rest of the workspace is using.
    let interpreter = discovered_python_executable()?;
    run_uv(
        app_root,
        &[
            "venv",
            "--seed",
            // Replace any existing environment. Unlike `uv sync`, installing
            // with pip cannot prune packages that are no longer in the lock, so
            // starting clean is what keeps this path equivalent to a sync.
            "--clear",
            "--python",
            &interpreter,
            &venv.to_string_lossy(),
        ],
    )?;

    // Export the resolved lock rather than re-resolving, so pip installs
    // exactly what `uv sync` would have. The project itself is excluded here
    // and installed separately, matching `--no-editable`.
    let requirements =
        std::env::temp_dir().join(format!("origen-uv-requirements-{}.txt", std::process::id()));
    let requirements_arg = requirements.to_string_lossy().to_string();
    let export = run_uv(
        app_root,
        &[
            "export",
            "--frozen",
            "--all-groups",
            "--no-hashes",
            "--no-emit-project",
            "-o",
            &requirements_arg,
        ],
    );
    if export.is_err() {
        let _ = std::fs::remove_file(&requirements);
        return export;
    }

    let python = venv.join("Scripts").join("python.exe");
    let result = (|| -> origen::Result<()> {
        run_python(
            &python,
            &["-m", "pip", "install", "-r", &requirements_arg],
            app_root,
        )?;
        if is_installable_project(app_root) {
            run_python(
                &python,
                &["-m", "pip", "install", "--no-deps", "."],
                app_root,
            )
        } else {
            displayln!("Project is virtual (tool.uv package = false); dependencies only.");
            Ok(())
        }
    })();
    let _ = std::fs::remove_file(&requirements);
    result
}

/// Does UV install this project itself, or only its dependencies?
///
/// A project marked `[tool.uv] package = false` is virtual: `uv sync` installs
/// its dependencies and never builds the project. Installing it with pip anyway
/// makes setuptools guess at the layout and publish whatever directories it
/// happens to find.
fn is_installable_project(root: &std::path::Path) -> bool {
    let contents = match std::fs::read_to_string(root.join("pyproject.toml")) {
        Ok(c) => c,
        Err(_) => return true,
    };
    let parsed: toml::Value = match toml::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return true,
    };
    parsed
        .get("tool")
        .and_then(|t| t.get("uv"))
        .and_then(|uv| uv.get("package"))
        .and_then(|p| p.as_bool())
        .unwrap_or(true)
}

fn run_python(
    python: &std::path::Path,
    args: &[&str],
    cwd: &std::path::Path,
) -> origen::Result<()> {
    let mut command = Command::new(python);
    command.args(args).current_dir(cwd);
    log_debug!("Running Python command: {:?}", command);
    displayln!("+ {} {}", python.display(), args.join(" "));
    let status = command.status()?;
    if !status.success() {
        bail!("'{}' failed with status {}", args.join(" "), status)
    }
    Ok(())
}

/// Resolve the discovered Python command to a concrete interpreter path, so it
/// can be handed to UV unambiguously.
fn discovered_python_executable() -> origen::Result<String> {
    let output = Command::new(&PYTHON_CONFIG.command)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()?;
    if !output.status.success() {
        bail!(
            "Could not resolve the path of Python command '{}'",
            PYTHON_CONFIG.command
        )
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        bail!(
            "Python command '{}' did not report an executable path",
            PYTHON_CONFIG.command
        )
    }
    Ok(path)
}

fn require_python() {
    print!("Is a suitable Python available? ... ");
    if PYTHON_CONFIG.available {
        greenln("YES");
    } else {
        redln("NO");
        display_redln!(
            "Could not find Python >= {}. Install a supported Python and try again.",
            MIN_PYTHON_VERSION
        );
        std::process::exit(1);
    }
}

fn run_uv(root: &std::path::Path, args: &[&str]) -> origen::Result<()> {
    let mut command = Command::new("uv");
    command.arg("--project").arg(root).args(args);
    log_debug!("Running UV command: {:?}", command);
    // Echoed unconditionally: provisioning is long-running and its failures are
    // otherwise opaque, particularly in CI where no verbosity flag is passed.
    displayln!("+ uv {}", args.join(" "));
    let status = command.status()?;
    if !status.success() {
        bail!("UV command failed with status {}", status);
    }
    Ok(())
}

fn require_uv() {
    let required = VersionReq::parse(&format!(">={}", MINIMUM_UV_VERSION)).unwrap();
    if let Some(version) = uv_version() {
        if required.matches(&version) {
            displayln!("UV {} is available", version);
            return;
        }
    }

    display_redln!(
        "UV >= {} is required. Install the standalone UV binary from https://docs.astral.sh/uv/getting-started/installation/ and rerun the command.",
        MINIMUM_UV_VERSION
    );
    std::process::exit(1);
}
