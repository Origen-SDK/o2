use pyo3::class::basic::PyObjectProtocol;
use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;

use origen::core::model::Model;

#[pymodule]
/// Implements the module _origen.model in Python
pub fn model(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ModelDB>()?;
    m.add_class::<MemoryMaps>()?;

    Ok(())
}

#[pyclass]
#[derive(Debug)]
pub struct MemoryMaps {}

#[pymethods]
impl MemoryMaps {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({ MemoryMaps {} });
    }

    fn __getattr__(&self, name: String) {
        println!("Yo, you called reg: {}", name);
    }
}

#[pyproto]
impl PyObjectProtocol for MemoryMaps {
    fn __repr__(&self) -> PyResult<String> {
        Ok("Yo dawg".to_string())
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
                model: Model::new(name),
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
