pub mod outcomes;
mod reference_files;

use pyo3::prelude::*;

pub use outcomes::Outcome;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "framework")?;
    reference_files::define(py, m)?;
    outcomes::define(py, m)?;
    parent.add_submodule(m)?;
    Ok(())
}
