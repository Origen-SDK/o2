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
use origen::error::Error;

pub trait DictLikeAPI {
  //type NameToIdMapper;
  //type ItemContainer;
  //type IdMapper;
  type PyItem;

  fn model_id(&self) -> usize;
  //fn id_mapping(&self, model: &origen::core::model::Model) -> &IndexMap<String, usize>;
  //fn get_item()
  fn lookup_key(&self) -> &str;
  //fn get_item(&self) -> PyObject;
  fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error>;

  fn keys(&self) -> PyResult<Vec<String>> {
    let dut = DUT.lock().unwrap();
    let model = dut.get_model(self.model_id())?;
    let names = model.lookup(self.lookup_key())?;
    Ok(names.iter().map(|(k, _)| k.clone()).collect())
  }

  fn values(&self) -> PyResult<Vec<PyObject>> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut v: Vec<PyObject> = Vec::new();
    for (n, _item) in items {
        v.push(self.new_pyitem(py, n, self.model_id())?);
    }
    Ok(v)
  }

  fn items(&self) -> PyResult<Vec<(String, PyObject)>> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut _items: Vec<(String, PyObject)> = Vec::new();
    for (n, _item) in items.iter() {
        _items.push((
            n.clone(),
            self.new_pyitem(py, &n, self.model_id())?
        ));
    }
    Ok(_items)
  }

  // Functions for PyMappingProtocol
  fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();
    //let item = dut.get_item("timesets", self.model_id(), name)?;
    let model = dut.get_mut_model(self.model_id())?;
    let item = model.lookup(self.lookup_key())?.get(name);

    let gil = Python::acquire_gil();
    let py = gil.python();
    match item {
        Some(_item) => Ok(self.new_pyitem(py, name, self.model_id())?),
        None => Err(pyo3::exceptions::KeyError::py_err(format!(
            "No pin or pin alias found for {}",
            name
        ))),
    }
  }

  fn __len__(&self) -> PyResult<usize> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;
    Ok(items.len())
  }
}

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
/// Implements the module _origen.model in Python
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