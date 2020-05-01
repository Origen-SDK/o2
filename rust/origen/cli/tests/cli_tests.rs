
// Global commands could go here the working dir is the target/debug dir.
//
// I initially tried using predicates and assert_cmd and didn't
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

use std::process::Command;

// There's probably a prettier way to do this, but don't feel like fighting at the moment
// Cargo sets an env var to point to the executable for testing. In the CI env that
// wasn't happening. So, I'm setting another env and checking for it if the cargo
// env isn't set. Probably could have just used the same env var name...
fn ogn_cmd() -> String {
    let retval;
    match option_env!("CARGO_BIN_EXE_ORIGEN"){
        Some(x) => retval = x,
        None => retval = option_env!("TRAVIS_ORIGEN_CLI").unwrap(),
    }
    String::from(retval)
}

#[test]
fn origen_v_responds() -> Result<(), Box<dyn std::error::Error>> {
    // .output()? will wait for completion and return an Output struct
    // see https://doc.rust-lang.org/std/process/struct.Output.html
    let output = Command::new(ogn_cmd())
        .arg("-v")
        .output()?;
    
    // check no error was returned
    assert!(output.status.success());

    // get stdout from the command execution in String format for testing
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Origen"));
    assert!(stdout.contains(" 2."));

    Ok(())
}

#[test]
fn origen_bad_arg() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(ogn_cmd())
        .arg("invalid_cmd_here")
        .output()?;

    // check that an error (not success) result was returned
    assert!(!output.status.success());

    // get stderr from the command execution in String format for testing
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("error:"));

    Ok(())
}