mod application;
mod model;

use origen::{APPLICATION_CONFIG, ORIGEN_CONFIG, STATUS};
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
// Imported pyapi modules
use application::PyInit_app;
use model::PyInit_model;

#[pymodule]
/// This is the top-level _origen module which can be imported by Python
fn _origen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;

    m.add_wrapped(wrap_pymodule!(app))?;
    m.add_wrapped(wrap_pymodule!(model))?;
    Ok(())
}

/// Returns the Origen status which informs whether an app is present, the Origen version,
/// etc.
#[pyfunction]
fn status(py: Python) -> PyResult<PyObject> {
    let ret = pyo3::types::PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let _ = ret.set_item("is_app_present", &STATUS.is_app_present);
    let _ = ret.set_item("root", format!("{}", STATUS.root.display()));
    let _ = ret.set_item("origen_version", &STATUS.origen_version.to_string());
    Ok(ret.into())
}

/// Returns the Origen configuration (as defined in origen.toml files)
#[pyfunction]
fn config(py: Python) -> PyResult<PyObject> {
    let ret = pyo3::types::PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let _ = ret.set_item("python_cmd", &ORIGEN_CONFIG.python_cmd);
    let _ = ret.set_item("pkg_server", &ORIGEN_CONFIG.pkg_server);
    let _ = ret.set_item("pkg_server_push", &ORIGEN_CONFIG.pkg_server_push);
    let _ = ret.set_item("pkg_server_pull", &ORIGEN_CONFIG.pkg_server_pull);
    let _ = ret.set_item("some_val", &ORIGEN_CONFIG.some_val);
    Ok(ret.into())
}

/// Returns the Origen application configuration (as defined in application.toml)
#[pyfunction]
fn app_config(py: Python) -> PyResult<PyObject> {
    let ret = pyo3::types::PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let _ = ret.set_item("name", &APPLICATION_CONFIG.name);
    let _ = ret.set_item("target", &APPLICATION_CONFIG.target);
    let _ = ret.set_item("environment", &APPLICATION_CONFIG.environment);
    Ok(ret.into())
}
