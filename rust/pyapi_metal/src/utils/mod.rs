mod differ;
mod ldap;
pub mod revision_control;

use pyo3::prelude::*;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "utils")?;
    revision_control::define(py, subm)?;
    ldap::define(py, subm)?;
    differ::define(py, subm)?;
    parent.add_submodule(subm)?;
    Ok(())
}
