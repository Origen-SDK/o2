// Global commands could go here the working dir is the target/debug dir.
//
// Initially tried using predicates and assert_cmd, but didn't
// find them especially helpful. They can be added as dev dependencies.
//
// See the following:
// https://rust-cli.github.io/book/tutorial/testing.html#testing-cli-applications-by-running-them
// https://docs.rs/predicates/1.0.4/predicates/
// https://docs.rs/assert_cmd/1.0.1/assert_cmd/
// https://crates.io/crates/rexpect
//
// helpful examples of using Command. Also possible to do interactive tests if needed
// https://rust-lang-nursery.github.io/rust-cookbook/os/external.html

use std::fs;
use std::process::Command;

// Cargo sets an env var to point to the executable for testing.
fn ogn_cmd() -> String {
    option_env!("CARGO_BIN_EXE_origen")
        .unwrap_or("origen")
        .to_string()
}

#[test]
fn origen_v_responds() -> Result<(), Box<dyn std::error::Error>> {
    // .output()? will wait for completion and return an Output struct
    // see https://doc.rust-lang.org/std/process/struct.Output.html
    let output = Command::new(ogn_cmd()).arg("-v").output()?;

    // check no error was returned
    assert!(output.status.success());

    // get stdout from the command execution in String format for testing
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains(" 2."));

    Ok(())
}

#[test]
fn origen_bad_arg() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(ogn_cmd()).arg("invalid_cmd_here").output()?;

    // check that an error (not success) result was returned
    assert!(!output.status.success());

    // get stderr from the command execution in String format for testing
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("error:"));

    Ok(())
}

const POETRY_PROJECT: &str = r#"[tool.poetry]
name = "migration-cli-test"
version = "1.0.0"
description = "Migration CLI test"
authors = ["Origen-SDK"]

[tool.poetry.dependencies]
python = ">=3.8,<3.13"
idna = "^3.6"
"#;

#[test]
fn env_migrate_dry_run_is_global_and_does_not_write() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pyproject = directory.path().join("pyproject.toml");
    let poetry_lock = directory.path().join("poetry.lock");
    fs::write(&pyproject, POETRY_PROJECT)?;
    fs::write(&poetry_lock, b"poetry lock sentinel")?;

    let output = Command::new(ogn_cmd())
        .args(["env", "migrate", "--dry-run", "--project"])
        .arg(directory.path())
        .output()?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("--- a/pyproject.toml"));
    assert!(stdout.contains("+++ b/pyproject.toml"));
    assert!(stdout.contains("Dry run only; no files were changed."));
    assert_eq!(fs::read_to_string(&pyproject)?, POETRY_PROJECT);
    assert_eq!(fs::read(&poetry_lock)?, b"poetry lock sentinel");
    assert!(!directory.path().join("uv.lock").exists());
    Ok(())
}

#[test]
fn exec_rejects_poetry_only_projects_before_uv() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    fs::write(directory.path().join("pyproject.toml"), POETRY_PROJECT)?;

    let output = Command::new(ogn_cmd())
        .current_dir(directory.path())
        .args(["exec", "python", "-c", "print('must not run')"])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("This application uses Poetry-only metadata"));
    assert!(stderr.contains("origen env migrate --dry-run"));
    assert!(!String::from_utf8(output.stdout)?.contains("must not run"));
    Ok(())
}
