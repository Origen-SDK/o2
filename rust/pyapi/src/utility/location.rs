use origen::utility::location::Location as OrigenLoc;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use crate::pypath;

#[pyclass]
pub struct Location {
    pub location: OrigenLoc,
}

#[pymethods]
impl Location {

    #[getter]
    fn url(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.url() {
            Some(url) => url.to_object(py),
            None => py.None()
        })
    }

    #[getter]
    fn git(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.git() {
            Some(git) => git.to_object(py),
            None => py.None()
        })
    }

    #[getter]
    fn git_https(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.git_https() {
            Some(git) => git.to_object(py),
            None => py.None()
        })
    }

    #[getter]
    fn git_ssh(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.git_ssh() {
            Some(git) => git.to_object(py),
            None => py.None()
        })
    }

    #[getter]
    fn path(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.path() {
            Some(_path) => pypath!(py, self.location.location),
            None => py.None()
        })
    }

}