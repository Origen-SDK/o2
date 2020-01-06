use origen::DUT;
use pyo3::prelude::*;
use pyo3::class::mapping::*;
use pyo3::exceptions;
use origen::error::Error;

#[macro_export]
macro_rules! pytimeset_container {
  ($py:expr, $model_id:expr) => {
    Py::new($py, TimesetContainer {model_id: $model_id}).unwrap()
  };
}

#[pyclass]
pub struct TimesetContainer {
  pub model_id: usize,
}

#[pymethods]
impl TimesetContainer {
  fn keys(&self) -> PyResult<Vec<String>> {
    super::super::timesets::DictLikeAPI::keys(self)
    // let mut dut = DUT.lock().unwrap();
    // let model = dut.get_model(self.model_id)?;
    // let names = &model.timesets;
    // Ok(names.iter().map(|(k, _)| k.clone()).collect())
  }
}

impl super::super::timesets::DictLikeAPI for TimesetContainer {
  //type IdMapper = HashMap<String, usize>; //origen::core::model::timesets::Timesets;
  type PyItem = super::timeset::Timeset;
  // , model: &origen::core::model::Model

  fn lookup_key(&self) -> &str {
    &"timesets"
  }

  fn model_id(&self) -> usize {
    self.model_id
  }

  fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error> {
    //Ok(super::timeset::Timeset::new(name, model_id))
    Ok(Py::new(py, super::timeset::Timeset::new(name, model_id)).unwrap().to_object(py))
  }
}

#[pyproto]
impl PyMappingProtocol for TimesetContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let gil = Python::acquire_gil();
        let py = gil.python();
        match pytimeset!(py, model, self.model_id, name) {
          Ok(t) => Ok(t),
          Err(_) => Err(exceptions::KeyError::py_err(format!(
            "No timeset found for {}",
            name
          ))),
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.timesets.len())
    }
}