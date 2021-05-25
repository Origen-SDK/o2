pub mod caller;
pub mod ldap;
pub mod location;
#[allow(non_snake_case)]
pub mod mailer;
pub mod metadata;
pub mod revision_control;
pub mod session_store;
pub mod transaction;
pub mod unit_testers;

use ldap::PyInit_ldap;
use location::Location;
use mailer::PyInit_mailer;
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use revision_control::PyInit_revision_control;
use session_store::PyInit_session_store;
use transaction::Transaction;
use unit_testers::PyInit_unit_testers;

use num_bigint::BigUint;
use origen::utility::big_uint_helpers::BigUintHelpers;

#[pymodule]
pub fn utility(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Location>()?;
    m.add_class::<Transaction>()?;
    m.add_wrapped(wrap_pyfunction!(reverse_bits))?;
    m.add_wrapped(wrap_pymodule!(mailer))?;
    m.add_wrapped(wrap_pymodule!(session_store))?;
    m.add_wrapped(wrap_pymodule!(ldap))?;
    m.add_wrapped(wrap_pymodule!(revision_control))?;
    m.add_wrapped(wrap_pymodule!(unit_testers))?;
    Ok(())
}

#[pyfunction]
pub fn reverse_bits(_py: Python, num: BigUint, width: Option<u64>) -> PyResult<BigUint> {
    Ok(num.reverse(width.unwrap_or(num.bits()) as usize)?)
}
