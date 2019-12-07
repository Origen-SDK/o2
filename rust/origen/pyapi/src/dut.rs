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
    store: DUT,
}

#[pymethods]
impl PyDUT {
    #[new]
    fn new(obj: &PyRawObject, id: String) {
        obj.init({
            PyDUT {
                store: DUT::new(id),
            }
        });
    }

    /// Creates a new model at the given path
    fn create_sub_block(&mut self, path: &str, id: &str) -> PyResult<()> {
        self.store
            .create_sub_block(path, id)
            .map_err(|e| exceptions::OSError::py_err(e.msg))
    }
}
