use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

#[pymodule]
pub fn application(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyApplication>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyApplication {}

#[pymethods]
impl PyApplication {
    #[new]
    fn new() -> Self {
        PyApplication {}
    }

    #[getter]
    fn version(&self) -> PyResult<String> {
        let v = origen::app().unwrap().version()?.to_string();
        Ok(format!("{}", origen::utility::version::to_pep440(&v)?))
    }
}

/* The Base application is implemented mostly in Python, but has some relevant
   properties usable in Rust.

   Below are some functions to grab data from an assumed origen.application.Base instance
*/

/// Query if the current object is an instance of origen.application.Base
/// Note: this could have several methods overridden. Just check that the aforementioned
/// class is one of the object's ancestors
pub fn is_base_app(query: &PyAny) -> PyResult<bool> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = PyDict::new(py);
    locals.set_item("origen", py.import("origen")?.to_object(py))?;
    locals.set_item("builtins", py.import("builtins")?.to_object(py))?;
    locals.set_item("query", query.to_object(py))?;
    let result = py.eval(
        "builtins.isinstance(query, origen.application.Base)",
        Some(locals),
        None,
    )?;
    Ok(result.extract::<bool>()?)
}

/// Return the name of the given app. Equivalent to `app.name` in Python
/// Returns an error if the given object isn't a `origen.application.Base`
pub fn get_name(app: &PyAny) -> PyResult<String> {
    if is_base_app(app)? {
        Ok(app.getattr("name")?.extract::<String>()?)
    } else {
        crate::runtime_error!("Cannot get name of non-origen.application.Base object")
    }
}

#[allow(dead_code)]
/// Return the root path of the given app. Equivalent to `app.root` in Python
/// Returns an error if the given object isn't a `origen.application.Base`
pub fn get_root(app: &PyAny) -> PyResult<PathBuf> {
    if is_base_app(app)? {
        let p = app.getattr("root")?.extract::<String>()?;
        Ok(PathBuf::from(p))
    } else {
        crate::runtime_error!("Cannot get root of non-origen.application.Base object")
    }
}

#[allow(dead_code)]
/// Return the app path of the given app. Equivalent to `app.app_dir` in Python
/// Returns an error if the given object isn't a `origen.application.Base`
pub fn get_app_dir(app: &PyAny) -> PyResult<PathBuf> {
    if is_base_app(app)? {
        let p = app.getattr("app_dir")?.extract::<String>()?;
        Ok(PathBuf::from(p))
    } else {
        crate::runtime_error!("Cannot get root of non-origen.application.Base object")
    }
}
