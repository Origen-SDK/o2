use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use origen::core::model::pins::pin::Pin;
use pyo3::types::{PyList};

#[pymodule]
/// Implements the module _origen.model in Python
pub fn pins(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PinContainer>()?;
    Ok(())
}

#[pyclass]
struct PinContainer {
    pin_container: origen::core::model::pins::PinContainer,
}

#[pymethods]
impl PinContainer {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({
            PinContainer {
                pin_container: origen::core::model::pins::PinContainer::new(),
            }
        })
    }

    fn add_pin(&mut self, name: &str) -> PyResult<()> {
        self.pin_container.add_pin(name.to_string());
        Ok(())
    }

    fn pin_names(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<String> = Vec::new();
        for (n, p) in &self.pin_container.pins {
            v.push(n.clone());
        }
        let l = PyList::new(py, &v);
        Ok(l.into())
    }

    fn pin_fields_for(&mut self, name: &str) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ret = pyo3::types::PyDict::new(py);
        let p = self.pin_container.get_pin(name);

        // Cast all the available fields to PyObjects and add them to the return
        // dictionary
        let _ = ret.set_item("postured_state", p.postured_state);
        let _ = ret.set_item("action", p.action.as_str());
        //let _ = ret.set_item("role", p.role);
        Ok(ret.into())
    }
    
    /// Returns a Dictionary where each key is the pin name and the associated
    /// value is another Dictionary containing all the pin fields, as would be returned
    /// by pin_fields_for(...)
    fn pins(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        // Top dictionary which will contain all the keys
        let pins = pyo3::types::PyDict::new(py);
        for (n, p) in &self.pin_container.pins {
            let pin_fields = pyo3::types::PyDict::new(py);
            pin_fields.set_item("postured_state", p.postured_state);
            pin_fields.set_item("action", p.action.as_str());
            pins.set_item(n.clone(), pin_fields);
        }
        Ok(pins.into())
    }

    fn unique_pins(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<String> = Vec::new();
        for (n, p) in &self.pin_container.pins {
            v.push(n.clone());
        }
        let l = PyList::new(py, &v);
        Ok(l.into())
    }
}