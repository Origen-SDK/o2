use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;

use origen::core::model::Model;

#[pymodule]
/// Implements the module _origen.model in Python
pub fn model(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ModelDB>()?;

    Ok(())
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

    //fn add_reg(&mut self, name: &str, offset: u32) -> PyResult<()> {
    //    self.model.add_reg(name, offset);
    //    Ok(())
    //}

    fn number_of_regs(&self) -> PyResult<usize> {
        Ok(self.model.number_of_regs())
    }
}
