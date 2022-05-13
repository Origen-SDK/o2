mod file_permissions;
pub mod outcomes;
mod reference_files;
pub mod sessions;
pub mod users;

pub use file_permissions::FilePermissions;
pub use outcomes::Outcome;
pub use outcomes::Outcome as PyOutcome;
use pyo3::prelude::*;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "framework")?;
    reference_files::define(py, m)?;
    outcomes::define(py, m)?;
    sessions::define(py, m)?;
    users::define(py, m)?;
    file_permissions::define(py, m)?;
    parent.add_submodule(m)?;
    Ok(())
}
