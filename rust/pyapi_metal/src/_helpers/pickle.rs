use pyo3::prelude::*;
use pyo3::conversion::AsPyPointer;
use pyo3::types::PyBytes;


pub fn pickle(py: Python, object: &impl AsPyPointer) -> PyResult<Vec<u8>> {
    let pickle = PyModule::import(py, "pickle")?;
    pickle
        .getattr("dumps")?
        .call1((object,))?
        .extract::<Vec<u8>>()
}

pub fn depickle<'a>(py: Python<'a>, object: &Vec<u8>) -> PyResult<&'a PyAny> {
    let pickle = PyModule::import(py, "pickle")?;
    let bytes = PyBytes::new(py, object);
    pickle.getattr("loads")?.call1((bytes,))
}
