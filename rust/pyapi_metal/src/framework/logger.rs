use origen_metal::LOGGER;
use crate::pypath;
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
    subm.add_wrapped(wrap_pyfunction!(get_verbosity))?;
    subm.add_wrapped(wrap_pyfunction!(get_keywords))?;
    subm.add_class::<Logger>()?;
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

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn debug(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.debug_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn deprecated(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.deprecated_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn error(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.error_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn info(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.info_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn success(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.success_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn warning(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.warning_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn display(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    LOGGER.display_block(&pytuple_to_vector_str!(messages));
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
fn log(_py: Python, messages: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<()> {
    display(_py, messages, _kwargs)
}

#[pyfunction]
#[pyo3(signature=(*messages, **_kwargs))]
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

#[pyfunction]
fn get_verbosity() -> PyResult<u8> {
    Ok(LOGGER.verbosity())
}

#[pyfunction]
fn get_keywords() -> PyResult<Vec<String>> {
    Ok(LOGGER.keywords())
}

/// Class-like wrapper for Logger
#[pyclass]
pub struct Logger {}

#[pymethods]
impl Logger {

    #[new]
    fn new() -> Self {
        Self {}
    }

    #[pyo3(signature=(*messages, **kwargs))]
    fn debug(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        debug(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn deprecated(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        deprecated(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn error(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        error(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn info(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        info(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn success(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        success(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn warning(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        warning(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn display(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        display(py, messages, kwargs)
    }
    
    #[pyo3(signature=(*messages, **kwargs))]
    fn log(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        log(py, messages, kwargs)
    }

    #[pyo3(signature=(*messages, **kwargs))]
    fn trace(&self, py: Python, messages: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        trace(py, messages, kwargs)
    }

    #[getter]
    fn output_file(&self, py: Python) -> PyResult<PyObject> {
        Ok(pypath!(py, LOGGER.output_file().display()))
    }

    #[getter]
    fn get_verbosity(&self) -> PyResult<u8> {
        get_verbosity()
    }

    #[setter]
    fn set_verbosity(&self, level: u8) -> PyResult<()> {
        set_verbosity(level)
    }

    #[getter]
    fn get_keywords(&self) -> PyResult<Vec<String>> {
        get_keywords()
    }

    #[setter]
    fn set_keywords(&self, keywords: Vec<String>) -> PyResult<()> {
        set_verbosity_keywords(keywords)
    }
}
