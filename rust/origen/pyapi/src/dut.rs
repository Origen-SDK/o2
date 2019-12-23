use pyo3::exceptions;
use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;
use origen::core::model::registers::AccessType;
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
    fn new(obj: &PyRawObject, name: &str) {
        DUT.lock().unwrap().change(name);
        obj.init({ PyDUT {} });
    }

    /// Creates a new model at the given path
    fn create_model(&self, parent_id: Option<usize>, name: &str) -> PyResult<usize> {
        Ok(DUT.lock().unwrap().create_model(parent_id, name)?)
    }

    fn create_memory_map(
        &self,
        model_id: usize,
        name: &str,
        address_unit_bits: Option<u32>,
    ) -> PyResult<usize> {
        Ok(DUT
            .lock()
            .unwrap()
            .create_memory_map(model_id, name, address_unit_bits)?)
    }

    fn create_address_block(
        &self,
        memory_map_id: usize,
        name: &str,
        base_address: Option<u64>,
        range: Option<u64>,
        width: Option<u64>,
        access: Option<&str>,
    ) -> PyResult<usize> {
        let acc: AccessType = match access {
            Some(x) => match x.parse() {
                Ok(y) => y,
                Err(msg) => return Err(exceptions::OSError::py_err(msg)),
            },
            None => AccessType::ReadWrite,
        };

        Ok(DUT.lock().unwrap().create_address_block(
            memory_map_id,
            name,
            base_address,
            range,
            width,
            Some(acc),
        )?)
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
