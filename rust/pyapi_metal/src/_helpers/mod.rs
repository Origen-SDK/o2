pub mod typed_value;
pub mod pickle;

use crate::pypath;
use pyo3::prelude::*;
use std::path::PathBuf;

pub fn to_py_paths<T: std::fmt::Display>(paths: &Vec<T>) -> PyResult<Vec<PyObject>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut retn: Vec<PyObject> = vec![];
    for p in paths {
        retn.push(pypath!(py, format!("{}", p)));
    }
    Ok(retn)
}

/// Attempts to extract a PyAny as a Rust PathBuf.
/// Accepts either a str or pathlib.Path object.
pub fn pypath_as_string(path: &PyAny) -> PyResult<String> {
    if let Ok(p) = path.extract::<String>() {
        Ok(p)
    } else if path.get_type().name()?.to_string() == "Path"
        || path.get_type().name()?.to_string() == "WindowsPath"
        || path.get_type().name()?.to_string() == "PosixPath"
    {
        Ok(path.call_method0("__str__")?.extract::<String>()?)
    } else {
        crate::type_error!(&format!(
            "Cannot extract input as either a str or pathlib.Path object. Received {}",
            path.get_type().name()?.to_string()
        ))
    }
}

/// Similar to pypath_as_string, except will return a PathBuf
pub fn pypath_as_pathbuf(path: &PyAny) -> PyResult<PathBuf> {
    Ok(PathBuf::from(pypath_as_string(path)?))
}
