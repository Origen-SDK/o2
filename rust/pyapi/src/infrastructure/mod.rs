pub mod pyproject;

use pyo3::prelude::*;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "infrastructure")?;
    pyproject::define(py, subm)?;
    m.add_submodule(subm)?;
    Ok(())
}
