use crate::pypath;
use pyo3::prelude::*;

pub fn to_py_paths<T: std::fmt::Display>(paths: &Vec<T>) -> PyResult<Vec<PyObject>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut retn: Vec<PyObject> = vec![];
    for p in paths {
        retn.push(pypath!(py, format!("{}", p)));
    }
    Ok(retn)
}
