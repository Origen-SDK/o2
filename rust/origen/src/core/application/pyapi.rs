use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use super::target::CURRENT_TARGET;

#[pymodule]
/// Implements the module _origen.app in Python
pub fn app(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(current_target))?;

    Ok(())
}

#[pyfunction]
/// Returns a dict containing information about the currently selected
/// target/environment
fn current_target(py: Python) -> PyResult<PyObject> {
    Ok(CURRENT_TARGET.to_py_dict(&py).into())
}
