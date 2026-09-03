mod mailer;
mod maillist;
mod maillists;

pub use mailer::{Mailer, OM_MAILER_CLASS_QP};
pub use maillist::Maillist;
pub use maillists::{Maillists, OM_MAILLISTS_CLASS_QP};
use pyo3::prelude::*;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "mailer")?;
    subm.add_class::<Mailer>()?;
    subm.add_class::<Maillist>()?;
    subm.add_class::<Maillists>()?;
    m.add_submodule(subm)?;
    Ok(())
}
