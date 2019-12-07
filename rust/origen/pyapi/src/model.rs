use origen::LOGGER;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use origen::core::model::Model;

#[pymodule]
/// Implements the module _origen.model in Python
pub fn model(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ModelDB>()?;
    m.add_class::<MemoryMaps>()?;

    m.add_wrapped(wrap_pyfunction!(memory_maps))?;

    Ok(())
}

#[pyfunction]
fn memory_maps() -> PyResult<Py<MemoryMaps>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    Py::new(py, MemoryMaps {})
}

/// Implements the user API to work with a model's collection of memory maps, an instance
/// of this is returned by my_model.memory_maps
#[pyclass]
#[derive(Debug)]
pub struct MemoryMaps {}

#[pymethods]
impl MemoryMaps {}

#[pyproto]
impl PyObjectProtocol for MemoryMaps {
    fn __repr__(&self) -> PyResult<String> {
        LOGGER.error("Memory map not found!");
        Ok("Here should be a nice graphic of the memory maps".to_string())
    }
}

#[pyclass]
#[derive(Debug)]
pub struct ModelDB {
    model: Model,
}

#[pymethods]
impl ModelDB {
    #[new]
    fn new(obj: &PyRawObject, name: String) {
        obj.init({
            ModelDB {
                model: Model::new(name, "".to_string()),
            }
        });
    }

    fn add_reg(
        &mut self,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
        offset: u32,
        size: Option<u32>,
    ) -> PyResult<()> {
        self.model
            .add_reg(memory_map, address_block, id, offset, size);
        Ok(())
    }

    fn number_of_regs(&self) -> PyResult<usize> {
        Ok(self.model.number_of_regs())
    }
}
