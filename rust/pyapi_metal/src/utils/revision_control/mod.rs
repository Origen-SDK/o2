pub mod _frontend;
pub mod status;
pub mod supported;

use crate::framework::Outcome;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use status::Status;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "revision_control")?;
    subm.add_class::<Base>()?;
    subm.add_class::<Status>()?;
    supported::define(py, subm)?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass(subclass)]
pub struct Base {}

#[pymethods]
impl Base {
    #[new]
    #[args(_args = "*", _config = "**")]
    fn new(_args: &PyTuple, _config: Option<&PyDict>) -> PyResult<Self> {
        Ok(Self {})
    }
}

#[cfg(debug_assertions)]
pub(crate) fn define_tests(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rc_init_from_metal, m)?)?;
    m.add_function(wrap_pyfunction!(python_git_mod_path, m)?)?;
    m.add_function(wrap_pyfunction!(ping_rc, m)?)?;
    m.add_function(wrap_pyfunction!(rc_status, m)?)?;
    Ok(())
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn rc_init_from_metal(_py: Python) -> PyResult<Outcome> {
    Ok(origen_metal::frontend::with_frontend(|f| {
        let rc = f.require_rc()?;
        Ok(Outcome::from_origen(rc.init()?))
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn python_git_mod_path(_py: Python) -> PyResult<&str> {
    Ok(supported::git::PY_GIT_MOD_PATH)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn ping_rc(_py: Python) -> PyResult<String> {
    Ok(origen_metal::frontend::with_frontend(|f| {
        let rc = f.require_rc()?;
        Ok(rc.system()?)
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn rc_status(_py: Python) -> PyResult<Status> {
    Ok(origen_metal::frontend::with_frontend(|f| {
        let rc = f.require_rc()?;
        Ok(Status::from_origen(rc.status()?))
    })?)
}
