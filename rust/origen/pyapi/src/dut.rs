use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;
use origen::DUT;

/// Implements the module _origen.dut in Python which exposes all
/// DUT-related APIs
#[pymodule]
pub fn dut(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDUT>()?;

    Ok(())
}

#[pyclass]
#[derive(Debug)]
pub struct PyDUT {}

#[pymethods]
impl PyDUT {
    #[new]
    /// Instantiating a new instance of PyDUT means re-loading the target
    fn new(obj: &PyRawObject, id: &str) {
        DUT.lock().unwrap().change(id);
        obj.init({ PyDUT {} });
    }

    /// Creates a new model at the given path
    fn create_sub_block(&self, parent_id: usize, name: &str) -> PyResult<usize> {
        Ok(DUT.lock().unwrap().create_sub_block(parent_id, name)?)
    }

    fn create_reg(
        &self,
        address_block_id: usize,
        name: &str,
        offset: u32,
        size: Option<u32>,
    ) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        Ok(DUT
            .lock()
            .unwrap()
            .create_reg(address_block_id, name, offset, size)?)
    }
}
