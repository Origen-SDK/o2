//! This module implements classes which expose ATE-specific APIs for use in both pattern and program generation.
mod ultraflex;
mod v93k;

use pyo3::prelude::*;

#[pymodule]
fn tester_apis(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<v93k::V93K>()?;
    m.add_class::<ultraflex::ULTRAFLEX>()?;
    Ok(())
}
