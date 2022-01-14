pub mod outcomes;
pub mod sessions;
mod reference_files;
mod file_permissions;

use pyo3::prelude::*;
pub use outcomes::Outcome;
pub use file_permissions::FilePermissions;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "framework")?;
    reference_files::define(py, m)?;
    outcomes::define(py, m)?;
    sessions::define(py, m)?;
    file_permissions::define(py, m)?;
    parent.add_submodule(m)?;
    Ok(())
}
