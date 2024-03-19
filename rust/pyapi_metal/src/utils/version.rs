use origen_metal::utils::version::Version as OMVersion;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::path::PathBuf;

#[pyclass]
pub struct Version {
    _origen_version: OMVersion,
}

impl Version {
    fn coerce_error_type(err: Result<OMVersion, origen_metal::Error>) -> PyResult<OMVersion> {
        match err {
            Ok(v) => Ok(v),
            Err(e) => if e.msg.starts_with("unexpected character '") {
                value_error!(e.msg)
            } else {
                runtime_error!(e.msg)
            }
        }
    }
}

#[pymethods]
impl Version {
    #[new]
    fn new(ver: &str) -> PyResult<Self> {
        Ok(Self {
            _origen_version: {
                Self::coerce_error_type(OMVersion::new_pep440(ver))?
            }
        })
    }

    #[classmethod]
    fn from_pyproject(_cls: &PyType, pyproject: PathBuf) -> PyResult<Self> {
        Ok( Self {
            _origen_version: OMVersion::from_pyproject_with_toml_handle(pyproject)?.orig_version().clone()
        })
    }

    #[classmethod]
    fn from_cargo(_cls: &PyType, cargo_toml: PathBuf) -> PyResult<Self> {
        Ok( Self {
            _origen_version: OMVersion::from_cargo_with_toml_handle(cargo_toml)?.orig_version().clone()
        })
    }

    #[classmethod]
    fn pep440(_cls: &PyType, ver: &str) -> PyResult<Self> {
        Ok( Self {
            _origen_version: {
                Self::coerce_error_type(OMVersion::new_pep440(ver))?
            }
        })
    }

    #[classmethod]
    fn semver(_cls: &PyType, ver: &str) -> PyResult<Self> {
        Ok( Self {
            _origen_version: {
                Self::coerce_error_type(OMVersion::new_semver(ver))?
            }
        })
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self._origen_version.to_string())
    }

    fn convert_to_pep440(&mut self) -> PyResult<bool> {
        Ok(self._origen_version.convert_to_pep440())
    }

    fn convert_to_semver(&mut self) -> PyResult<bool> {
        Ok(self._origen_version.convert_to_semver())
    }

    #[getter]
    fn is_pep440(&self) -> PyResult<bool> {
        Ok(self._origen_version.is_pep440())
    }

    #[getter]
    fn is_semver(&self) -> PyResult<bool> {
        Ok(self._origen_version.is_semver())
    }

    #[getter]
    fn pep440_str(&self) -> PyResult<String> {
        Ok(self._origen_version.to_pep440_string())
    }

    #[getter]
    fn semver_str(&self) -> PyResult<String> {
        Ok(self._origen_version.to_semver_string())
    }
}
