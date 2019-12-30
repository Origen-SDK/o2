use origen::DUT;
use origen::error::Error;
use pyo3::prelude::*;
use pyo3::{exceptions, PyErr};
use super::pin::Pin;
use super::pin_collection::PinCollection;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes, PySlice};

#[pyclass]
pub struct PinGroup {
    pub id: String,
    pub path: String,
    pub model_id: usize,
}

#[pymethods]
impl PinGroup {
    // Even though we're storing the id in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_id(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let grp = model.pin(&self.id);
        match grp {
            Some(_grp) => {
                Ok(_grp.id.clone())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.get_pin_group_data(&self.id))
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        model.set_pin_group_data(&self.id, data)?;
        
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, Self {
          id: self.id.clone(),
          path: self.path.clone(),
          model_id: self.model_id,
        }).unwrap())
    }

    fn set(&self, data: u32) -> PyResult<Py<Self>> {
        return self.set_data(data);
    }

    fn with_mask(&self, mask: usize) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        model.set_pin_group_nonsticky_mask(&self.id, mask)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, Self {
          id: self.id.clone(),
          path: self.path.clone(),
          model_id: self.model_id,
        }).unwrap())
    }

    #[getter]
    fn get_pin_ids(&self) -> PyResult<Vec<String>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let grp = model.pin(&self.id);

        let mut v: Vec<String> = Vec::new();
        match grp {
            Some(_grp) => {
                for n in _grp.pin_ids.iter() {
                    v.push(n.clone());
                }
                Ok(v)
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

    #[getter]
    fn get_pins(&self) -> PyResult<Vec<Py<Pin>>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let grp = model.pin(&self.id);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<Py<Pin>> = Vec::new();
        match grp {
            Some(_grp) => {
                for n in _grp.pin_ids.iter() {
                    v.push(Py::new(py, Pin {
                        id: String::from(n),
                        path: String::from(&self.path),
                        model_id: self.model_id,
                    }).unwrap());
                }
                Ok(v)
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.get_pin_actions_for_group(&self.id)?)
    }

    fn drive(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      Ok(model.drive_pin_group(&self.id, data, Option::None)?)
    }

    fn verify(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      Ok(model.verify_pin_group(&self.id, data, Option::None)?)
    }

    fn capture(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      Ok(model.capture_pin_group(&self.id, Option::None)?)
    }

    fn highz(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      Ok(model.highz_pin_group(&self.id, Option::None)?)
    }

    fn reset(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      Ok(model.reset_pin_group(&self.id)?)
    }

    #[getter]
    fn get_physical_ids(&self) -> PyResult<Vec<String>> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(self.model_id)?;
      let ids = model.resolve_pin_group_ids(&self.id)?;
      Ok(ids.clone())
   }

   #[getter]
   fn get_width(&self) -> PyResult<usize> {
     let mut dut = DUT.lock().unwrap();
     let model = dut.get_mut_model(self.model_id)?;
     let grp = model.pin(&self.id).unwrap();
     Ok(grp.len())
   }

   #[getter]
   fn get_reset_data(&self) -> PyResult<u32> {
     let mut dut = DUT.lock().unwrap();
     let model = dut.get_mut_model(self.model_id)?;
     Ok(model.get_pin_group_reset_data(&self.id))
   }

   #[getter]
   fn get_reset_actions(&self) -> PyResult<String> {
     let mut dut = DUT.lock().unwrap();
     let model = dut.get_mut_model(self.model_id)?;
     Ok(model.get_pin_reset_actions_for_group(&self.id)?)
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

    #[getter]
    fn get_big_endian(&self) -> PyResult<bool> {
        let is_little_endian = self.get_little_endian()?;
        Ok(!is_little_endian)
    }

    #[getter]
    fn get_little_endian(&self) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let grp = model.pin(&self.id);

        match grp {
            Some(_grp) => {
                Ok(_grp.is_little_endian())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let grp = model.pin(&self.id);
        match grp {
            Some(_grp) => {
                Ok(_grp.len())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", self.id)))
        }
    }

    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.pin_group_contains(&self.id, item)?)
    }
}

#[pyproto]
impl<'p> pyo3::class::PyMappingProtocol<'p> for PinGroup {
    // Indexing example: https://github.com/PyO3/pyo3/blob/master/tests/test_dunder.rs#L423-L438
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Ok(slice) = idx.cast_as::<PySlice>() {
          // Indices requires (what I think is) a max size. Should be plenty.
          let indices = slice.indices(8192)?;
          let collection = model.slice_pin_group(&self.id, indices.start as usize, indices.stop as usize, indices.step as usize)?;
          Ok(Py::new(py, PinCollection::from(collection)).unwrap().to_object(py))
        } else {
          let i = idx.extract::<isize>().unwrap();
          let collection = model.slice_pin_group(&self.id, i as usize, i as usize, 1)?;
          Ok(Py::new(py, PinCollection::from(collection)).unwrap().to_object(py))
        }
      }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroup {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinGroupIter> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(slf.model_id)?;
        let grp = model.pin(&slf.id).unwrap();

        Ok(PinGroupIter {
            keys: grp.pin_ids.clone(),
            i: 0,
            path: slf.path.clone(),
            model_id: slf.model_id,
        })
    }
}

#[pyclass]
pub struct PinGroupIter {
  keys: Vec<String>,
  i: usize,
  path: String,
  model_id: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroupIter {
  fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(slf.to_object(py))
  }

  fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PinCollection>> {
    if slf.i >= slf.keys.len() {
        return Ok(None)
    }
    let id = slf.keys[slf.i].clone();
    let collection = PinCollection::new(slf.model_id, vec!(id), Option::None)?;
    slf.i += 1;
    Ok(Some(collection))
  }
}
