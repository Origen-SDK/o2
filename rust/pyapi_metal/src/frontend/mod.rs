static MOD_PATH: &'static str = "origen_metal._origen_metal";
static MOD: &'static str = "frontend";
static PY_FRONTEND: &'static str = "__py_frontend__";

#[macro_export]
macro_rules! frontend_mod {
    ($py:expr) => {{
        $py.import(crate::frontend::MOD_PATH)?
            .getattr(crate::frontend::MOD)?
            .extract::<&pyo3::types::PyModule>()?
    }};
}

mod _frontend;
mod py_frontend;

use pyo3::prelude::*;

pub use _frontend::Frontend;
pub use py_frontend::PyFrontend;
pub(crate) use py_frontend::{with_py_frontend, with_required_rc};

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let fm = PyModule::new(py, "frontend")?;
    fm.add_class::<PyFrontend>()?;
    fm.add_function(wrap_pyfunction!(initialize, fm)?)?;
    fm.add_function(wrap_pyfunction!(frontend, fm)?)?;
    fm.add_function(wrap_pyfunction!(reset, fm)?)?;
    m.add_submodule(fm)?;
    Ok(())
}

pub(crate) fn with_frontend_mod<F, T>(mut func: F) -> PyResult<T>
where
    F: FnMut(Python, &PyModule) -> PyResult<T>,
{
    Python::with_gil(|py| {
        let fm = frontend_mod!(py);
        func(py, fm)
    })
}

#[pyfunction]
pub(crate) fn frontend(py: Python) -> PyResult<Option<PyRef<PyFrontend>>> {
    if origen_metal::frontend::frontend_set()? {
        let m = frontend_mod!(py);
        Ok(Some(
            m.getattr(PY_FRONTEND)?.extract::<PyRef<PyFrontend>>()?,
        ))
    } else {
        Ok(None)
    }
}

#[pyfunction]
pub(crate) fn initialize(_py: Python) -> PyResult<bool> {
    if origen_metal::frontend::frontend_set()? {
        Ok(false)
    } else {
        origen_metal::frontend::set_frontend(Box::new(Frontend::new()?))?;
        Ok(true)
    }
}

#[pyfunction]
pub(crate) fn reset(_py: Python) -> PyResult<()> {
    origen_metal::frontend::reset()?;
    with_frontend_mod(|py, m| m.setattr(PY_FRONTEND, py.None()))
}
