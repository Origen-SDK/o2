mod git;
pub mod _frontend;

use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use pyo3::types::PyDict;
use git::{PyInit_git, PY_GIT_MOD_PATH};

use origen::STATUS;
use origen::revision_control::SupportedSystems;
use origen::revision_control::Status as OrigenStatus;

use crate::{runtime_error, pypath};
use crate::_helpers::to_py_paths;

#[pymodule]
fn revision_control(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(app_rc))?;
    m.add_wrapped(wrap_pymodule!(git))?;
    m.add_class::<Base>()?;
    m.add_class::<Status>()?;
    Ok(())
}

/// Creates a RC driver from the application's ``config.toml``
#[pyfunction]
fn app_rc() -> PyResult<Option<PyObject>> {
    // Raise an error if we aren't in an application instance
    let app;
    match &STATUS.app {
        Some(a) => app = a,
        None => return runtime_error!("Cannot retrieve the application's revision control: no application found!")
    }

    let config = app.config();
    if let Some(rc_config) = &config.revision_control {
        // If we're calling this function, we're at least in a python shell.
        // Search for python classes first, falling back on a straight Rust-implemented
        // one if none is found.
        // (Note that a Python one, such as the default Git, is just a wrapper around
        //  the Rust-implemented Git driver)
        if let Some(c) = rc_config.get("system") {
            // Get the module and try to import it
            let split = c.rsplitn(1, ".");
            if split.count() == 2 {
                // Have a class (hopefully) of the form 'a.b.Class'
                todo!();
            }

            // fall back to Rust's lookup parameters
            let gil = Python::acquire_gil();
            let py = gil.python();
            let locals = PyDict::new(py);
            let py_rc_config = PyDict::new(py);
            for (k, v) in rc_config.iter() {
                py_rc_config.set_item(k, v)?;
            }
            py_rc_config.set_item("local", app.root.display().to_string())?;
            locals.set_item("py_rc_config", py_rc_config)?;
            locals.set_item("origen_git", py.import(PY_GIT_MOD_PATH)?.to_object(py))?;

            match SupportedSystems::from_str(c)? {
                SupportedSystems::Git => {
                    // Use the Git driver through Python
                    let pygit = py.eval(
                        "origen_git.Git(py_rc_config)",
                        Some(locals),
                        None
                    )?;
                    Ok(Some(pygit.to_object(py)))
                },
                SupportedSystems::Designsync => {
                    // Use the DS driver through Python
                    // runtime_error!("DesignSync not support yet!")
                    todo!();
                }
            }
        } else {
            // Invalid config
            return runtime_error!("Could not discern RC system from app config");
        }
    } else {
        // Return None if no revision_control parameter is given
        Ok(None)
    }
}

#[pyclass(subclass)]
pub struct Base {}

#[pymethods]
impl Base {}

#[pyclass(subclass)]
pub struct Status {
    stat: OrigenStatus
}

#[pymethods]
impl Status {
    #[getter]
    fn added(&self) -> PyResult<Vec<PyObject>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut retn: Vec<PyObject> = vec![];
        for added in self.stat.added.iter() {
            retn.push(pypath!(py, added.display()));
        }
        Ok(retn)
    }

    #[getter]
    fn removed(&self) -> PyResult<Vec<PyObject>> {
        to_py_paths(&self.stat.removed.iter().map( |p| p.display() ).collect())
    }

    #[getter]
    fn conflicted(&self) -> PyResult<Vec<PyObject>> {
        to_py_paths(&self.stat.conflicted.iter().map( |p| p.display() ).collect())
    }

    #[getter]
    fn changed(&self) -> PyResult<Vec<PyObject>> {
        to_py_paths(&self.stat.changed.iter().map( |p| p.display() ).collect())
    }

    #[getter]
    fn revision(&self) -> PyResult<String> {
        Ok(self.stat.revision.clone())
    }

    #[getter]
    fn is_modified(&self) -> PyResult<bool> {
        Ok(self.stat.is_modified())
    }
}

impl Status {
    pub fn stat(&self) -> &OrigenStatus {
        &self.stat
    }
}