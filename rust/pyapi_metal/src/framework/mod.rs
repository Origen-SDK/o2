pub mod outcomes;
pub mod sessions;
mod reference_files;
mod file_permissions;
pub mod users;

use pyo3::prelude::*;
pub use outcomes::Outcome;
pub use outcomes::Outcome as PyOutcome;
pub use file_permissions::FilePermissions;

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
