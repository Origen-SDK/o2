pub mod jtag;
pub mod simple;
pub mod swd;
// pub mod ahb;

use pyo3::prelude::*;

#[pymodule]
/// Implements the module _origen.services in Python
pub fn services(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<jtag::JTAG>()?;
    m.add_class::<swd::SWD>()?;
    m.add_class::<simple::Simple>()?;
    // m.add_class::<ahb::AHB>()?;

    Ok(())
}
