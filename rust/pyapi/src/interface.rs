use origen::Error;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
//use std::collections::HashMap;

#[pymodule]
pub fn interface(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyInterface>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyInterface {
    //python_testers: HashMap<String, PyObject>,
//instantiated_testers: HashMap<String, PyObject>,
//metadata: Vec<PyObject>,
}

#[pymethods]
impl PyInterface {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({ PyInterface {} });
    }
}
