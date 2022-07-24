mod mailer;
mod maillist;
mod maillists;

use pyo3::prelude::*;
pub use mailer::{Mailer, OM_MAILER_CLASS_QP};
pub use maillist::Maillist;
pub use maillists::{Maillists, OM_MAILLISTS_CLASS_QP};

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "mailer")?;
    subm.add_class::<Mailer>()?;
    subm.add_class::<Maillist>()?;
    subm.add_class::<Maillists>()?;
    m.add_submodule(subm)?;
    Ok(())
}
