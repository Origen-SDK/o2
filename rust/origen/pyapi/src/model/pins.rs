use origen::error::Error;
use origen::core::model::pins::pin::PinActions;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

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

    // fn add_pin(&mut self, name: &str) -> PyResult<()> {
    //     self.pin_container.add_pin(name.to_string());
    //     Ok(())
    // }

    // fn pin_fields_for(&mut self, name: &str) -> PyResult<PyObject> {
    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     let ret = pyo3::types::PyDict::new(py);
    //     let p = self.pin_container.get_pin(name)?;
    //     let _ = ret.set_item("postured_state", p.postured_state);
    //     let _ = ret.set_item("action", p.action.as_str());
    //     Ok(ret.into())
    // }

    // #[args(_kwargs = "**")]
    // fn update_pin_fields_for(&mut self, name: &str, _kwargs: Option<&PyDict>) -> PyResult<()> {
    //     let pin = self.pin_container.get_pin(name)?;
    //     if let Some(kwargs) = _kwargs {
    //         for(field, val) in kwargs {
    //           let f: String = field.extract()?;
    //           if f == "postured_state" {
    //             pin.postured_state = val.extract()?;
    //           } else if f == "action" {
    //             pin.action = PinActions::from_str(val.extract()?).unwrap();
    //           } else if f == "add_alias" {
    //             pin.add_alias(field.extract()?);
    //           } else {
    //             return Err(PyErr::from(Error::new(&format!("Unknown pin field '{}'", f))));
    //           }
    //         }
    //     }
    //     Ok(())
    // }

    // fn unique_pins(&self) -> PyResult<PyObject> {
    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     let mut v: Vec<String> = Vec::new();
    //     for (n, _p) in &self.pin_container.pins {
    //         v.push(n.clone());
    //     }
    //     let l = PyList::new(py, &v);
    //     Ok(l.into())
    // }
}
