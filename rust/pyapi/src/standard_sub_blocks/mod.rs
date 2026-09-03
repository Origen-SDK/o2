mod arm_debug;

use pyo3::prelude::*;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "standard_sub_blocks")?;
    arm_debug::define(py, subm)?;
    m.add_submodule(subm)?;
    Ok(())
}
