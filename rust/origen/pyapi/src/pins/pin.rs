use origen::DUT;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};

#[pyclass]
pub struct Pin {
    pub id: String,
    pub path: String,
}

#[macro_export]
macro_rules! pypin {
  ($py:expr, $id:expr, $path:expr) => {
      Py::new($py, crate::pins::pin::Pin {
        id: String::from($id),
        path: String::from($path),
      }).unwrap()
  }
}

#[pymethods]
impl Pin {

    // Even though we're storing the id in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_id(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model._pin(&self.id)?;
        Ok(pin.id.clone())
    }

    #[getter]
    fn get_data(&self) -> PyResult<u8> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model._pin(&self.id)?;
        Ok(pin.data)
    }

    #[getter]
    fn get_action(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model._pin(&self.id)?;
        Ok(String::from(pin.action.as_str()))
    }

    #[getter]
    fn get_aliases(&self) -> PyResult<Vec<String>> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      let pin = model._pin(&self.id)?;
      Ok(pin.aliases.clone())
    }

    // Debug helper: Get the id held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__id(&self) -> PyResult<String> {
        Ok(self.id.clone())
    }

    // Debug helper: Get the id held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__path(&self) -> PyResult<String> {
        Ok(self.path.clone())
    }
}