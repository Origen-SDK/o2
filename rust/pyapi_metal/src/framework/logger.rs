use origen_metal::LOGGER;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::wrap_pyfunction;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "logger")?;
    subm.add_wrapped(wrap_pyfunction!(debug))?;
    subm.add_wrapped(wrap_pyfunction!(deprecated))?;
    subm.add_wrapped(wrap_pyfunction!(error))?;
    subm.add_wrapped(wrap_pyfunction!(info))?;
    subm.add_wrapped(wrap_pyfunction!(success))?;
    subm.add_wrapped(wrap_pyfunction!(warning))?;
    subm.add_wrapped(wrap_pyfunction!(display))?;
    subm.add_wrapped(wrap_pyfunction!(log))?;
    subm.add_wrapped(wrap_pyfunction!(trace))?;
    subm.add_wrapped(wrap_pyfunction!(output_file))?;
    subm.add_wrapped(wrap_pyfunction!(set_verbosity))?;
    subm.add_wrapped(wrap_pyfunction!(set_verbosity_keywords))?;
    m.add_submodule(subm)?;
    Ok(())
}

macro_rules! pytuple_to_vector_str {
    ($strs:expr) => {
        // There's probably a better way to do this, but hell if I can find it.
        $strs
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
    };
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn debug(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.debug_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn deprecated(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.deprecated_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn error(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.error_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn info(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.info_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn success(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.success_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn warning(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.warning_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn display(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.display_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn log(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    display(_py, messages, _kwargs)
}

#[pyfunction(messages = "*", _kwargs = "**")]
fn trace(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.trace_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
fn output_file(_py: Python) -> PyResult<String> {
    Ok(LOGGER.output_file().to_string_lossy().to_string())
}

#[pyfunction]
fn set_verbosity(level: u8) -> PyResult<()> {
    LOGGER.set_verbosity(level)?;
    Ok(())
}

#[pyfunction]
fn set_verbosity_keywords(keywords: Vec<String>) -> PyResult<()> {
    LOGGER.set_verbosity_keywords(keywords)?;
    Ok(())
}
