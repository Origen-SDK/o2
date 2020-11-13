pub mod location;

use location::Location;
use pyo3::prelude::*;

#[pymodule]
pub fn utility(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Location>()?;
    Ok(())
}
