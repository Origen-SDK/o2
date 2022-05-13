use pyo3::prelude::*;
use pyo3::class::basic::CompareOp;
use origen_metal::utils::file::FilePermissions as OmFilePermissions;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "file_permissions")?;
    subm.add_function(wrap_pyfunction!(private, subm)?)?;
    subm.add_function(wrap_pyfunction!(group, subm)?)?;
    subm.add_function(wrap_pyfunction!(group_writable, subm)?)?;
    subm.add_function(wrap_pyfunction!(public_with_group_writable, subm)?)?;
    subm.add_function(wrap_pyfunction!(public, subm)?)?;
    subm.add_function(wrap_pyfunction!(world_writable, subm)?)?;
    subm.add_function(wrap_pyfunction!(custom, subm)?)?;
    subm.add_class::<FilePermissions>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
pub fn private() -> PyResult<FilePermissions> {
    Ok(FilePermissions::from_metal(&OmFilePermissions::Private))
}

#[pyfunction]
pub fn group() -> PyResult<FilePermissions> {
    Ok(FilePermissions::from_metal(&OmFilePermissions::Group))
}

#[pyfunction]
pub fn group_writable() -> PyResult<FilePermissions> {
    Ok(FilePermissions::from_metal(&OmFilePermissions::GroupWritable))
}

#[pyfunction]
pub fn public_with_group_writable() -> PyResult<FilePermissions> {
    Ok(FilePermissions::from_metal(&OmFilePermissions::PublicWithGroupWritable))
}

#[pyfunction]
pub fn public() -> PyResult<FilePermissions> {
    Ok(FilePermissions::from_metal(&OmFilePermissions::Public))
}

#[pyfunction]
pub fn world_writable() -> PyResult<FilePermissions> {
    Ok(FilePermissions::from_metal(&OmFilePermissions::WorldWritable))
}

#[pyfunction]
pub fn custom(permissions: &PyAny) -> PyResult<FilePermissions> {
    FilePermissions::new(permissions)
}

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
                "Can not build FilePermissions from type '{}'",
                permissions.get_type().name()?
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

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        match other.extract::<PyRef<Self>>() {
            Ok(other_fp) => {
                let result = self.om_fps == other_fp.om_fps;
                match op {
                    CompareOp::Eq => Ok(result),
                    CompareOp::Ne => Ok(!result),
                    _ => crate::not_implemented_error!("FilePermissions only support equals and not-equals comparisons"),
                }
            },
            Err(_) => Ok(false)
        }
    }
}

#[pyproto]
impl pyo3::class::number::PyNumberProtocol for FilePermissions {
    fn __int__(&self) -> PyResult<u16> {
        self.to_i()
    }
}

impl From<&OmFilePermissions> for FilePermissions {
    fn from(om_fps: &OmFilePermissions) -> Self {
        Self::from_metal(om_fps)
    }
}
