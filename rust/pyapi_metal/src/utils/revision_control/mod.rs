pub mod _frontend;
use crate::framework::Outcome;

use pyo3::prelude::*;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "revision_control")?;
    subm.add_class::<Base>()?;
    // m.add_class::<Status>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass(subclass)]
pub struct Base {}

#[pymethods]
impl Base {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(Self {})
    }
}

#[cfg(debug_assertions)]
pub(crate) fn define_tests(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rc_init_from_metal, m)?)?;
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
