use core::CONFIG;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// This module is a python module implemented in Rust.
#[pymodule]
fn _origen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(version))?;
    m.add_wrapped(wrap_pyfunction!(root))?;

    Ok(())
}

/// Returns the application root directory when invoked within an Origen application
/// workspace, if not in an application returns an empty string
#[pyfunction]
fn root() -> PyResult<String> {
    if CONFIG.is_app_present {
        let path = CONFIG.root.clone().into_os_string().into_string().unwrap();
        Ok(path)
    } else {
        Ok("".to_string())
    }
}

/// Returns the Origen version
#[pyfunction]
fn version() -> PyResult<String> {
    let v = CONFIG.origen_version.to_string();
    Ok(v)
}
