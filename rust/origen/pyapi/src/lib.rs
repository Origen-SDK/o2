mod model;

use origen::{APPLICATION_CONFIG, ORIGEN_CONFIG, STATUS, LOGGER};
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use pyo3::types::{PyDict, PyTuple};
// Imported pyapi modules
use model::PyInit_model;

#[pymodule]
/// This is the top-level _origen module which can be imported by Python
fn _origen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;
    m.add_wrapped(wrap_pyfunction!(clean_mode))?;
    m.add_wrapped(wrap_pyfunction!(target_file))?;

    m.add_wrapped(wrap_pymodule!(logger))?;
    m.add_wrapped(wrap_pymodule!(model))?;
    Ok(())
}

#[pymodule]
fn logger(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(debug))?;
    m.add_wrapped(wrap_pyfunction!(deprecated))?;
    m.add_wrapped(wrap_pyfunction!(error))?;
    m.add_wrapped(wrap_pyfunction!(info))?;
    m.add_wrapped(wrap_pyfunction!(log))?;
    m.add_wrapped(wrap_pyfunction!(success))?;
    m.add_wrapped(wrap_pyfunction!(warning))?;
    m.add_wrapped(wrap_pyfunction!(output_file))?;
    Ok(())
}

macro_rules! pytuple_to_vector_str {
    ($strs:expr) => {
        // There's probably a better way to do this, but hell if I can find it.
        $strs.iter().map( |s| s.to_string()).collect::<Vec<_>>().iter().map( |s| s.as_str()).collect::<Vec<_>>()
    };
}

#[pyfunction(messages="*", _kwargs="**")]
fn debug(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.debug_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages="*", _kwargs="**")]
fn deprecated(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.deprecated_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages="*", _kwargs="**")]
fn error(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.error_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages="*", _kwargs="**")]
fn info(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.info_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages="*", _kwargs="**")]
fn log(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.log_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages="*", _kwargs="**")]
fn success(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.success_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages="*", _kwargs="**")]
fn warning(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.warning_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
fn output_file(_py: Python) -> PyResult<String> {
    Ok(LOGGER.output_file.to_string_lossy().to_string())
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
    let _ = ret.set_item("id", &APPLICATION_CONFIG.id);
    let _ = ret.set_item("target", &APPLICATION_CONFIG.target);
    let _ = ret.set_item("environment", &APPLICATION_CONFIG.environment);
    let _ = ret.set_item("mode", &APPLICATION_CONFIG.mode);
    Ok(ret.into())
}

/// Sanitizes the given mode string and returns it, but will exit the process if it is invalid
#[pyfunction]
fn clean_mode(name: &str) -> PyResult<String> {
    let c = origen::clean_mode(name);
    Ok(c)
}

#[pyfunction]
/// Sanitizes the given target/env name and returns the matching file, but will exit the process
/// if it does not uniquely identify a single target/env file.
fn target_file(name: &str, dir: &str) -> PyResult<String> {
    let c = origen::core::application::target::clean_name(name, dir, true);
    Ok(c)
}
