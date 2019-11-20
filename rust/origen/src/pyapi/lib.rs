use core::{CONFIG, STATUS};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// This is a python module implemented in Rust.
#[pymodule]
fn _origen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;

    Ok(())
}

/// Returns the Origen status which informs whether an app is present, the Origen version,
/// etc.
#[pyfunction]
fn status(py: Python) -> PyResult<PyObject> {
    Ok(STATUS.to_py_dict(&py).into())
}

/// Returns the Origen configuration (as defined in origen.toml files)
#[pyfunction]
fn config(py: Python) -> PyResult<PyObject> {
    Ok(CONFIG.to_py_dict(&py).into())
}

/// Returns the Origen application configuration (as defined in application.toml)
#[pyfunction]
fn app_config(py: Python) -> PyResult<PyObject> {
    Ok(core::application::CONFIG.to_py_dict(&py).into())
}
