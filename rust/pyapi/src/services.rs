pub mod jtag;
pub mod simple;
pub mod swd;
// pub mod ahb;

use pyo3::prelude::*;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "services")?;
    subm.add_class::<jtag::JTAG>()?;
    subm.add_class::<swd::SWD>()?;
    subm.add_class::<simple::Simple>()?;
    m.add_submodule(subm)?;
    Ok(())
}
