use pyo3::prelude::*;
use origen::utility::version::Version as OrigenVersion;

#[pyclass]
pub struct Version {
    _origen_version: OrigenVersion
}

#[pymethods]
impl Version {
    #[new]
    fn new(ver: &str) -> PyResult<Self> {
        Ok(Self { _origen_version: OrigenVersion::new_pep440(ver)? })
    }
}