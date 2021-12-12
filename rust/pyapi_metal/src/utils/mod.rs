mod differ;
pub mod revision_control;
mod ldap;

use pyo3::prelude::*;
use origen_metal::utils::file::FilePermissions as OmFilePermissions;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "utils")?;
    revision_control::define(py, subm)?;
    ldap::define(py, subm)?;
    differ::define(py, subm)?;
    parent.add_submodule(subm)?;
    Ok(())
}

// TESTS_NEEDED
// TODO move to framework
#[pyclass]
pub struct FilePermissions {
    om_fps: OmFilePermissions,
}

impl FilePermissions {
    pub fn from_metal(fp: &OmFilePermissions) -> Self {
        Self {
            om_fps: fp.clone()
        }
    }

    pub fn to_metal(fp: &PyAny) -> PyResult<OmFilePermissions> {
        if let Ok(s) = fp.extract::<&str>() {
            Ok(OmFilePermissions::from_str(s)?)
        } else if let Ok(i) = fp.extract::<u16>() {
            Ok(OmFilePermissions::from_i(i)?)
        } else if let Ok(slf) = fp.extract::<PyRef<Self>>() {
            Ok(slf.om_fps.clone())
        } else {
            return crate::runtime_error!(format!(
                "Could not derive file permissions from input of type '{}'",
                fp.get_type().to_string()
            ));
        }
    }

    pub fn to_metal_optional(file_permissions: Option<&PyAny>) -> PyResult<Option<OmFilePermissions>> {
        if let Some(fp) = file_permissions {
            Ok(Some(Self::to_metal(fp)?))
        } else {
            Ok(None)
        }
    }
}

#[pymethods]
impl FilePermissions {
    #[new]
    fn new(permissions: &PyAny) -> PyResult<Self> {
        if let Ok(p) = permissions.extract::<&str>() {
            Ok(Self {
                om_fps: OmFilePermissions::from_str(&p)?,
            })
        } else if let Ok(p) = permissions.extract::<u16>() {
            Ok(Self {
                om_fps: OmFilePermissions::from_i(p)?,
            })
        } else {
            crate::type_error!(format!(
                "Can not build FilePermissions from type {}",
                permissions.get_type()
            ))
        }
    }

    #[getter]
    fn to_i(&self) -> PyResult<u16> {
        Ok(self.om_fps.to_i())
    }

    #[getter]
    fn to_s(&self) -> PyResult<String> {
        Ok(self.om_fps.to_str().to_string())
    }
}

#[pyproto]
impl pyo3::class::basic::PyObjectProtocol for FilePermissions {
    fn __str__(&self) -> PyResult<String> {
        self.to_s()
    }
}

#[pyproto]
impl pyo3::class::number::PyNumberProtocol for FilePermissions {
    fn __int__(&self) -> PyResult<u16> {
        self.to_i()
    }
}
