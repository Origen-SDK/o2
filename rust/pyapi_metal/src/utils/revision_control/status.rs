use crate::_helpers::to_py_paths;
use crate::pypath;
use origen_metal::utils::revision_control::Status as OrigenStatus;
use pyo3::prelude::*;

#[pyclass(subclass)]
pub struct Status {
    pub stat: OrigenStatus,
}

#[pymethods]
impl Status {
    #[getter]
    fn added(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut retn: Vec<PyObject> = vec![];
        for added in self.stat.added.iter() {
            retn.push(pypath!(py, added.display()));
        }
        Ok(retn)
    }

    #[getter]
    fn removed(&self, py: Python) -> PyResult<Vec<PyObject>> {
        to_py_paths(py, &self.stat.removed.iter().map(|p| p.display()).collect())
    }

    #[getter]
    fn conflicted(&self, py: Python) -> PyResult<Vec<PyObject>> {
        to_py_paths(py, &self.stat.conflicted.iter().map(|p| p.display()).collect())
    }

    #[getter]
    fn changed(&self, py: Python) -> PyResult<Vec<PyObject>> {
        to_py_paths(py, &self.stat.changed.iter().map(|p| p.display()).collect())
    }

    #[getter]
    fn revision(&self) -> PyResult<String> {
        Ok(self.stat.revision.clone())
    }

    #[getter]
    fn is_modified(&self) -> PyResult<bool> {
        Ok(self.stat.is_modified())
    }

    fn summarize(&self) -> PyResult<()> {
        Ok(self.stat.summarize())
    }
}

impl Status {
    pub fn stat(&self) -> &OrigenStatus {
        &self.stat
    }

    pub fn from_origen(stat: OrigenStatus) -> Self {
        Self { stat }
    }

    pub fn into_origen(&self) -> PyResult<OrigenStatus> {
        Ok(self.stat.clone())
    }
}
