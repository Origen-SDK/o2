pub mod icl;

use pyo3::prelude::*;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let module = PyModule::new(py, "ijtag")?;
    icl::define(py, module)?;
    parent.add_submodule(module)?;
    Ok(())
}
