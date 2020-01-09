use origen::DUT;
use pyo3::prelude::*;

#[macro_export]
macro_rules! pytimeset {
  ($py:expr, $model:expr, $model_id:expr, $name:expr) => {
    if $model.contains_timeset($name) {
      Ok(Py::new($py, crate::timesets::timeset::Timeset {
        name: String::from($name),
        model_id: $model_id,
      }).unwrap().to_object($py))
    } else {
      // Note: Errors here shouldn't happen. Any errors that arise are either
      // bugs or from the user meta-programming their way into the backend DB.
      Err(PyErr::from(origen::error::Error::new(&format!("No timeset {} has been added on block {}", $name, $model.name))))
    }
  };
}

// Returns a (Python) Timeset or NoneType instance.
// Note: this does NOT return a Rust Option::None, but 
#[macro_export]
macro_rules! pytimeset_or_pynone {
  ($py:expr, $model:expr, $model_id:expr, $name:expr) => {
    if $model.contains_timeset($name) {
      Py::new($py, crate::timesets::timeset::Timeset {
        name: String::from($name),
        model_id: $model_id,
      }).unwrap().to_object($py)
    } else {
      $py.None()
    }
  };
}

#[pyclass]
pub struct Timeset {
  pub name: String,
  pub model_id: usize,
}

#[pymethods]
impl Timeset {

  #[getter]
  fn get_name(&self) -> PyResult<String> {
    let dut = DUT.lock().unwrap();
    let timeset = dut.get_timeset(self.model_id, &self.name);
    Ok(timeset.unwrap().name.clone())
  }

  #[getter]
  fn get_period(&self) -> PyResult<f64> {
    let dut = DUT.lock().unwrap();
    let timeset = dut._get_timeset(self.model_id, &self.name)?;
    Ok(timeset.eval(Option::None)?)
  }

  #[getter]
  fn get_default_period(&self) -> PyResult<PyObject> {
    let dut = DUT.lock().unwrap();
    let timeset = dut._get_timeset(self.model_id, &self.name)?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(match timeset.default_period {
      Some(p) => p.to_object(py),
      None => py.None(),
    })
  }

  #[allow(non_snake_case)]
  #[getter]
  fn get___eval_str__(&self) -> PyResult<String> {
    let dut = DUT.lock().unwrap();
    let timeset = dut._get_timeset(self.model_id, &self.name)?;
    Ok(timeset.eval_str().clone())
  }

  #[allow(non_snake_case)]
  #[getter]
  fn get___period__(&self) -> PyResult<PyObject> {
    let dut = DUT.lock().unwrap();
    let timeset = dut._get_timeset(self.model_id, &self.name)?;
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(match &timeset.period_as_string {
      Some(p) => p.clone().to_object(py),
      None => py.None(),
    })
  }

  // #[getter]
  // fn get_drive_waves(&self) -> PyResult<Py<DriveWaveContainer> {
  //   // ...
  // }
  // #[getter]
  // fn get_verify_waves(&self) -> PyResult<Py<DriveWaveContainer> {
  //   // ...
  // }

}

impl Timeset {
  pub fn new(name: &str, model_id: usize) -> Self {
    Self {
      name: String::from(name),
      model_id: model_id
    }
  }
}