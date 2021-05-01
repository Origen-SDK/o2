use pyo3::prelude::*;
use crate::pypath;
use pyo3::types::PyDict;
use std::collections::HashMap;

pub fn to_py_paths<T: std::fmt::Display>(paths: &Vec<T>) -> PyResult<Vec<PyObject>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut retn: Vec<PyObject> = vec![];
    for p in paths {
        retn.push(pypath!(py, format!("{}", p)));
    }
    Ok(retn)
}

pub fn hashmap_to_pydict<'p>(py: Python<'p>, hmap: &HashMap<String, String>) -> PyResult<&'p PyDict> {
    let py_config = PyDict::new(py);
    for (k, v) in hmap.iter() {
        py_config.set_item(k, v)?;
    }
    Ok(py_config)
}
