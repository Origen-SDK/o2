pub mod pickle;
pub mod typed_value;
pub mod contextlib;
use crate::cfg_if;

#[macro_use]
pub mod config;

#[macro_use]
pub mod errors;

use crate::{pypath, runtime_error};
use indexmap::IndexMap;
use pyo3::conversion::ToPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use std::path::PathBuf;

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
        ));
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

    let t = py.eval(&cls_path, Some(locals), None)?;
    t.extract::<&PyType>()
}

pub fn new_py_obj<'p>(
    py: Python<'p>,
    class: &'p PyType,
    args: Option<impl IntoPy<Py<PyTuple>>>,
    kwargs: Option<&PyDict>,
) -> PyResult<&'p PyAny> {
    let t;
    class.call(
        if let Some(a) = args {
            t = a.into_py(py);
            t.as_ref(py)
        } else {
            PyTuple::empty(py)
        },
        kwargs,
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

// TODO replace with generic?
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

pub fn map_to_pydict<'p, K, V>(
    py: Python<'p>,
    map: &'p mut dyn Iterator<Item = (K, V)>,
) -> PyResult<Py<PyDict>>
where
    K: 'p + ToPyObject,
    V: 'p + ToPyObject,
{
    let pydict = PyDict::new(py);
    for (k, v) in map.into_iter() {
        pydict.set_item(k, v)?;
    }
    Ok(pydict.into())
}

pub fn with_new_pydict<F, T>(py: Python, mut f: F) -> PyResult<Py<PyDict>>
where
    F: FnMut(&PyDict) -> PyResult<T>,
{
    let pydict = PyDict::new(py);
    f(pydict)?;
    Ok(pydict.into())
}

// TEST_NEEDED
pub fn get_qualified_attr(s: &str) -> PyResult<Py<PyAny>> {
    Python::with_gil( |py| {
        let mut split = s.split(".");
        let mut current: PyObject;

        let starting = split.next().unwrap();
        let remaining = split.collect::<Vec<&str>>();
        let mut current_str = starting.to_string();

        if remaining.len() == 0 {
            // Assume "builtins if no module is given"
            let builtins = PyModule::import(py, "builtins")?.to_object(py);
            return builtins.getattr(py, starting);
        } else {
            current = PyModule::import(py, starting)?.to_object(py);
        }

        for component in remaining {
            current_str.push_str(".");
            current_str.push_str(component);
            match PyModule::import(py, &current_str) {
                Ok(py_mod) => {
                    current = py_mod.to_object(py);
                },
                Err(e) => {
                    match current.getattr(py, component) {
                        Ok(attr) => current = attr.to_object(py),
                        Err(e2) => {
                            return runtime_error!(format!(
                                "Failed to get qualified attribute '{}': \n\n{} \n\n{}",
                                s,
                                e.to_string(),
                                e2.to_string(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(current)
    })
}

cfg_if! {
    if #[cfg(debug_assertions)] {
        #[cfg(debug_assertions)]
        use pyo3::types::PyModule;
        pub(crate) fn define_tests(py: Python, test_mod: &PyModule) -> PyResult<()> {
            let subm = PyModule::new(py, "_helpers")?;
            contextlib::define_tests(py, subm)?;
            test_mod.add_submodule(subm)?;
            Ok(())
        }
    }
}