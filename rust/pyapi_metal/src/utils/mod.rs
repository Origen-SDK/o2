mod differ;
pub mod ldap;
pub mod revision_control;
pub mod mailer;
pub mod version;

use pyo3::prelude::*;
use version::Version;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "utils")?;
    revision_control::define(py, subm)?;
    ldap::define(py, subm)?;
    differ::define(py, subm)?;
    mailer::define(py, subm)?;
    subm.add_class::<Version>()?;
    parent.add_submodule(subm)?;
    Ok(())
}
