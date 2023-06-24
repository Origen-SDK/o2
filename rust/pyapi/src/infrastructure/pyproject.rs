use origen::STATUS;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use origen::core::status::DependencySrc;

pub fn populate_status(py: Python, status: &PyDict) -> PyResult<()> {
    if let Some(d) = STATUS.dependency_src().as_ref() {
        if let Some(pyproject) = d.src_file() {
            status.set_item("pyproject", pyapi_metal::pypath!(py, pyproject.display()))?;
        } else {
            status.set_item("pyproject", py.None())?;
        }
        status.set_item("invocation", PyProjectSrc::from(d).to_py(py)?)?;
    } else {
        status.set_item("pyproject", py.None())?;
        status.set_item("invocation", py.None())?;
    }
    Ok(())
}

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "pyproject")?;
    subm.add_class::<PyProjectSrc>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass]
pub enum PyProjectSrc {
    App,
    Workspace,
    UserGlobal,
    Global,
    None,
}

impl PyProjectSrc {
    pub fn to_py(self, py: Python) -> PyResult<Py<Self>> {
        Py::new(py, self)
    }
}

impl From<&DependencySrc> for PyProjectSrc {
    fn from(src: &DependencySrc) -> Self {
        match src {
            DependencySrc::App(_) => Self::App,
            DependencySrc::Workspace(_) => Self::Workspace,
            DependencySrc::UserGlobal(_) => Self::UserGlobal,
            DependencySrc::Global(_) => Self::Global,
            DependencySrc::None => Self::None,
        }
    }
}