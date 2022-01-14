pub mod typed_value;
pub mod pickle;

#[macro_use]
pub mod errors;

use crate::{pypath, runtime_error};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType, PyTuple};
use std::path::PathBuf;
use indexmap::IndexMap;
use pyo3::conversion::ToPyObject;

// Converts a PyAny (as string or a type) into a PyType
pub fn pytype_from_pyany<'p>(py: Python<'p>, t: &'p PyAny) -> PyResult<&'p PyType> {
    if let Ok(pyt) = t.extract::<&'p PyType>() {
        Ok(pyt)
    } else if let Ok(pyt) = t.extract::<&str>() {
        pytype_from_str(py, pyt)
    } else {
        return runtime_error!(format!(
            "Cannot extract python class from input of class '{}'",
            t.get_type()
        ))
    }
}

pub fn pytype_from_str<'p>(py: Python<'p>, class: impl std::fmt::Display) -> PyResult<&'p PyType> {
    let cls = class.to_string();
    let split = cls.splitn(2, ".").collect::<Vec<&str>>();
    let locals = PyDict::new(py);
    let cls_path: String;
    if split.len() > 1 {
        locals.set_item("mod", py.import(split[0])?.to_object(py))?;
        cls_path = format!("mod.{}", split[1]);
    } else {
        cls_path = split[0].to_string();
    }

    let t = py.eval(
        &cls_path,
        Some(locals),
        None,
    )?;
    t.extract::<&PyType>()
}

pub fn new_py_obj<'p>(py: Python<'p>, class: &'p PyType, args: Option<impl IntoPy<Py<PyTuple>>>, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
    let t;
    class.call(
        if let Some(a) = args {
            t = a.into_py(py);
            t.as_ref(py)
        } else {
            PyTuple::empty(py)
        },
        kwargs
    )
}

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

pub fn indexmap_to_pydict<'p>(
    py: Python<'p>,
    hmap: &IndexMap<impl ToPyObject, impl ToPyObject>,
) -> PyResult<Py<PyDict>> {
    let py_config = PyDict::new(py);
    for (k, v) in hmap.iter() {
        py_config.set_item(k, v)?;
    }
    Ok(py_config.into())
}
