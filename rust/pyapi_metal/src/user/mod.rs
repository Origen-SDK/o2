mod current;

use pyo3::prelude::*;
use pyo3::py_run;

#[pyfunction]
pub fn hi() -> PyResult<String> {
    Ok("Hi, from the origen_metal.user".to_string())
}

pub(crate) fn register(py: Python, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "user")?;
    m.add_function(wrap_pyfunction!(hi, m)?)?;
    // py_run! is quick-and-dirty; should be replaced by PyO3 API calls in actual code
    py_run!(py, m, "import sys; sys.modules['origen_metal.user'] = m");
    parent.add_submodule(m)?;
    current::register(py, m)?;
    Ok(())
}
