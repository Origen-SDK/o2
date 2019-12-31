use pyo3::prelude::*;
use origen::DUT;
use pyo3::{wrap_pymodule};
use crate::pins::PyInit_pins;

/// Implements the module _origen.dut in Python which exposes all
/// DUT-related APIs
#[pymodule]
pub fn dut(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDUT>()?;
    m.add_wrapped(wrap_pymodule!(pins))?;

    Ok(())
}

#[pyclass]
#[derive(Debug)]
pub struct PyDUT {}

#[pymethods]
impl PyDUT {
    #[new]
    /// Instantiating a new instance of PyDUT means re-loading the target
    fn new(obj: &PyRawObject, name: &str) {
        DUT.lock().unwrap().change(name);
        obj.init({ PyDUT {} });
    }

    /// Creates a new model at the given path
    fn create_model(&self, parent_id: Option<usize>, name: &str) -> PyResult<usize> {
        Ok(DUT.lock().unwrap().create_model(parent_id, name)?)
    }

    fn create_reg(
        &self,
        address_block_id: usize,
        name: &str,
        offset: u32,
        size: Option<u32>,
    ) -> PyResult<usize> {
        Ok(DUT
            .lock()
            .unwrap()
            .create_reg(address_block_id, name, offset, size)?)
    }
}
