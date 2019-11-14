use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
#[macro_use]
extern crate lazy_static;

pub mod workspace;

lazy_static! {
    pub static ref CONFIG: workspace::Config = workspace::Config::default();
}

#[pyfunction]
fn hello() -> PyResult<String> {
    Ok("Yo!".to_string())
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn origen(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(hello))?;

    Ok(())
}
