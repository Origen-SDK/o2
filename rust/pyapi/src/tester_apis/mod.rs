//! This module implements classes which expose ATE-specific APIs for use in both pattern and program generation.
mod igxl;
mod v93k;

pub use igxl::IGXL;
pub use v93k::V93K;

use pyo3::prelude::*;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "tester_apis")?;
    subm.add_class::<v93k::V93K>()?;
    subm.add_class::<igxl::IGXL>()?;
    m.add_submodule(subm)?;
    Ok(())
}
