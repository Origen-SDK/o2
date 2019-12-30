use origen::DUT;
use pyo3::prelude::*;
use pyo3::{exceptions};
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};
use pyo3::class::mapping::*;

use super::pin_group::PinGroup;
use super::pin_collection::PinCollection;
use origen::core::model::pins::Endianness;

#[pyclass]
pub struct PinContainer {
    pub path: String,
    pub model_id: usize,
}

#[pymethods]
impl PinContainer {
    fn keys(&self) -> PyResult<Vec<String>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let ids = &model.pins;

        let mut v: Vec<String> = Vec::new();
        for (n, _p) in ids {
            v.push(n.clone());
        }
        Ok(v)
    }

    fn values(&self) -> PyResult<Vec<Py<PinGroup>>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let pins = &model.pins;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<Py<PinGroup>> = Vec::new();
        for (n, _p) in pins {
            v.push(Py::new(py, PinGroup {
                id: String::from(n.clone()),
                path: String::from(self.path.clone()),
                model_id: self.model_id,
              }).unwrap())
        }
        Ok(v)
    } 

    fn items(&self) -> PyResult<Vec<(String, Py<PinGroup>)>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let pins = &model.pins;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut items: Vec<(String, Py<PinGroup>)> = Vec::new();
        for (n, _p) in pins {
            items.push((
                n.clone(),
                Py::new(py, PinGroup {
                    id: String::from(n.clone()),
                    path: String::from(self.path.clone()),
                    model_id: self.model_id,
                  }).unwrap(),
            ));
        }
        Ok(items)
    }

    #[getter]
    fn get_ids(&self) -> PyResult<Vec<String>> {
        self.keys()
    }

    #[args(ids = "*", options = "**")]
    fn collect(&self, ids: &PyTuple, options: Option<&PyDict>) -> PyResult<Py<PinCollection>> {
      let gil = Python::acquire_gil();
      let py = gil.python();
      let mut endianness = Option::None;
      match options {
        Some(options) => {
          if let Some(opt) = options.get_item("little_endian") {
              if opt.extract::<bool>()? {
                  endianness = Option::Some(Endianness::LittleEndian);
              } else {
                  endianness = Option::Some(Endianness::BigEndian);
              }
          }
        },
        None => {}
      }

      let mut id_strs: Vec<String> = vec!();
      let mut regex_indices: Vec<usize> = vec!();
      for (i, id) in ids.iter().enumerate() {
        let t = id.get_type().name();
        if id.get_type().name() == "re.Pattern" {
          let r = id.getattr("pattern").unwrap();
          id_strs.push(format!("/{}/", r));
        } else if let _id = id.extract::<String>()? {
          id_strs.push(_id.clone());
        }
      }
      let collection = PinCollection::new(self.model_id, id_strs, endianness)?;
      let c = Py::new(py, collection).unwrap();
      Ok(c)
  }
}

#[pyproto]
impl PyMappingProtocol for PinContainer {
    fn __getitem__(&self, id: &str) -> PyResult<Py<PinGroup>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                  id: String::from(id),
                  path: String::from(&self.path),
                  model_id: self.model_id,
              }).unwrap())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin or pin alias found for {}", id)))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.number_of_ids())
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinContainerIter> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(slf.model_id)?;
        Ok(PinContainerIter {
            keys: model.pins.iter().map(|(s, _)| s.clone()).collect(),
            i: 0,
        })
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinContainer {
    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.contains(item))
    }
}

#[pyclass]
pub struct PinContainerIter {
    keys: Vec<String>,
    i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinContainerIter {

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    /// The Iterator will be created with an index starting at 0 and the pin ids at the time of its creation.
    /// For each call to 'next', we'll create a pin object with the next value in the list, or None, if no more keys are available.
    /// Note: this means that the iterator can become stale if the PinContainer is changed. This can happen if the iterator is stored from Python code
    ///  directly. E.g.: i = dut.pins.__iter__() => iterator with the pin ids at the time of creation,
    /// Todo: Fix the above using iterators. My Rust skills aren't there yet though... - Coreyeng
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None)
        }
        let id = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(id))
    }
}
