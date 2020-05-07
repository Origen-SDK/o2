pub mod location;

use pyo3::prelude::*;
use location::Location;

#[pymodule]
pub fn utility(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<Location>()?;
  Ok(())
}