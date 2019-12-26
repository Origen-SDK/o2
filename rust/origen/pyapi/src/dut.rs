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
    fn create_sub_block(&self, path: &str, id: &str) -> PyResult<()> {
        Ok(DUT.lock().unwrap().create_sub_block(path, id)?)
    }

    fn create_reg(
        &self,
        path: &str,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
        offset: u32,
        size: Option<u32>,
    ) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        Ok(dut
            .get_mut_model(path)?
            .create_reg(memory_map, address_block, id, offset, size)?)
    }
}
