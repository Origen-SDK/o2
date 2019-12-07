use origen::LOGGER;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::wrap_pyfunction;

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
fn log(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.log_block(&pytuple_to_vector_str!(messages));
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

#[pyfunction]
fn output_file(_py: Python) -> PyResult<String> {
    Ok(LOGGER.output_file.to_string_lossy().to_string())
}
