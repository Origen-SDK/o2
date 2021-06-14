pub mod _frontend;

use pyo3::prelude::*;

#[pymodule]
pub fn website(_py: Python, _m: &PyModule) -> PyResult<()> {
    Ok(())
}
