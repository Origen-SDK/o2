pub mod _frontend;

use pyo3::prelude::*;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "website")?;
    m.add_submodule(subm)?;
    Ok(())
}
