use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;
use crate::register::BitCollection;
use origen::DUT;
use pyo3::exceptions;

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
        DUT.lock()
            .unwrap()
            .create_sub_block(path, id)
            // Can't get the Origen errors to cast properly to a PyErr For some reason,
            // so have to do this
            .map_err(|e| exceptions::OSError::py_err(e.msg))
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
        // Can't get the Origen errors to cast properly to a PyErr For some reason,
        // so have to do this
        let model = match dut.get_mut_model(path) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };
        model
            .create_reg(memory_map, address_block, id, offset, size)
            // Can't get the Origen errors to cast properly to a PyErr For some reason,
            // so have to do this
            .map_err(|e| exceptions::OSError::py_err(e.msg))
    }

    fn number_of_regs(&self, path: &str) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        // Can't get the Origen errors to cast properly to a PyErr For some reason,
        // so have to do this
        let model = match dut.get_model(path) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };
        Ok(model.number_of_regs())
    }

    fn get_reg(
        &self,
        path: &str,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
    ) -> PyResult<BitCollection> {
        let dut = DUT.lock().unwrap();
        let model = match dut.get_model(path) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };
        let reg = match model.get_reg(memory_map, address_block, id) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };

        Ok(BitCollection::from_reg(
            path,
            memory_map,
            address_block,
            reg,
        ))
    }
}
