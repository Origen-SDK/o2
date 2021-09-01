mod reference_files;

use crate::py_submodule;
use pyo3::prelude::*;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    py_submodule(py, parent, "origen_metal.framework", |m| {
        reference_files::define(py, m)?;
        Ok(())
    })
}
