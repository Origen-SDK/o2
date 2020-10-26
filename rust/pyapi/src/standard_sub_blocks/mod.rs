mod arm_debug;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use arm_debug::PyInit_arm_debug;

#[pymodule]
/// Implements the module _origen.standard_sub_blocks in Python
pub fn standard_sub_blocks(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(arm_debug))?;

    Ok(())
}
