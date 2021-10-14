pub mod git;

use pyo3::prelude::*;

pub use git::Git;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "supported")?;
    subm.add_class::<Git>()?;
    m.add_submodule(subm)?;
    Ok(())
}
