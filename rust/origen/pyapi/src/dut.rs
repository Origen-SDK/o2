use crate::pins::PyInit_pins;
use crate::registers::PyInit_registers;
use pyo3::prelude::*;
use pyo3::wrap_pymodule;

/// Implements the module _origen.dut in Python which exposes all
/// DUT-related APIs
#[pymodule]
pub fn dut(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDUT>()?;
    m.add_wrapped(wrap_pymodule!(pins))?;
    m.add_wrapped(wrap_pymodule!(registers))?;
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
        origen::dut().change(name);
        obj.init({ PyDUT {} });
    }

    /// Creates a new model at the given path
    fn create_model(&self, parent_id: Option<usize>, name: &str) -> PyResult<usize> {
        Ok(origen::dut().create_model(parent_id, name)?)
    }

    fn model_console_display(&self, model_id: usize) -> PyResult<String> {
        let dut = origen::dut();
        let model = dut.get_model(model_id)?;
        Ok(model.console_display(&dut)?)
    }
}
