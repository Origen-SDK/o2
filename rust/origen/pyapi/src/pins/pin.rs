use origen::DUT;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};

#[pyclass]
pub struct Pin {
    pub name: String,
    pub path: String,
    pub model_id: usize,
}

#[macro_export]
macro_rules! pypin {
  ($py:expr, $name:expr, $model_id:expr) => {
      Py::new($py, crate::pins::pin::Pin {
        name: String::from($name),
        path: String::from(""),
        model_id: model_id,
      }).unwrap()
  }
}

#[pymethods]
impl Pin {

    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let pin = model._pin(&self.name)?;
        Ok(pin.name.clone())
    }

    #[getter]
    fn get_data(&self) -> PyResult<u8> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let pin = model._pin(&self.name)?;
        Ok(pin.data)
    }

    #[getter]
    fn get_action(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let pin = model._pin(&self.name)?;
        Ok(String::from(pin.action.as_str()))
    }

    #[getter]
    fn get_aliases(&self) -> PyResult<Vec<String>> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      let pin = model._pin(&self.name)?;
      Ok(pin.aliases.clone())
    }

    #[getter]
    fn get_reset_data(&self) -> PyResult<PyObject> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      let pin = model._get_physical_pin(&self.name)?;
      
      let gil = Python::acquire_gil();
      let py = gil.python();
      match pin.reset_data {
        Some(d) => {
          Ok(d.to_object(py))
        },
        None => {
          Ok(py.None())
        }
      }
    }

    #[getter]
    fn get_reset_action(&self) -> PyResult<PyObject> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      let pin = model._get_physical_pin(&self.name)?;
      
      let gil = Python::acquire_gil();
      let py = gil.python();
      match pin.reset_action {
        Some(a) => {
          Ok(String::from(a.as_char().to_string()).to_object(py))
        },
        None => {
          Ok(py.None())
        }
      }
    }

    // Debug helper: Get the name held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__name(&self) -> PyResult<String> {
        Ok(self.name.clone())
    }

    // Debug helper: Get the path held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__path(&self) -> PyResult<String> {
        Ok(self.path.clone())
    }
}