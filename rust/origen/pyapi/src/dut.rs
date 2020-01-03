use crate::pins::PyInit_pins;
use origen::DUT;
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};
use origen::error::Error;

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
pub struct PyDUT {
    metadata: Vec<PyObject>,
}

#[pymethods]
impl PyDUT {
    #[new]
    /// Instantiating a new instance of PyDUT means re-loading the target
    fn new(obj: &PyRawObject, name: &str) {
        DUT.lock().unwrap().change(name);
        obj.init({ PyDUT { metadata: vec!() } });
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

    pub fn push_metadata(&mut self, item: &PyAny) -> usize {
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        self.metadata.push(item.to_object(py));
        self.metadata.len() - 1
    }

    pub fn override_metadata_at(&mut self, idx: usize, item: &PyAny) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if self.metadata.len() > idx {
            self.metadata[idx] = item.to_object(py);
            Ok(())
        } else {
            Err(PyErr::from(Error::new(&format!("Overriding metadata at {} exceeds the size of the current metadata vector!", idx))))
        }
    }

    pub fn get_metadata(&self, idx: usize) -> PyResult<&PyObject> {
        Ok(&self.metadata[idx])
    }
}

impl PyDUT {

}
