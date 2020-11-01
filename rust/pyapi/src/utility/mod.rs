pub mod location;
pub mod transaction;

use location::Location;
use transaction::{Transaction};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use num_bigint::BigUint;
use origen::utility::big_uint_helpers::BigUintHelpers;

#[pymodule]
pub fn utility(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Location>()?;
    m.add_class::<Transaction>()?;
    m.add_wrapped(wrap_pyfunction!(reverse_bits))?;
    Ok(())
}

#[pyfunction]
pub fn reverse_bits(_py: Python, num: BigUint, width: Option<u64>) -> PyResult<BigUint> {
    Ok(num.reverse(width.unwrap_or(num.bits()) as usize)?)
}