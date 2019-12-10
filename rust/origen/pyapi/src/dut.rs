use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;
use origen::core::dut::DUT;
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
pub struct PyDUT {
    dut: DUT,
}

#[pymethods]
impl PyDUT {
    #[new]
    fn new(obj: &PyRawObject, id: String) {
        obj.init({ PyDUT { dut: DUT::new(id) } });
    }

    /// Creates a new model at the given path
    fn create_sub_block(&mut self, path: &str, id: &str) -> PyResult<()> {
        self.dut
            .create_sub_block(path, id)
            // Can't get the Origen errors to cast properly to a PyErr For some reason,
            // so have to do this
            .map_err(|e| exceptions::OSError::py_err(e.msg))
    }

    fn create_reg(
        &mut self,
        path: &str,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
        offset: u32,
        size: Option<u32>,
    ) -> PyResult<()> {
        // Can't get the Origen errors to cast properly to a PyErr For some reason,
        // so have to do this
        let model = match self.dut.get_mut_model(path) {
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
        // Can't get the Origen errors to cast properly to a PyErr For some reason,
        // so have to do this
        let model = match self.dut.get_model(path) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };
        Ok(model.number_of_regs())
    }
}
