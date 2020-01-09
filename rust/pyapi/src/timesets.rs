use origen::DUT;
use super::dut::PyDUT;
use pyo3::prelude::*;

#[macro_use]
mod timeset;
#[macro_use]
mod timeset_container;

use timeset::Timeset;
use timeset_container::TimesetContainer;
use pyo3::types::{PyDict, PyAny};

#[macro_export]
macro_rules! type_error {
  ($message:expr) => {
    Err(pyo3::exceptions::TypeError::py_err(format!(
      "{}",
      $message
    )))
  };
}

#[pymodule]
pub fn timesets(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TimesetContainer>()?;
    m.add_class::<Timeset>()?;
    Ok(())
}

#[pymethods]
impl PyDUT {
  #[args(kwargs = "**")]
  fn add_timeset(&self, model_id: usize, name: &str, period: &PyAny, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();

    dut.create_timeset(
      model_id,
      name,
      if let Ok(p) = period.extract::<String>() {
        Some(Box::new(p))
      } else if let Ok(p) = period.extract::<f64>() {
        Some(Box::new(p))
      } else if period.get_type().name() == "NoneType" {
        Option::None
      } else {
        return type_error!("Could not convert 'period' argument to String or NoneType!");
      },
      match kwargs {
        Some(args) => {
          match args.get_item("default_period") {
            Some(arg) => Some(arg.extract::<f64>()?),
            None => Option::None,
          }
        },
        None => Option::None,
      }
    )?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let model = dut.get_mut_model(model_id)?;
    Ok(pytimeset!(py, model, model_id, name)?)
  }

  fn timeset(&self, model_id: usize, name: &str) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(model_id)?;
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(pytimeset_or_pynone!(py, model, model_id, name))
  }

  fn timesets(&self, model_id: usize) -> PyResult<Py<TimesetContainer>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(pytimeset_container!(py, model_id))
  }
}