use core::application::pyapi::PyInit_app;
use core::application::APPLICATION_CONFIG;
use core::{ORIGEN_CONFIG, STATUS};
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};

#[pymodule]
/// This is a python module implemented in Rust.
fn _origen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;

    // From core/application/pyapi.rs
    m.add_wrapped(wrap_pymodule!(app))?;
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
    Ok(ORIGEN_CONFIG.to_py_dict(&py).into())
}

/// Returns the Origen application configuration (as defined in application.toml)
#[pyfunction]
fn app_config(py: Python) -> PyResult<PyObject> {
    Ok(APPLICATION_CONFIG.to_py_dict(&py).into())
}
