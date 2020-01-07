use pyo3::prelude::*;
use pyo3::class::mapping::*;
use origen::error::Error;
use super::super::meta::py_like_apis::dict_like_api::{DictLikeAPI, DictLikeIter};

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
    DictLikeAPI::keys(self)
  }

  fn values(&self) -> PyResult<Vec<PyObject>> {
    DictLikeAPI::values(self)
  }

  fn items(&self) -> PyResult<Vec<(String, PyObject)>> {
    DictLikeAPI::items(self)
  }

  fn get(&self, name: &str) -> PyResult<PyObject> {
    DictLikeAPI::get(self, name)
  }
}

impl DictLikeAPI for TimesetContainer {
  fn lookup_key(&self) -> &str {
    &"timesets"
  }

  fn model_id(&self) -> usize {
    self.model_id
  }

  fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error> {
    Ok(Py::new(py, super::timeset::Timeset::new(name, model_id)).unwrap().to_object(py))
  }
}

#[pyproto]
impl PyMappingProtocol for TimesetContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
      DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
      DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for TimesetContainer {
  fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
    DictLikeAPI::__iter__(&*slf)
  }
}
