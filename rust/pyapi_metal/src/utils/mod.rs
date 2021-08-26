mod differ;

use crate::py_submodule;
use pyo3::prelude::*;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    py_submodule(py, parent, "origen_metal.utils", |m| {
        differ::define(py, m)?;
        Ok(())
    })
}
