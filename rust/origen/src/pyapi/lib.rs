use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::fs;

#[macro_use]
extern crate lazy_static;

pub mod workspace;
pub mod commands;

lazy_static! {
    pub static ref CONFIG: workspace::Config = workspace::Config::default();
}

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
   // The file has been placed there by the build script.
   include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[pyfunction]
fn hello() -> PyResult<String> {
    let contents = fs::read_to_string(CONFIG.root.join("config").join("version.toml"))
        .expect("Something went wrong reading the version file");
    //Ok("Yo!".to_string())
    Ok(contents)
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn _origen(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(hello))?;

    Ok(())
}
