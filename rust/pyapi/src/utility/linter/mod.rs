pub mod _frontend;

use pyo3::prelude::*;

#[pymodule]
pub fn linter(_py: Python, _m: &PyModule) -> PyResult<()> {
    Ok(())
}