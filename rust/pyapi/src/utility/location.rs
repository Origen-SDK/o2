use origen::utility::location::Location as OrigenLoc;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use crate::pypath;

/// Helper class to store and query "location" types, which could be local paths, git URLs, as either SSH or HTTPS, or something else.
/// The location need not be valid to create an instance of this class.
#[pyclass]
pub struct Location {
    pub location: OrigenLoc,
}

#[pymethods]
impl Location {

    /// Returns the location's target, regardless of what it may be.
    ///
    /// Returns:
    ///     str: Location's target as a ``str``
    #[getter]
    fn target(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(self.location.location.clone())
    }

    /// If the location points to a web URL, returns that URL.
    ///
    /// Returns:
    ///     str: URL as a ``str``, if the Location points to a URL
    ///     None: Otherwise
    #[getter]
    fn url(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.url() {
            Some(url) => url.to_object(py),
            None => py.None()
        })
    }

    /// If the location points to a Git repo, returns that repo URL.
    ///
    /// Notes
    /// -----
    ///  * Returns the Git repo for either HTTPS or SSH paths.
    ///
    /// Returns:
    ///     str: Repo URL as a ``str``, if the Location points to a Git repo
    ///     None: Otherwise
    #[getter]
    fn git(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.git() {
            Some(git) => git.to_object(py),
            None => py.None()
        })
    }

    /// If the location points to a Git repo via HTTPS, returns that repo URL.
    ///
    /// Returns:
    ///     str: Repo URL as a ``str``, if the Location points to a Git repo via HTTPS
    ///     None: Otherwise
    #[getter]
    fn git_https(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.git_https() {
            Some(git) => git.to_object(py),
            None => py.None()
        })
    }

    /// If the location points to a Git repo via SSH, returns that repo URL.
    ///
    /// Returns:
    ///     str: Repo URL as a ``str``, if the Location points to a Git repo via SSH
    ///     None: Otherwise
    #[getter]
    fn git_ssh(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match self.location.git_ssh() {
            Some(git) => git.to_object(py),
            None => py.None()
        })
    }


    /// If the location points to an OS path, return that path.
    ///
    /// Returns:
    ///     pathlib.Path: Repo URL as a |pathlib.Path| object, if the Location points to a local path.
    ///     None: Otherwise
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